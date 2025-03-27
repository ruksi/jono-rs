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
    let context = create_test_context("test_harvest");
    let inspector = Inspector::with_context(context.clone());
    let producer = Producer::with_context(context.clone());

    assert_eq!(inspector.acquire_harvestable_jobs(1)?.len(), 0);

    let job_id = JobPlan::new()
        .payload(json!({"action": "test_action"}))
        .dispatch(&producer)?;
    assert_eq!(inspector.get_job_status(&job_id)?, JobStatus::Queued);

    assert_eq!(inspector.acquire_harvestable_jobs(1)?.len(), 0);

    let consumer = Consumer::with_context(context.clone(), NoopWorker);
    let outcome = consumer.run_next()?;
    assert!(matches!(outcome, Some(Outcome::Success(_))));

    // it shouldn't be expired, yet
    let expired = inspector.clean_harvestable_jobs()?;
    assert_eq!(expired, 0);

    // you can pop the harvestables
    let harvest = inspector.acquire_harvestable_jobs(3)?;
    assert_eq!(harvest.len(), 1);
    let c_metadata = harvest.first().unwrap().clone();
    assert_eq!(c_metadata.payload, json!({"action": "test_action"}));
    assert_eq!(c_metadata.outcome.unwrap(), json!({"processed": true}));

    // the single harvestable was processed already
    let harvest = inspector.acquire_harvestable_jobs(1)?;
    assert_eq!(harvest.len(), 0);

    producer.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_nonexistent() -> Result<()> {
    let context = create_test_context("test_nothing_to_harvest");
    let inspector = Inspector::with_context(context.clone());
    assert_eq!(inspector.acquire_harvestable_jobs(0)?.len(), 0);
    assert_eq!(inspector.acquire_harvestable_jobs(1)?.len(), 0);
    assert_eq!(inspector.acquire_harvestable_jobs(2)?.len(), 0);
    Ok(())
}
