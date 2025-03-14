//! `jono_core` provides shared utilities for the Jono queue system.
//!
//! This crate includes common functionality used across the Jono components,
//! such as ULID generation, Redis key management, and error types.

use redis::RedisError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use ulid::Ulid;

/// A standardized way to read JONO_REDIS_URL env var
pub fn get_redis_url() -> String {
    std::env::var("JONO_REDIS_URL").unwrap_or_else(|_| "redis://localhost:6380".to_string())
}

/// Return for Jono operations that can succeed (OK) or fail (Err)
pub type JonoResult<T> = Result<T, JonoError>;

/// All the possible errors from Jono operations
#[derive(Error, Debug)]
pub enum JonoError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Job not found: {0}")]
    NotFound(String),
    #[error("Invalid job definition: {0}")]
    InvalidJob(String),
}

/// All the possible states that a job can be in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// The job is queued and waiting to be processed.
    Queued,
    /// The job is currently being processed by a worker.
    Running,
    /// The job has been completed successfully.
    Completed,
    /// The job has failed to complete after specified retries.
    Failed,
    /// The job has been specifically canceled.
    Cancelled,
    /// The job is scheduled to run at a future time.
    Scheduled,
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            JobStatus::Queued => "queued".to_string(),
            JobStatus::Running => "running".to_string(),
            JobStatus::Completed => "completed".to_string(),
            JobStatus::Failed => "failed".to_string(),
            JobStatus::Cancelled => "cancelled".to_string(),
            JobStatus::Scheduled => "scheduled".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl FromStr for JobStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(JobStatus::Queued),
            "running" => Ok(JobStatus::Running),
            "completed" => Ok(JobStatus::Completed),
            "failed" => Ok(JobStatus::Failed),
            "cancelled" => Ok(JobStatus::Cancelled),
            "scheduled" => Ok(JobStatus::Scheduled),
            _ => Err(format!("Unknown job status: {}", s)),
        }
    }
}

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

/// Redis key generator for Jono components
pub struct JonoKeys {
    prefix: String,
    topic: String,
}

impl JonoKeys {
    /// Create a new Redis key generator with the default "jono" prefix and over given topic
    pub fn with_topic(topic: &str) -> Self {
        Self {
            prefix: "jono".to_string(),
            topic: topic.to_string(),
        }
    }

    /// Redis key for the sorted set that holds queued jobs
    pub fn queued_set(&self) -> String {
        format!("{}:{}:queued", self.prefix, self.topic)
    }

    /// Redis key for the sorted set that holds running jobs
    pub fn running_set(&self) -> String {
        format!("{}:{}:running", self.prefix, self.topic)
    }

    /// Redis key for the sorted set that communicates which jobs have been canceled
    pub fn cancelled_set(&self) -> String {
        format!("{}:{}:cancelled", self.prefix, self.topic)
    }

    /// Redis key for the sorted set that holds the jobs scheduled to run later
    pub fn scheduled_set(&self) -> String {
        format!("{}:{}:scheduled", self.prefix, self.topic)
    }

    /// Redis key for the hash that holds job metadata
    pub fn job_metadata_hash(&self, job_id: &str) -> String {
        format!("{}:{}:job:{}", self.prefix, self.topic, job_id)
    }
}

/// Get current timestamp in milliseconds since UNIX epoch
pub fn current_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}

/// Generate a new ULID for a job
pub fn generate_job_id() -> String {
    Ulid::new().to_string()
}
