use crate::{JonoError, JonoForum, JonoKeys, JonoResult};
use redis::Connection;

/// Topic-specific context for Jono operations
pub struct JonoContext {
    forum: JonoForum,
    topic: String,
    keys: JonoKeys,
}

impl JonoContext {
    /// Create a new context with the given forum and topic
    pub fn new(forum: JonoForum, topic: &str) -> Self {
        Self {
            forum,
            topic: topic.to_string(),
            keys: JonoKeys::with_topic(topic),
        }
    }

    /// Get a Redis connection from the forum
    pub fn get_connection(&self) -> JonoResult<Connection> {
        self.forum
            .redis_client()
            .get_connection()
            .map_err(JonoError::Redis)
    }

    /// Get the topic name
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// Get the Redis keys for this topic
    pub fn keys(&self) -> &JonoKeys {
        &self.keys
    }

    /// Create a new context for a different topic using the same forum
    pub fn for_topic(&self, topic: &str) -> Self {
        Self::new(self.forum.clone(), topic)
    }
}

impl Clone for JonoContext {
    fn clone(&self) -> Self {
        Self {
            forum: self.forum.clone(),
            topic: self.topic.clone(),
            keys: self.keys.clone(),
        }
    }
}
