use std::time::Duration;

/// Configuration for the Harvester
pub struct HarvestConfig {
    /// How long to wait between polling for new jobs
    poll_interval: Duration,

    /// How long to wait for a job to become available each poll or "blocking"
    poll_timeout: Duration,

    /// Maximum number of consecutive errors before stopping
    max_consecutive_errors: usize,

    /// Maximum number of jobs to harvest in a single batch
    batch_size: usize,
}

impl Default for HarvestConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(100),
            poll_timeout: Duration::from_secs(5),
            max_consecutive_errors: 3,
            batch_size: 1,
        }
    }
}

impl HarvestConfig {
    pub fn new() -> HarvestConfig {
        HarvestConfig::default()
    }

    pub fn poll_interval(mut self, poll_interval: Duration) -> HarvestConfig {
        self.poll_interval = poll_interval;
        self
    }
    pub fn get_poll_interval(&self) -> Duration {
        self.poll_interval
    }

    pub fn poll_timeout(mut self, poll_timeout: Duration) -> HarvestConfig {
        self.poll_timeout = poll_timeout;
        self
    }
    pub fn get_poll_timeout(&self) -> Duration {
        self.poll_timeout
    }

    pub fn max_consecutive_errors(mut self, max_consecutive_errors: usize) -> HarvestConfig {
        self.max_consecutive_errors = max_consecutive_errors;
        self
    }
    pub fn get_max_consecutive_errors(&self) -> usize {
        self.max_consecutive_errors
    }

    pub fn batch_size(mut self, batch_size: usize) -> HarvestConfig {
        self.batch_size = batch_size;
        self
    }
    pub fn get_batch_size(&self) -> usize {
        self.batch_size
    }
}
