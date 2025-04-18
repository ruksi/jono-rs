use jono_core::{JobMetadata, Result};
use serde_json::Value;
use std::future::Future;

pub trait Reaper: Send + Sync {
    fn reap<'a>(
        &'a self,
        load: &'a Reapload,
    ) -> impl Future<Output = Result<ReapSummary>> + Send + 'a;
}

pub struct Reapload {
    pub job_id: String,
    pub payload: Value,
    pub work_summary: Value,
}

impl Reapload {
    pub fn from_metadata(metadata: JobMetadata) -> Self {
        let work_summary = metadata.work_summary.unwrap_or(Value::Null);
        Self {
            job_id: metadata.id,
            payload: metadata.payload,
            work_summary,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReapSummary {
    Success(Option<Value>),
    Failure(String),
}
