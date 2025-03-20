use crate::{Outcome, Worker, Workload};
use jono_core::{Inspector, JobMetadata, JonoContext, JonoError, JonoResult, current_timestamp_ms};
use redis::{Commands, Connection};
use serde_json::json;
use std::sync::Arc;

/// Interface for getting, processing and resolving jobs from Jono queues.
pub struct Consumer {
    context: JonoContext,
    handler: Arc<dyn Worker>,
}

impl Consumer {
    pub fn with_context(context: JonoContext, handler: Arc<dyn Worker>) -> Self {
        Self { context, handler }
    }

    pub fn acquire_next_job(&self) -> JonoResult<Option<JobMetadata>> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        let entries: Vec<(String, i64)> = conn
            .zpopmin(keys.queued_set(), 1)
            .map_err(JonoError::Redis)?;

        if let Some((job_id, _)) = entries.first() {
            let inspector = Inspector::with_context(self.context.clone());
            let metadata = inspector.get_job_metadata(job_id)?;
            self.start_job(job_id)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    pub fn process_job(&self, metadata: JobMetadata) -> JonoResult<Outcome> {
        let workload = Workload {
            metadata: metadata.clone(),
            context: self.context.clone(),
        };

        let inspector = Inspector::with_context(self.context.clone());
        if !inspector.job_exists(&metadata.id)? {
            return Ok(Outcome::Failure("Job no longer exists".to_string()));
        }
        if inspector.job_is_canceled(&metadata.id)? {
            return Ok(Outcome::Failure("Job was cancelled".to_string()));
        }

        let outcome = self.handler.handle_job(workload);
        match &outcome {
            Ok(Outcome::Success(outcome_data)) => {
                self.complete_job(&metadata.id, outcome_data.clone())?;
            }
            Ok(Outcome::Failure(_error_message)) => {
                todo!();
            }
            Err(e) => {
                let _error_message = format!("Error processing job: {}", e);
                todo!();
            }
        }

        outcome
    }

    fn start_job(&self, job_id: &str) -> JonoResult<()> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();
        let now = current_timestamp_ms();
        let expiry = now + self.handler.heartbeat_expiry().as_millis() as i64;
        let metadata_key = keys.job_metadata_hash(job_id);

        let _: () = redis::pipe()
            .zadd(keys.running_set(), job_id, expiry)
            .hset(&metadata_key, "status", "running")
            .hset(&metadata_key, "started_at", now.to_string())
            .query(&mut conn)
            .map_err(JonoError::Redis)?;

        Ok(())
    }

    fn complete_job(&self, job_id: &str, outcome: Option<serde_json::Value>) -> JonoResult<()> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        let outcome_data = outcome.unwrap_or(json!(null));
        let outcome_json =
            serde_json::to_string(&outcome_data).map_err(JonoError::Serialization)?;
        let metadata_key = keys.job_metadata_hash(job_id);

        let now = current_timestamp_ms();
        let _: () = redis::pipe()
            .zrem(keys.running_set(), job_id)
            .hset(&metadata_key, "completed_at", now.to_string())
            .hset(&metadata_key, "outcome", outcome_json)
            .query(&mut conn)
            .map_err(JonoError::Redis)?;

        Ok(())
    }

    fn get_connection(&self) -> JonoResult<Connection> {
        self.context.get_connection()
    }
}
