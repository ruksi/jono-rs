use crate::consumer_config::ConsumerConfig;
use crate::{WorkSummary, Worker, Workload};
use jono_core::{Context, Inspector, JonoError, Result, current_timestamp_ms};
use redis::AsyncCommands;
use serde_json::json;
use std::thread;

/// Interface for getting, processing and resolving jobs from Jono queues.
pub struct Consumer<W: Worker> {
    context: Context,
    config: ConsumerConfig,
    worker: W,
}

impl<W: Worker> Consumer<W> {
    pub fn with_context(context: Context, worker: W) -> Self {
        Self {
            context,
            config: ConsumerConfig::default(),
            worker,
        }
    }

    pub fn with_config(mut self, config: ConsumerConfig) -> Self {
        self.config = config;
        self
    }

    pub async fn run(&self) -> Result<()> {
        let mut consecutive_errors = 0;

        loop {
            match self.run_next().await {
                Ok(Some(_)) => {
                    consecutive_errors = 0;
                }
                Ok(None) => {
                    thread::sleep(self.config.get_poll_interval());
                }
                Err(e) => {
                    consecutive_errors += 1;
                    eprintln!("Error processing job: {}", e);

                    if consecutive_errors >= self.config.get_max_consecutive_errors() {
                        return Err(JonoError::TooManyErrors(consecutive_errors));
                    }
                    thread::sleep(self.config.get_poll_interval());
                }
            }
        }
    }

    pub async fn run_next(&self) -> Result<Option<WorkSummary>> {
        match self.start_next_job().await? {
            Some(metadata) => {
                let summary = self.process_job(metadata).await?;
                Ok(Some(summary))
            }
            None => Ok(None),
        }
    }

    async fn start_next_job(&self) -> Result<Option<Workload>> {
        let mut conn = self.get_connection().await?;
        let timeout = self.config.get_poll_timeout().as_secs_f64();
        let keys = self.context.keys();

        let entries: Vec<(String, String, i64)> = conn.bzpopmin(keys.queued_set(), timeout).await?;

        if let Some((_set, job_id, _score)) = entries.first() {
            let inspector = Inspector::with_context(self.context.clone());
            let metadata = inspector.get_job_metadata(job_id).await?;
            let workload = Workload::from_metadata(metadata);
            self.mark_job_as_started(job_id).await?;
            Ok(Some(workload))
        } else {
            Ok(None)
        }
    }

    async fn mark_job_as_started(&self, job_id: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let keys = self.context.keys();
        let now = current_timestamp_ms();
        let expiry = now + self.config.get_heartbeat_timeout().as_millis() as i64;
        let metadata_key = keys.job_metadata_hash(job_id);

        let _: () = redis::pipe()
            .atomic()
            .zadd(keys.started_set(), job_id, expiry)
            .hset(&metadata_key, "status", "started")
            .hset(&metadata_key, "started_at", now.to_string())
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    async fn process_job(&self, workload: Workload) -> Result<WorkSummary> {
        let inspector = Inspector::with_context(self.context.clone());
        if !inspector.job_exists(&workload.job_id).await? {
            return Ok(WorkSummary::Failure("Job no longer exists".to_string()));
        }
        if inspector.is_job_aborted(&workload.job_id).await? {
            return Ok(WorkSummary::Failure("Job was canceled".to_string()));
        }

        let summary = self.worker.work(&workload).await?;

        match &summary {
            WorkSummary::Success(summary_data) => {
                self.complete_job(&workload.job_id, summary_data.clone())
                    .await?;
            }
            WorkSummary::Failure(error_message) => {
                eprintln!("Job {} failed: {}", workload.job_id, error_message);
                todo!();
            }
        }

        Ok(summary)
    }

    async fn complete_job(
        &self,
        job_id: &str,
        work_summary: Option<serde_json::Value>,
    ) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let keys = self.context.keys();

        let summ_data = work_summary.unwrap_or(json!(null));
        let summ_json = serde_json::to_string(&summ_data)?;
        let metadata_key = keys.job_metadata_hash(job_id);

        let now = current_timestamp_ms();
        let ttl_ms = 24 * 60 * 60 * 1000; // 24 hours to collect the work summaries
        let expiry_time_score = now + ttl_ms;

        let _: () = redis::pipe()
            .atomic()
            .zrem(keys.started_set(), job_id)
            .zadd(keys.completed_set(), job_id, expiry_time_score)
            .hset(&metadata_key, "status", "completed")
            .hset(&metadata_key, "completed_at", now.to_string())
            .hset(&metadata_key, "work_summary", summ_json)
            .expire(&metadata_key, ttl_ms / 1000)
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    async fn get_connection(&self) -> Result<impl redis::aio::ConnectionLike> {
        self.context.get_connection().await
    }
}
