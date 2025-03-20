use crate::{Error, Forum, Keys, Result};
use redis::Connection;

/// Topic-specific context for Jono operations
#[derive(Clone)]
pub struct Context {
    forum: Forum,
    topic: String,
    keys: Keys,
}

impl Context {
    /// Create a new context with the given forum and topic
    pub fn new(forum: Forum, topic: &str) -> Self {
        Self {
            forum,
            topic: topic.to_string(),
            keys: Keys::with_topic(topic),
        }
    }

    /// Get a Redis connection from the forum
    pub fn get_connection(&self) -> Result<Connection> {
        self.forum
            .redis_client()
            .get_connection()
            .map_err(Error::Redis)
    }

    /// Get the topic name
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// Get the Redis keys for this topic
    pub fn keys(&self) -> &Keys {
        &self.keys
    }

    /// Create a new context for a different topic using the same forum
    pub fn for_topic(&self, topic: &str) -> Self {
        Self::new(self.forum.clone(), topic)
    }
}
