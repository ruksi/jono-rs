#![cfg(all(feature = "produce", feature = "consume"))]

mod common;

use common::create_test_context;
use jono::prelude::*;
use jono_core::Result;
use serde_json::json;

struct NoopWorker;

impl Worker for NoopWorker {
    async fn process(&self, _: &Workload) -> Result<Outcome> {
        Ok(Outcome::Success(Some(json!({"processed": true}))))
    }
}

#[tokio::test]
async fn test_basics() -> Result<()> {
    let context = create_test_context("test_consume");
    let inspector = Inspector::with_context(context.clone());
    let producer = Producer::with_context(context.clone());
    let job_id = JobPlan::new()
        .payload(json!({"action": "test_action"}))
        .submit(&producer)?;
    assert_eq!(inspector.get_job_status(&job_id)?, JobStatus::Queued);

    let consumer = Consumer::with_context(context.clone(), NoopWorker);
    let outcome = consumer.run_next().await?;
    let Some(Outcome::Success(_)) = outcome else {
        panic!("Expected job to succeed but got {:?}", outcome);
    };

    let metadata = inspector.get_job_metadata(&job_id)?;
    assert_eq!(metadata.outcome.unwrap(), json!({"processed": true}));
    assert_eq!(inspector.get_job_status(&job_id)?, JobStatus::Harvestable);

    producer.clean_job(&job_id)?;
    Ok(())
}

#[tokio::test]
async fn test_with_config() -> Result<()> {
    use std::time::Duration;

    let context = create_test_context("test_config");
    let producer = Producer::with_context(context.clone());
    let job_id = JobPlan::new()
        .payload(json!({"action": "configured_action"}))
        .submit(&producer)?;

    let consumer = Consumer::with_context(context.clone(), NoopWorker).with_config(
        ConsumerConfig::new()
            .polling_interval(Duration::from_millis(50))
            .heartbeat_interval(Duration::from_secs(2)),
    );

    let outcome = consumer.run_next().await?.unwrap();
    let Outcome::Success(_) = outcome else {
        panic!("Expected job to succeed");
    };

    producer.clean_job(&job_id)?;
    Ok(())
}

#[tokio::test]
async fn test_nonexistent_job() -> Result<()> {
    let context = create_test_context("test_consume");
    let consumer = Consumer::with_context(context, NoopWorker);
    assert!(consumer.run_next().await.is_ok_and(|v| v.is_none()));
    Ok(())
}
