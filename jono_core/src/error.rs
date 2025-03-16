use redis::RedisError;
use thiserror::Error;

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
