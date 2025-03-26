#![cfg(all(feature = "produce", feature = "consume"))]

mod common;

use common::create_test_context;
use jono::prelude::*;
use serde_json::json;

struct NoopWorker;

impl Worker for NoopWorker {
    fn process(&self, _: &Workload) -> Result<Outcome> {
        Ok(Outcome::Success(Some(json!({"processed": true}))))
    }
}

#[test]
fn test_basics() -> Result<()> {
    let context = create_test_context("test_collect");
    let inspector = Inspector::with_context(context.clone());
    let producer = Producer::with_context(context.clone());

    let job_id = JobPlan::new()
        .payload(json!({"action": "test_action"}))
        .dispatch(&producer)?;
    assert_eq!(inspector.get_job_status(&job_id)?, JobStatus::Queued);

    let consumer = Consumer::with_context(context.clone(), NoopWorker);
    let outcome = consumer.run_next()?;
    assert!(matches!(outcome, Some(Outcome::Success(_))));

    // it shouldn't be expired, yet
    let expired = inspector.clean_completed_jobs()?;
    assert_eq!(expired, 0);

    // you can pop the completed jobs
    let completed = inspector.acquire_completed_jobs(3)?;
    assert_eq!(completed.len(), 1);
    let c_metadata = completed.first().unwrap().clone();
    assert_eq!(c_metadata.payload, json!({"action": "test_action"}));
    assert_eq!(c_metadata.outcome.unwrap(), json!({"processed": true}));

    // the single completion was processed already
    let completed = inspector.acquire_completed_jobs(1)?;
    assert_eq!(completed.len(), 0);

    producer.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_nonexistent() -> Result<()> {
    let context = create_test_context("test_nothing_to_collect");
    let inspector = Inspector::with_context(context.clone());
    assert_eq!(inspector.acquire_completed_jobs(0)?.len(), 0);
    assert_eq!(inspector.acquire_completed_jobs(1)?.len(), 0);
    assert_eq!(inspector.acquire_completed_jobs(2)?.len(), 0);
    Ok(())
}
