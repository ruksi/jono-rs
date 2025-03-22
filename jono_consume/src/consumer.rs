use crate::consumer_config::ConsumerConfig;
use crate::{Outcome, Worker, Workload};
use jono_core::{Context, Error, Inspector, Result, current_timestamp_ms};
use redis::{Commands, Connection};
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

    pub fn run(&self) -> Result<()> {
        let mut consecutive_errors = 0;

        loop {
            match self.run_next() {
                Ok(Some(_)) => {
                    consecutive_errors = 0;
                }
                Ok(None) => {
                    thread::sleep(self.config.get_polling_interval());
                }
                Err(e) => {
                    consecutive_errors += 1;
                    eprintln!("Error processing job: {}", e);

                    if consecutive_errors >= self.config.get_max_consecutive_errors() {
                        return Err(Error::InvalidJob(format!(
                            "Too many consecutive errors ({})",
                            consecutive_errors
                        )));
                    }
                    thread::sleep(self.config.get_polling_interval());
                }
            }
        }
    }

    pub fn run_next(&self) -> Result<Option<Outcome>> {
        match self.acquire_next_job()? {
            Some(metadata) => {
                let outcome = self.process_job(metadata)?;
                Ok(Some(outcome))
            }
            None => Ok(None),
        }
    }

    fn acquire_next_job(&self) -> Result<Option<Workload>> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        let entries: Vec<(String, i64)> =
            conn.zpopmin(keys.queued_set(), 1).map_err(Error::Redis)?;

        if let Some((job_id, _)) = entries.first() {
            let inspector = Inspector::with_context(self.context.clone());
            let metadata = inspector.get_job_metadata(job_id)?;
            let workload = Workload::from_metadata(metadata);
            self.mark_job_running(job_id)?;
            Ok(Some(workload))
        } else {
            Ok(None)
        }
    }

    fn mark_job_running(&self, job_id: &str) -> Result<()> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();
        let now = current_timestamp_ms();
        let expiry = now + self.config.get_heartbeat_timeout().as_millis() as i64;
        let metadata_key = keys.job_metadata_hash(job_id);

        let _: () = redis::pipe()
            .zadd(keys.running_set(), job_id, expiry)
            .hset(&metadata_key, "status", "running")
            .hset(&metadata_key, "started_at", now.to_string())
            .query(&mut conn)
            .map_err(Error::Redis)?;

        Ok(())
    }

    fn process_job(&self, workload: Workload) -> Result<Outcome> {
        let inspector = Inspector::with_context(self.context.clone());
        if !inspector.job_exists(&workload.job_id)? {
            return Ok(Outcome::Failure("Job no longer exists".to_string()));
        }
        if inspector.job_is_canceled(&workload.job_id)? {
            return Ok(Outcome::Failure("Job was canceled".to_string()));
        }

        let outcome = self.worker.process(&workload)?;

        match &outcome {
            Outcome::Success(outcome_data) => {
                self.complete_job(&workload.job_id, outcome_data.clone())?;
            }
            Outcome::Failure(error_message) => {
                eprintln!("Job {} failed: {}", workload.job_id, error_message);
                todo!();
            }
        }

        Ok(outcome)
    }

    fn complete_job(&self, job_id: &str, outcome: Option<serde_json::Value>) -> Result<()> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        let out_data = outcome.unwrap_or(json!(null));
        let out_json = serde_json::to_string(&out_data).map_err(Error::Serialization)?;
        let metadata_key = keys.job_metadata_hash(job_id);

        let now = current_timestamp_ms();
        let _: () = redis::pipe()
            .zrem(keys.running_set(), job_id)
            .hset(&metadata_key, "status", "completed")
            .hset(&metadata_key, "completed_at", now.to_string())
            .hset(&metadata_key, "outcome", out_json)
            .query(&mut conn)
            .map_err(Error::Redis)?;

        Ok(())
    }

    fn get_connection(&self) -> Result<Connection> {
        self.context.get_connection()
    }
}
