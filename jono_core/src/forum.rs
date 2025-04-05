use crate::{JonoError, Result};
use deadpool_redis::{Config, Pool, Runtime};

/// Central Redis connection pool that manages access to all topics
pub struct Forum {
    redis_pool: Pool,
    redis_url: String,
}

impl Forum {
    /// Create a forum with the specified Redis URL
    pub fn new(redis_url: &str) -> Result<Self> {
        #[cfg(feature = "runtime-async-std")]
        let runtime = Some(Runtime::AsyncStd1);

        #[cfg(feature = "runtime-tokio")]
        let runtime = Some(Runtime::Tokio1);

        #[cfg(not(any(feature = "runtime-tokio", feature = "runtime-async-std")))]
        compile_error!(
            "Please enable either the 'runtime-tokio' or 'runtime-async-std' feature in your Cargo.toml"
        );

        #[cfg(all(feature = "runtime-async-std", feature = "runtime-tokio"))]
        compile_error!(
            "Please enable only one of the 'runtime-tokio' or 'runtime-async-std' features in your Cargo.toml"
        );

        let pool = Config::from_url(redis_url).create_pool(runtime)?;

        Ok(Self {
            redis_pool: pool,
            redis_url: redis_url.to_string(),
        })
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

    /// Get a reference to the underlying Redis pool
    pub(crate) fn redis_pool(&self) -> &Pool {
        &self.redis_pool
    }

    /// Get the Redis URL used by this forum
    pub fn redis_url(&self) -> &str {
        &self.redis_url
    }
}

impl Clone for Forum {
    fn clone(&self) -> Self {
        Self {
            redis_pool: self.redis_pool.clone(),
            redis_url: self.redis_url.clone(),
        }
    }
}
