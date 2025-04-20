use crate::error::{JonoError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata for a job in the Jono system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    /// Unique identifier for the job
    pub id: String,

    /// The job JSON payload
    pub payload: serde_json::Value,

    /// The maximum number of attempts allowed
    pub max_attempts: u32,

    /// Priority; lower values are processed first
    pub initial_priority: i64,

    /// Current number of attempts made
    pub attempt_count: u32,

    // Keeping track what made attempts fail
    pub attempt_history: Vec<serde_json::Value>,

    // Result on job completion from the worker
    pub work_summary: Option<serde_json::Value>,

    // Who submitted the job; custom or hostname
    pub origin: String,
}

impl JobMetadata {
    /// Convert a Redis hash into more structured job metadata
    pub fn from_hash(hash: HashMap<String, String>) -> Result<Self> {
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

        let work_summary_str = hash
            .get("work_summary")
            .ok_or_else(|| JonoError::InvalidJob("Missing work_summary field".to_string()))?;
        let work_summary = serde_json::from_str(work_summary_str)
            .map_err(|_| JonoError::InvalidJob("Invalid work_summary JSON".to_string()))?;

        let origin = hash
            .get("origin")
            .ok_or_else(|| JonoError::InvalidJob("Missing origin field".to_string()))?
            .clone();

        Ok(Self {
            id,
            payload,
            max_attempts,
            attempt_count,
            initial_priority,
            attempt_history: vec![],
            work_summary,
            origin,
        })
    }
}
