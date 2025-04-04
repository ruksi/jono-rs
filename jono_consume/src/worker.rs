use jono_core::{JobMetadata, Result};
use serde_json::Value;
use std::future::Future;

pub trait Worker: Send + Sync {
    fn process<'a>(&'a self, job: &'a Workload) -> impl Future<Output = Result<Outcome>> + Send + 'a;
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
