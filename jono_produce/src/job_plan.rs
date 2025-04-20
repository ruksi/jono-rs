use jono_core::{JonoError, Result};
use serde::{Deserialize, Serialize};

/// Job plan represents **a description** of a task to be queued soon and, _hopefully_, completed.
/// Yes, it's a builder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPlan {
    /// The job JSON payload
    payload: Option<serde_json::Value>,

    /// Maximum number of attempts allowed for this job
    max_attempts: u32,

    /// Priority of the job; lower values are processed first
    priority: i64,

    /// When the job should be executed; UNIX timestamp in milliseconds or 0 to be executed as soon as possible
    postponed_to: i64,

    /// Who submitted the job; custom or hostname
    origin: Option<String>,
}

impl JobPlan {
    pub fn new() -> JobPlan {
        JobPlan {
            payload: None,
            max_attempts: 1,
            priority: 0,
            postponed_to: 0,
            origin: None,
        }
    }

    pub fn payload(mut self, payload: serde_json::Value) -> JobPlan {
        self.payload = Some(payload);
        self
    }
    pub fn get_payload(&self) -> Option<&serde_json::Value> {
        self.payload.as_ref()
    }

    pub fn max_attempts(mut self, max_attempts: u32) -> JobPlan {
        self.max_attempts = max_attempts;
        self
    }
    pub fn get_max_attempts(&self) -> u32 {
        self.max_attempts
    }

    pub fn priority(mut self, priority: i64) -> JobPlan {
        self.priority = priority;
        self
    }
    pub fn get_priority(&self) -> i64 {
        self.priority
    }

    pub fn postponed_to(mut self, postponed_to: i64) -> JobPlan {
        self.postponed_to = postponed_to;
        self
    }
    pub fn get_postponed_to(&self) -> i64 {
        self.postponed_to
    }

    pub fn origin(mut self, origin: impl ToString) -> JobPlan {
        self.origin = Some(origin.to_string());
        self
    }
    pub fn get_origin(&self) -> Option<&str> {
        self.origin.as_deref()
    }

    pub async fn submit(self, producer: &crate::Producer) -> Result<String> {
        if self.payload.is_none() {
            return Err(JonoError::InvalidJob("Job payload is required".to_string()));
        }
        producer.submit_job(self).await
    }
}
