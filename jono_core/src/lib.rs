//! `jono_core` provides shared utilities for the Jono queue system.
//!
//! This crate includes common functionality used across the Jono components,
//! such as ULID generation, Redis key management, and error types.

mod context;
mod error;
mod forum;
mod inspector;
mod job_metadata;
mod job_status;
mod keys;
mod util;

pub use context::Context;
pub use error::{JonoError, Result};
pub use forum::Forum;
pub use inspector::Inspector;
pub use job_metadata::JobMetadata;
pub use job_status::JobStatus;
pub use keys::Keys;
pub use util::{current_timestamp_ms, generate_job_id, get_redis_url};

pub mod prelude {
    pub use crate::{Context, Forum, Inspector, JobMetadata, JobStatus, JonoError};
}
