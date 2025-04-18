/// Return for Jono operations that can succeed (OK) or fail (Err)
pub type Result<T> = std::result::Result<T, JonoError>;

/// All the possible errors from Jono operations
#[derive(Debug)]
pub enum JonoError {
    CreatePool(deadpool_redis::CreatePoolError),
    RedisPool(deadpool_redis::PoolError),
    Redis(redis::RedisError),
    Serialization(serde_json::Error),
    JobNotFound(String),  // job id
    InvalidJob(String),   // message with details what is invalid
    TooManyErrors(usize), // the number of errors
    MissingEnvVar(&'static str),
}

impl From<deadpool_redis::CreatePoolError> for JonoError {
    fn from(err: deadpool_redis::CreatePoolError) -> Self {
        JonoError::CreatePool(err)
    }
}

impl From<deadpool_redis::PoolError> for JonoError {
    fn from(err: deadpool_redis::PoolError) -> Self {
        JonoError::RedisPool(err)
    }
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
        use JonoError::*;
        match self {
            CreatePool(err) => write!(f, "Redis pool create error: {}", err),
            RedisPool(err) => write!(f, "Redis pool error: {}", err),
            Redis(err) => write!(f, "Redis error: {}", err),
            Serialization(err) => write!(f, "Serialization error: {}", err),
            JobNotFound(job_id) => write!(f, "Job not found: {}", job_id),
            InvalidJob(msg) => write!(f, "Invalid job: {}", msg),
            TooManyErrors(count) => write!(f, "Too many consecutive errors: {}", count),
            MissingEnvVar(var) => write!(f, "Missing environment variable: {}", var),
        }
    }
}

impl std::error::Error for JonoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use JonoError::*;
        match self {
            CreatePool(err) => Some(err),
            RedisPool(err) => Some(err),
            Redis(err) => Some(err),
            Serialization(err) => Some(err),
            JobNotFound(_) => None,
            InvalidJob(_) => None,
            TooManyErrors(_) => None,
            MissingEnvVar(_) => None,
        }
    }
}
