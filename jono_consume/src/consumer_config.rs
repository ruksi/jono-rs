use std::time::Duration;

/// Configuration options for a Consumer
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    /// How long to wait between polling for new jobs
    polling_interval: Duration,

    /// How often to update the heartbeat
    heartbeat_interval: Duration,

    /// How long before a job is considered abandoned
    heartbeat_timeout: Duration,

    /// Maximum number of consecutive errors before stopping
    max_consecutive_errors: usize,
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

impl ConsumerConfig {
    pub fn new() -> ConsumerConfig {
        ConsumerConfig::default()
    }

    pub fn polling_interval(mut self, polling_interval: Duration) -> ConsumerConfig {
        self.polling_interval = polling_interval;
        self
    }
    pub fn get_polling_interval(&self) -> Duration {
        self.polling_interval
    }

    pub fn heartbeat_interval(mut self, heartbeat_interval: Duration) -> ConsumerConfig {
        self.heartbeat_interval = heartbeat_interval;
        self
    }
    pub fn get_heartbeat_interval(&self) -> Duration {
        self.heartbeat_interval
    }

    pub fn heartbeat_timeout(mut self, heartbeat_timeout: Duration) -> ConsumerConfig {
        self.heartbeat_timeout = heartbeat_timeout;
        self
    }
    pub fn get_heartbeat_timeout(&self) -> Duration {
        self.heartbeat_timeout
    }

    pub fn max_consecutive_errors(mut self, max_consecutive_errors: usize) -> ConsumerConfig {
        self.max_consecutive_errors = max_consecutive_errors;
        self
    }
    pub fn get_max_consecutive_errors(&self) -> usize {
        self.max_consecutive_errors
    }
}
