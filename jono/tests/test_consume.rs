#![cfg(all(feature = "produce", feature = "consume"))]

mod common;

use common::create_test_context;
use jono::prelude::*;
use jono_core::Result;
use serde_json::json;

struct NoopWorker;

impl Worker for NoopWorker {
    async fn process(&self, _: &Workload) -> Result<WorkSummary> {
        Ok(WorkSummary::Success(Some(json!({"processed": true}))))
    }
}

#[tokio::test]
async fn test_basics() -> Result<()> {
    let context = create_test_context();
    let inspector = Inspector::with_context(context.clone());
    let producer = Producer::with_context(context.clone());
    let job_id = JobPlan::new()
        .payload(json!({"action": "test_action"}))
        .submit(&producer)
        .await?;
    assert_eq!(inspector.get_job_status(&job_id).await?, JobStatus::Queued);

    let consumer = Consumer::with_context(context.clone(), NoopWorker);
    let summary = consumer.run_next().await?;
    let Some(WorkSummary::Success(_)) = summary else {
        panic!("Expected job to succeed but got {:?}", summary);
    };

    let metadata = inspector.get_job_metadata(&job_id).await?;
    assert_eq!(metadata.work_summary.unwrap(), json!({"processed": true}));
    assert_eq!(
        inspector.get_job_status(&job_id).await?,
        JobStatus::Harvestable
    );

    producer.clean_job(&job_id).await?;
    Ok(())
}

#[tokio::test]
async fn test_with_config() -> Result<()> {
    use std::time::Duration;

    let context = create_test_context();
    let producer = Producer::with_context(context.clone());
    let job_id = JobPlan::new()
        .payload(json!({"action": "configured_action"}))
        .submit(&producer)
        .await?;

    let consumer = Consumer::with_context(context.clone(), NoopWorker).with_config(
        ConsumerConfig::new()
            .polling_interval(Duration::from_millis(50))
            .heartbeat_interval(Duration::from_secs(2)),
    );

    let work_summary = consumer.run_next().await?.unwrap();
    let WorkSummary::Success(_) = work_summary else {
        panic!("Expected job to succeed");
    };

    producer.clean_job(&job_id).await?;
    Ok(())
}

#[tokio::test]
async fn test_nonexistent_job() -> Result<()> {
    let context = create_test_context();
    let consumer = Consumer::with_context(context, NoopWorker);
    assert!(consumer.run_next().await.is_ok_and(|v| v.is_none()));
    Ok(())
}
