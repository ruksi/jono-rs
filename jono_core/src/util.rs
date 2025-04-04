use std::time::{SystemTime, UNIX_EPOCH};
use ulid::Ulid;

/// Generate a new ULID for a job
pub fn generate_job_id() -> String {
    Ulid::new().to_string()
}

/// Get current timestamp in milliseconds since UNIX epoch
pub fn current_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}

/// A standardized way to read JONO_REDIS_URL env var with REDIS_URL as fallback
pub(crate) fn get_redis_url(fallback: impl ToString) -> String {
    std::env::var("JONO_REDIS_URL")
        .or_else(|_| std::env::var("REDIS_URL"))
        .unwrap_or_else(|_| fallback.to_string())
}
