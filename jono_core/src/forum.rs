use crate::{JonoError, Result};
use redis::Client as RedisClient;

/// Central Redis connection pool that manages access to all topics
pub struct Forum {
    redis_client: RedisClient,
}

impl Forum {
    /// Create a forum with the specified Redis URL
    pub fn new(redis_url: &str) -> Result<Self> {
        let redis_client = RedisClient::open(redis_url)?;
        Ok(Self { redis_client })
    }

    /// Create a forum by trying to get the Redis URL from environment variables
    pub fn try_from_env() -> Result<Self> {
        let redis_url = crate::util::get_redis_url("");
        if redis_url.is_empty() {
            return Err(JonoError::MissingEnvVar("JONO_REDIS_URL")); // or "REDIS_URL"
        }
        Self::new(&redis_url)
    }

    /// Create a forum by trying to get the Redis URL from environment variables or using a fallback
    pub fn try_from_env_or(redis_url: impl ToString) -> Result<Self> {
        let redis_url = crate::util::get_redis_url(redis_url.to_string());
        if redis_url.is_empty() {
            return Err(JonoError::MissingEnvVar("JONO_REDIS_URL")); // or "REDIS_URL"
        }
        Self::new(&redis_url)
    }

    /// Create a context for a specific topic
    pub fn topic(&self, topic: impl ToString) -> crate::Context {
        crate::Context::new(self.clone(), topic)
    }

    /// Get a reference to the underlying Redis client
    pub(crate) fn redis_client(&self) -> &RedisClient {
        &self.redis_client
    }
}

impl Clone for Forum {
    fn clone(&self) -> Self {
        Self {
            redis_client: self.redis_client.clone(),
        }
    }
}
