//! `jono_produce` provides the interface for submitting jobs to Jono queues.
//!
//! This crate allows users to submit jobs to the queue, cancel jobs, and set job priorities.

use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::info;

use jono_core::*;

pub mod prelude {
    pub use crate::JobPlan;
    pub use crate::Producer;
}

/// Job plan represents **a description** of a task to be queued soon and, _hopefully_, completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPlan {
    /// ULID identifier for the job
    pub id: String,

    /// The job JSON payload
    pub payload: Value,

    /// Maximum number of attempts allowed for this job
    pub max_attempts: u32,

    /// Priority of the job; lower values are processed first
    pub priority: i64,

    /// When the job should be executed; UNIX timestamp in milliseconds or 0 to be executed as soon as possible
    pub scheduled_for: i64,
}

impl JobPlan {
    /// Create a new job plan with the given payload for immediate execution
    pub fn new<T: Serialize>(payload: T) -> JonoResult<Self> {
        let payload = serde_json::to_value(payload)?;
        Ok(Self {
            id: generate_job_id(),
            payload,
            max_attempts: 3,
            priority: 0,
            scheduled_for: 0,
        })
    }

    /// Set the maximum number of combined requeue and rerun attempts
    pub fn max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Set the priority; lower values are processed first
    pub fn priority(mut self, priority: i64) -> Self {
        self.priority = priority;
        self
    }

    /// Schedule the job for later execution
    pub fn schedule_for(mut self, timestamp_ms: i64) -> Self {
        self.scheduled_for = timestamp_ms;
        self
    }
}

/// Interface for submitting jobs to Jono queues
pub struct Producer {
    context: JonoContext,
}

impl Producer {
    /// Create a new Producer with the given topic context
    pub fn with_context(context: JonoContext) -> Self {
        Self { context }
    }

    /// Get a Redis connection
    fn get_connection(&self) -> JonoResult<Connection> {
        self.context.get_connection()
    }

    /// Send a new job to the queue
    pub fn dispatch(&self, job_plan: JobPlan) -> JonoResult<String> {
        let mut conn = self.get_connection()?;

        let keys = self.context.keys();
        let now = current_timestamp_ms();
        let metadata_key = keys.job_metadata_hash(&job_plan.id);

        let _: () = conn
            .hset(&metadata_key, "id", &job_plan.id)
            .map_err(JonoError::Redis)?;
        let _: () = conn
            .hset(
                &metadata_key,
                "payload",
                serde_json::to_string(&job_plan.payload)?,
            )
            .map_err(JonoError::Redis)?;
        let _: () = conn
            .hset(
                &metadata_key,
                "max_attempts",
                job_plan.max_attempts.to_string(),
            )
            .map_err(JonoError::Redis)?;
        let _: () = conn
            .hset(
                &metadata_key,
                "initial_priority",
                job_plan.priority.to_string(),
            )
            .map_err(JonoError::Redis)?;
        let _: () = conn
            .hset(&metadata_key, "created_at", now.to_string())
            .map_err(JonoError::Redis)?;
        let _: () = conn
            .hset(&metadata_key, "attempt_history", "[]")
            .map_err(JonoError::Redis)?;

        if job_plan.scheduled_for > 0 && job_plan.scheduled_for > now {
            let _: () = conn
                .zadd::<_, _, _, ()>(keys.scheduled_set(), &job_plan.id, job_plan.scheduled_for)
                .map_err(JonoError::Redis)?;

            info!(
                job_id = %job_plan.id,
                scheduled_for = %job_plan.scheduled_for,
                "Job scheduled for later execution"
            );
        } else {
            let _: () = conn
                .zadd::<_, _, _, ()>(keys.queued_set(), &job_plan.id, job_plan.priority)
                .map_err(JonoError::Redis)?;

            info!(
                job_id = %job_plan.id,
                priority = %job_plan.priority,
                "Job dispatched to queue"
            );
        }

        Ok(job_plan.id)
    }

    /// Cancel a job if it hasn't started processing yet
    pub fn cancel(&self, job_id: &str, grace_period_ms: i64) -> JonoResult<bool> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        let exists: bool = conn
            .exists(keys.job_metadata_hash(job_id))
            .map_err(JonoError::Redis)?;
        if !exists {
            return Err(JonoError::NotFound(format!("Job {} not found", job_id)));
        }

        let removed_from_queued: i32 = conn
            .zrem::<_, _, i32>(keys.queued_set(), job_id)
            .map_err(JonoError::Redis)?;
        let removed_from_scheduled: i32 = conn
            .zrem::<_, _, i32>(keys.scheduled_set(), job_id)
            .map_err(JonoError::Redis)?;

        let is_running: bool = conn
            .zscore::<_, _, Option<i64>>(keys.running_set(), job_id)
            .map_err(JonoError::Redis)?
            .is_some();

        // Get current time for both running and non-running cancellations
        let now = current_timestamp_ms();

        if is_running {
            let grace_end = now + grace_period_ms;

            let _: () = conn
                .zadd::<_, _, _, ()>(keys.cancelled_set(), job_id, grace_end)
                .map_err(JonoError::Redis)?;

            info!(
                job_id = %job_id,
                grace_end = %grace_end,
                "Running job marked for cancellation with grace period"
            );

            return Ok(true);
        }

        if removed_from_queued > 0 || removed_from_scheduled > 0 {
            let _: () = conn
                .zadd::<_, _, _, ()>(keys.cancelled_set(), job_id, now)
                .map_err(JonoError::Redis)?;
            info!(job_id = %job_id, "Job cancelled successfully");
            return Ok(true);
        }

        info!(job_id = %job_id, "Job not found in any active queue");
        Ok(false)
    }

    /// Get job metadata
    pub fn get_job_metadata(&self, job_id: &str) -> JonoResult<JobMetadata> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        let metadata_key = keys.job_metadata_hash(job_id);
        let exists: bool = conn.exists(&metadata_key).map_err(JonoError::Redis)?;
        if !exists {
            return Err(JonoError::NotFound(format!("Job {} not found", job_id)));
        }

        let metadata_map: HashMap<String, String> =
            conn.hgetall(metadata_key).map_err(JonoError::Redis)?;

        Ok(JobMetadata::from_hash(metadata_map)?)
    }

    /// Clean up job data from Redis
    pub fn clean_job(&self, job_id: &str) -> JonoResult<bool> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        let _: () = conn
            .zrem::<_, _, ()>(keys.queued_set(), job_id)
            .map_err(JonoError::Redis)?;
        let _: () = conn
            .zrem::<_, _, ()>(keys.running_set(), job_id)
            .map_err(JonoError::Redis)?;
        let _: () = conn
            .zrem::<_, _, ()>(keys.scheduled_set(), job_id)
            .map_err(JonoError::Redis)?;
        let _: () = conn
            .zrem::<_, _, ()>(keys.cancelled_set(), job_id)
            .map_err(JonoError::Redis)?;

        let job_key = keys.job_metadata_hash(job_id);
        let deleted: i32 = conn.del(job_key).map_err(JonoError::Redis)?;

        Ok(deleted > 0)
    }
}
