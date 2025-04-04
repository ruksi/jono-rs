/// Return for Jono operations that can succeed (OK) or fail (Err)
pub type Result<T> = std::result::Result<T, JonoError>;

/// All the possible errors from Jono operations
#[derive(Debug)]
pub enum JonoError {
    Redis(redis::RedisError),
    Serialization(serde_json::Error),
    JobNotFound(String),             // job id
    InvalidJob(String),              // message with details what is invalid
    TooManyConsecutiveErrors(usize), // the number of errors
}

impl From<redis::RedisError> for JonoError {
    fn from(err: redis::RedisError) -> Self {
        JonoError::Redis(err)
    }
}

impl From<serde_json::Error> for JonoError {
    fn from(err: serde_json::Error) -> Self {
        JonoError::Serialization(err)
    }
}

impl std::fmt::Display for JonoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JonoError::Redis(err) => write!(f, "Redis error: {}", err),
            JonoError::Serialization(err) => write!(f, "Serialization error: {}", err),
            JonoError::JobNotFound(job_id) => write!(f, "Job not found: {}", job_id),
            JonoError::InvalidJob(msg) => write!(f, "Invalid job: {}", msg),
            JonoError::TooManyConsecutiveErrors(count) => {
                write!(f, "Too many consecutive errors: {}", count)
            }
        }
    }
}

impl std::error::Error for JonoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            JonoError::Redis(err) => Some(err),
            JonoError::Serialization(err) => Some(err),
            JonoError::JobNotFound(_) => None,
            JonoError::InvalidJob(_) => None,
            JonoError::TooManyConsecutiveErrors(_) => None,
        }
    }
}
