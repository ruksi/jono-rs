use jono_core::{JobMetadata, Result};
use serde_json::Value;

pub trait Worker: Send + Sync {
    fn process(&self, job: &Workload) -> Result<Outcome>;
}

pub struct Workload {
    pub job_id: String,
    pub payload: Value,
}

impl Workload {
    pub fn from_metadata(metadata: JobMetadata) -> Self {
        Self {
            job_id: metadata.id,
            payload: metadata.payload,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Success(Option<Value>),
    Failure(String),
}
