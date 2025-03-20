use jono_core::{JobMetadata, JonoContext, JonoResult};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug)]
pub enum Outcome {
    Success(Option<Value>),
    Failure(String),
}

pub struct Workload {
    pub context: JonoContext,
    pub metadata: JobMetadata,
}

pub trait Worker: Send + Sync {
    fn handle_job(&self, workload: Workload) -> JonoResult<Outcome>;

    fn heartbeat_expiry(&self) -> Duration {
        Duration::from_secs(10)
    }
}
