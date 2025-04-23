use jono_consume::{WorkSummary, Worker, Workload};
use serde_json::json;

pub struct NoopWorker;

impl Worker for NoopWorker {
    async fn work(&self, _: &Workload) -> jono_core::Result<WorkSummary> {
        Ok(WorkSummary::Success(Some(json!({"processed": true}))))
    }
}
