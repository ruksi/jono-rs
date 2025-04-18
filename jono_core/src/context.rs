use crate::{Forum, Keys, Result};

/// Topic-specific context for Jono operations
#[derive(Clone)]
pub struct Context {
    forum: Forum,
    topic: String,
    keys: Keys,
}

impl Context {
    /// Create a new context with the given forum and topic
    pub fn new(forum: Forum, topic: impl ToString) -> Self {
        Self {
            forum,
            topic: topic.to_string(),
            keys: Keys::with_topic(topic),
        }
    }

    pub fn try_from_env(topic: impl ToString) -> Result<Self> {
        let forum = Forum::try_from_env()?;
        Ok(Self::new(forum, topic))
    }

    /// Get a Redis connection from the forum
    pub async fn get_connection(&self) -> Result<impl redis::aio::ConnectionLike> {
        Ok(self.forum.redis_pool().get().await?)
    }

    /// Get the topic name
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// Get the Redis keys for this topic
    pub fn keys(&self) -> &Keys {
        &self.keys
    }

    /// Get the forum for this context
    pub fn forum(&self) -> &Forum {
        &self.forum
    }

    /// Create a new context for a different topic using the same forum
    pub fn clone_for_topic(&self, topic: impl ToString) -> Self {
        Self::new(self.forum.clone(), topic)
    }
}
