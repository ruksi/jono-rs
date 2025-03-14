//! Jono is a priority job queue implemented with ULIDs and Redis sorted sets.
//!
//! This crate provides a priority job queue system that ensures jobs are
//! processed in priority order or FIFO when priorities are equal.

#[cfg(feature = "core")]
pub use jono_core as core;

#[cfg(feature = "dispatch")]
pub use jono_dispatch as dispatch;

pub mod prelude {
    //! The Jono prelude.
    //!
    //! This module provides the most commonly used types and functions.
    
    #[cfg(feature = "core")]
    pub use crate::core::{
        generate_job_id, current_timestamp_ms, timestamp_to_datetime,
        JonoError, JonoResult, JonoKeys, JobStatus,
    };
    
    #[cfg(feature = "dispatch")]
    pub use crate::dispatch::{Dispatcher, Job};
}
