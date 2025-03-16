//! `jono_core` provides shared utilities for the Jono queue system.
//!
//! This crate includes common functionality used across the Jono components,
//! such as ULID generation, Redis key management, and error types.

mod context;
mod error;
mod forum;
mod job_metadata;
mod job_status;
mod job_status_reader;
mod keys;
mod util;

pub use context::JonoContext;
pub use error::{JonoError, JonoResult};
pub use forum::JonoForum;
pub use job_metadata::JobMetadata;
pub use job_status::JobStatus;
pub use job_status_reader::JobStatusReader;
pub use keys::JonoKeys;
pub use util::{current_timestamp_ms, generate_job_id, get_redis_url};

pub mod prelude {
    pub use crate::{
        JobMetadata, JobStatus, JobStatusReader, JonoContext, JonoError, JonoForum, JonoResult,
    };
}
