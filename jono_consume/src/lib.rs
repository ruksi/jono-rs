//! Jono consumer components for processing jobs from a queue
//!
//! The Consumer is responsible for fetching jobs from the queue and processing them.

mod consumer;
mod worker;

pub use consumer::Consumer;
pub use worker::{Outcome, Worker, Workload};

pub mod prelude {
    pub use crate::{Consumer, Outcome, Worker, Workload};
}
