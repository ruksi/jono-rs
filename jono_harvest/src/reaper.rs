use jono_core::{JobMetadata, Result};
use serde_json::Value;
use std::future::Future;

pub trait Reaper: Send + Sync {
    fn process<'a>(&'a self, load: &'a Reapload)
    -> impl Future<Output = Result<Yield>> + Send + 'a;
}

pub struct Reapload {
    pub job_id: String,
    pub payload: Value,
    pub outcome: Value,
}

impl Reapload {
    pub fn from_metadata(metadata: JobMetadata) -> Self {
        let outcome = metadata.outcome.unwrap_or(Value::Null);
        Self {
            job_id: metadata.id,
            payload: metadata.payload,
            outcome,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Yield {
    Success(Option<Value>),
    Failure(String),
}
