use jono_core::{Context, JobMetadata, Result};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug)]
pub enum Outcome {
    Success(Option<Value>),
    Failure(String),
}

pub struct Workload {
    pub context: Context,
    pub metadata: JobMetadata,
}

pub trait Worker: Send + Sync {
    fn handle_job(&self, workload: Workload) -> Result<Outcome>;

    fn heartbeat_expiry(&self) -> Duration {
        Duration::from_secs(10)
    }
}
