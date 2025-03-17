//! Provides functionality to query details jobs in the Jono queue system.

use redis::{Commands, Connection};
use serde_json::Value;

use crate::{JobStatus, JonoContext, JonoError, JonoResult};

/// Interface for querying job details
pub struct Inspector {
    context: JonoContext,
}

impl Inspector {
    /// Create a new job inspector in the given topic context
    pub fn with_context(context: JonoContext) -> Self {
        Self { context }
    }

    /// Return a Redis connection
    fn get_connection(&self) -> JonoResult<Connection> {
        self.context.get_connection()
    }

    /// Return if a job exists
    pub fn job_exists(&self, job_id: &str) -> JonoResult<bool> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        let exists: bool = conn
            .exists(keys.job_metadata_hash(job_id))
            .map_err(JonoError::Redis)?;

        Ok(exists)
    }

    /// Get the current status of a job
    pub fn get_job_status(&self, job_id: &str) -> JonoResult<JobStatus> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        let exists: bool = conn
            .exists(keys.job_metadata_hash(job_id))
            .map_err(JonoError::Redis)?;
        if !exists {
            return Err(JonoError::NotFound(format!("Job {} not found", job_id)));
        }

        let in_running_set: bool = conn
            .zscore::<_, _, Option<i64>>(keys.running_set(), job_id)
            .map_err(JonoError::Redis)?
            .is_some();
        if in_running_set {
            return Ok(JobStatus::Running);
        }

        let in_queued_set: bool = conn
            .zscore::<_, _, Option<i64>>(keys.queued_set(), job_id)
            .map_err(JonoError::Redis)?
            .is_some();
        if in_queued_set {
            return Ok(JobStatus::Queued);
        }

        let in_scheduled_set: bool = conn
            .zscore::<_, _, Option<i64>>(keys.scheduled_set(), job_id)
            .map_err(JonoError::Redis)?
            .is_some();
        if in_scheduled_set {
            return Ok(JobStatus::Scheduled);
        }

        let in_canceled_set: bool = conn
            .zscore::<_, _, Option<i64>>(keys.cancelled_set(), job_id)
            .map_err(JonoError::Redis)?
            .is_some();
        if in_canceled_set {
            return Ok(JobStatus::Canceled);
        }

        let has_completed_at_field: Option<String> = conn
            .hget(keys.job_metadata_hash(job_id), "completed_at")
            .map_err(JonoError::Redis)?;
        if has_completed_at_field.is_some() {
            return Ok(JobStatus::Completed);
        }

        let attempt_history: Option<String> = conn
            .hget(keys.job_metadata_hash(job_id), "attempt_history")
            .map_err(JonoError::Redis)?;
        if let Some(history_str) = attempt_history {
            let history: Value =
                serde_json::from_str(&history_str).map_err(JonoError::Serialization)?;

            if let Value::Array(attempts) = history {
                let max_attempts: u32 = conn
                    .hget::<_, _, Option<String>>(keys.job_metadata_hash(job_id), "max_attempts")
                    .map_err(JonoError::Redis)?
                    .and_then(|m| m.parse().ok())
                    .unwrap_or(3);

                if attempts.len() >= max_attempts as usize {
                    return Ok(JobStatus::Failed);
                }
            }
        }

        Ok(JobStatus::Failed)
    }
}
