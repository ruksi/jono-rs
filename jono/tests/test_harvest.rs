#![cfg(all(feature = "produce", feature = "consume", feature = "harvest"))]

mod common;
mod reapers;
mod workers;

use common::create_test_context;
use jono::prelude::*;
use jono_core::Result;
use reapers::NoopReaper;
use serde_json::json;
use std::time::Duration;
use workers::NoopWorker;

#[tokio::test]
async fn test_basics() -> Result<()> {
    let context = create_test_context();
    let inspector = Inspector::with_context(context.clone());
    let producer = Producer::with_context(context.clone());
    let harvester = Harvester::with_context(context.clone(), NoopReaper)
        .with_config(HarvestConfig::new().poll_timeout(Duration::from_millis(1)));

    assert_eq!(harvester.harvest(1).await?.len(), 0);

    let job_id = JobPlan::new()
        .payload(json!({"action": "test_action"}))
        .submit(&producer)
        .await?;
    assert_eq!(inspector.get_job_status(&job_id).await?, JobStatus::Queued);

    assert_eq!(harvester.harvest(1).await?.len(), 0);

    let consumer = Consumer::with_context(context.clone(), NoopWorker);
    let work_summary = consumer.run_next().await?;
    assert!(matches!(work_summary, Some(WorkSummary::Success(_))));

    // it shouldn't be expired, yet
    let expired_count = harvester.clean_expired_completed().await?;
    assert_eq!(expired_count, 0);

    // you can pop the completed jobs
    let completed = harvester.harvest(3).await?;
    assert_eq!(completed.len(), 1);
    let job_metadata = completed.first().unwrap().clone();
    assert_eq!(job_metadata.payload, json!({"action": "test_action"}));
    assert_eq!(
        job_metadata.work_summary.unwrap(),
        json!({"processed": true})
    );

    // the single harvestable was processed already
    let completed = harvester.harvest(1).await?;
    assert_eq!(completed.len(), 0);

    producer.clean_job(&job_id).await?;
    Ok(())
}

#[tokio::test]
async fn test_nonexistent() -> Result<()> {
    let context = create_test_context();
    let harvester = Harvester::with_context(context.clone(), NoopReaper)
        .with_config(HarvestConfig::new().poll_timeout(Duration::from_millis(1)));
    assert_eq!(harvester.harvest(1).await?.len(), 0);
    assert_eq!(harvester.harvest(2).await?.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_clean_harvest() -> Result<()> {
    let context = create_test_context();
    let producer = Producer::with_context(context.clone());
    let harvester = Harvester::with_context(context.clone(), NoopReaper)
        .with_config(HarvestConfig::new().poll_timeout(Duration::from_millis(1)));

    let job_id = JobPlan::new()
        .payload(json!({"action": "test_action"}))
        .submit(&producer)
        .await?;

    let consumer = Consumer::with_context(context.clone(), NoopWorker);
    let work_summary = consumer.run_next().await?;
    assert!(matches!(work_summary, Some(WorkSummary::Success(_))));

    // if the job is cleaned, it can't be harvested even once
    producer.clean_job(&job_id).await?;
    assert_eq!(harvester.harvest(1).await?.len(), 0);
    producer.clean_job(&job_id).await?;

    Ok(())
}
