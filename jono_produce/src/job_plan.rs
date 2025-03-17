use jono_core::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
