/// Redis key generator for Jono components
#[derive(Clone)]
pub struct Keys {
    prefix: String,
    topic: String,
}

impl Keys {
    /// Create a new Redis key generator with the default "jono" prefix and over given topic
    pub fn with_topic(topic: impl ToString) -> Self {
        Self {
            prefix: "jono".to_string(),
            topic: topic.to_string(),
        }
    }

    /// Redis key for the hash that holds job metadata
    pub fn job_metadata_hash(&self, job_id: &str) -> String {
        format!("{}:{{{}}}:job:{}", self.prefix, self.topic, job_id)
    }

    /// Redis key for the sorted set that holds the postponed jobs with to-be-executed timestamps as scores
    pub fn postponed_set(&self) -> String {
        format!("{}:{{{}}}:postponed", self.prefix, self.topic)
    }

    /// Redis key for the sorted set that holds queued jobs with priority as scores
    pub fn queued_set(&self) -> String {
        format!("{}:{{{}}}:queued", self.prefix, self.topic)
    }

    /// Redis key for the sorted set that holds running jobs with heartbeat timestamps as scores
    pub fn started_set(&self) -> String {
        format!("{}:{{{}}}:started", self.prefix, self.topic)
    }

    /// Redis key for the sorted set that communicates which jobs have been canceled with grace period timestamps as scores
    pub fn aborted_set(&self) -> String {
        format!("{}:{{{}}}:aborted", self.prefix, self.topic)
    }

    /// Redis key for the sorted set that holds harvestable jobs (not post-processed but completed)
    /// with expiry timestamps as scores
    pub fn harvestable_set(&self) -> String {
        format!("{}:{{{}}}:harvestable", self.prefix, self.topic)
    }
}
