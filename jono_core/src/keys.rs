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
