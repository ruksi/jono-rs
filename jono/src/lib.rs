//! Jono is a priority job queue implemented with ULIDs and Redis sorted sets.
//!
//! This is an _umbrella crate_ for re-exporting different components of
//! the priority job queue system from a single place according to the
//! features enabled like `dispatch` for the `jono_dispatch` crate to allow
//! submitting jobs.
//!
//! Shared utilities from `core` will always be available, regardless of
//! the features enabled.
//!
//! Read documentation for each sub-crate for more information.

pub use jono_core as core;

#[cfg(feature = "dispatch")]
pub use jono_dispatch as dispatch;

pub mod prelude {
    pub use crate::core::prelude::*;

    #[cfg(feature = "dispatch")]
    pub use crate::dispatch::prelude::*;
}
