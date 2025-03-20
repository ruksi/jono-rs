//! `jono_produce` provides the interface for submitting jobs to Jono queues.
//!
//! This crate allows users to submit jobs to the queue, cancel jobs, and set job priorities.

mod job_plan;
mod worker;

pub use job_plan::JobPlan;
pub use worker::Producer;

pub mod prelude {
    pub use crate::{JobPlan, Producer};
}
