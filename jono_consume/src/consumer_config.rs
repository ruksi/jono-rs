use std::time::Duration;

/// Configuration options for a Consumer
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    /// How long to wait between polling for new jobs
    pub polling_interval: Duration,
    /// How often to update the heartbeat
    pub heartbeat_interval: Duration,
    /// How long before a job is considered abandoned
    pub heartbeat_timeout: Duration,
    /// Maximum number of consecutive errors before stopping
    pub max_consecutive_errors: usize,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        Self {
            polling_interval: Duration::from_millis(100),
            heartbeat_interval: Duration::from_secs(5),
            heartbeat_timeout: Duration::from_secs(10),
            max_consecutive_errors: 3,
        }
    }
}
