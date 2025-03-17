//! `jono_produce` provides the interface for submitting jobs to Jono queues.
//!
//! This crate allows users to submit jobs to the queue, cancel jobs, and set job priorities.

mod job_plan;
mod producer;

pub use job_plan::JobPlan;
pub use producer::Producer;

pub mod prelude {
    pub use crate::JobPlan;
    pub use crate::Producer;
}
