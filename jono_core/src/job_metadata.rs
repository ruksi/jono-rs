use crate::error::{JonoError, JonoResult};
use crate::job_status::JobStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata for a job in the Jono system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    /// Unique identifier for the job
    pub id: String,

    /// The current status of the job
    pub status: JobStatus,

    /// The job JSON payload
    pub payload: serde_json::Value,

    /// Maximum number of attempts allowed
    pub max_attempts: u32,

    /// Current number of attempts made
    pub attempt_count: u32,

    /// Priority (lower values are processed first)
    pub initial_priority: i64,
}

impl JobMetadata {
    /// Convert a Redis hash into more structured job metadata
    pub fn from_hash(hash: HashMap<String, String>, status: JobStatus) -> JonoResult<Self> {
        let id = hash
            .get("id")
            .ok_or_else(|| JonoError::InvalidJob("Missing id field".to_string()))?
            .clone();

        let payload_str = hash
            .get("payload")
            .ok_or_else(|| JonoError::InvalidJob("Missing payload field".to_string()))?;

        let payload = serde_json::from_str(payload_str)
            .map_err(|_| JonoError::InvalidJob("Invalid payload JSON".to_string()))?;

        let max_attempts = hash
            .get("max_attempts")
            .ok_or_else(|| JonoError::InvalidJob("Missing max_attempts field".to_string()))?
            .parse::<u32>()
            .map_err(|_| JonoError::InvalidJob("Invalid max_attempts".to_string()))?;

        let attempt_count = hash
            .get("attempt_count")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        let initial_priority = hash
            .get("initial_priority")
            .ok_or_else(|| JonoError::InvalidJob("Missing initial_priority field".to_string()))?
            .parse::<i64>()
            .map_err(|_| JonoError::InvalidJob("Invalid initial_priority".to_string()))?;

        Ok(Self {
            id,
            status,
            payload,
            max_attempts,
            attempt_count,
            initial_priority,
        })
    }
}
