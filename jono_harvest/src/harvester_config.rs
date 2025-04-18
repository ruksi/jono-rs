use std::time::Duration;

/// Configuration for the Harvester
pub struct HarvestConfig {
    /// How long to wait between polling for new jobs
    polling_interval: Duration,

    /// Maximum number of consecutive errors before stopping
    max_consecutive_errors: usize,

    /// Maximum number of jobs to harvest in a single batch
    batch_size: usize,
}

impl Default for HarvestConfig {
    fn default() -> Self {
        Self {
            polling_interval: Duration::from_millis(500),
            max_consecutive_errors: 3,
            batch_size: 1,
        }
    }
}

impl HarvestConfig {
    pub fn new() -> HarvestConfig {
        HarvestConfig::default()
    }

    pub fn polling_interval(mut self, polling_interval: Duration) -> HarvestConfig {
        self.polling_interval = polling_interval;
        self
    }
    pub fn get_polling_interval(&self) -> Duration {
        self.polling_interval
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
