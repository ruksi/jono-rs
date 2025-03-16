use crate::{JonoError, JonoResult};
use redis::Client as RedisClient;

/// Central Redis connection pool that manages access to all topics
pub struct JonoForum {
    redis_client: RedisClient,
}

impl JonoForum {
    /// Create a new forum with the specified Redis URL
    pub fn new(redis_url: &str) -> JonoResult<Self> {
        let redis_client = RedisClient::open(redis_url).map_err(JonoError::Redis)?;
        Ok(Self { redis_client })
    }

    /// Create a forum with the default Redis URL from environment
    pub fn default() -> JonoResult<Self> {
        let redis_url = crate::util::get_redis_url();
        Self::new(&redis_url)
    }

    /// Create a context for a specific topic
    pub fn topic(&self, topic: &str) -> crate::JonoContext {
        crate::JonoContext::new(self.clone(), topic)
    }

    /// Get a reference to the underlying Redis client
    pub(crate) fn redis_client(&self) -> &RedisClient {
        &self.redis_client
    }
}

impl Clone for JonoForum {
    fn clone(&self) -> Self {
        Self {
            redis_client: self.redis_client.clone(),
        }
    }
}
