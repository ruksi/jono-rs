/// Return for Jono operations that can succeed (OK) or fail (Err)
pub type Result<T> = std::result::Result<T, Error>;

/// All the possible errors from Jono operations
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Job not found: {0}")]
    NotFound(String),
    #[error("Invalid job definition: {0}")]
    InvalidJob(String),
}
