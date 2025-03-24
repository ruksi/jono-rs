//! Jono is a priority queue implemented with ULIDs and Redis sorted sets.
//!
//! This is an _umbrella crate_ for re-exporting different components of
//! the priority job queue system from a single place according to the
//! features enabled like `produce` for the `jono_produce` crate to allow
//! submitting jobs.
//!
//! Shared utilities from `core` will always be available, regardless of
//! the features enabled.
//!
//! Read documentation for each sub-crate for more information.

pub use jono_core as core;

#[cfg(feature = "produce")]
pub use jono_produce as produce;

#[cfg(feature = "consume")]
pub use jono_consume as consume;

pub mod prelude {
    pub use crate::core::prelude::*;

    #[cfg(feature = "produce")]
    pub use crate::produce::prelude::*;

    #[cfg(feature = "consume")]
    pub use crate::consume::prelude::*;
}
