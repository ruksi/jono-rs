#![cfg(all(feature = "produce", feature = "consume", feature = "harvest"))]

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
    let context = create_test_context("test_harvest");
    let inspector = Inspector::with_context(context.clone());
    let producer = Producer::with_context(context.clone());
    let harvester = Harvester::with_context(context.clone());

    assert_eq!(harvester.harvest(1).await?.len(), 0);

    let job_id = JobPlan::new()
        .payload(json!({"action": "test_action"}))
        .submit(&producer)
        .await?;
    assert_eq!(inspector.get_job_status(&job_id).await?, JobStatus::Queued);

    assert_eq!(harvester.harvest(1).await?.len(), 0);

    let consumer = Consumer::with_context(context.clone(), NoopWorker);
    let outcome = consumer.run_next().await?;
    assert!(matches!(outcome, Some(Outcome::Success(_))));

    // it shouldn't be expired, yet
    let expired_count = harvester.clean_expired_harvest().await?;
    assert_eq!(expired_count, 0);

    // you can pop the harvestables
    let harvestables = harvester.harvest(3).await?;
    assert_eq!(harvestables.len(), 1);
    let job_metadata = harvestables.first().unwrap().clone();
    assert_eq!(job_metadata.payload, json!({"action": "test_action"}));
    assert_eq!(job_metadata.outcome.unwrap(), json!({"processed": true}));

    // the single harvestable was processed already
    let harvestables = harvester.harvest(1).await?;
    assert_eq!(harvestables.len(), 0);

    producer.clean_job(&job_id).await?;
    Ok(())
}

#[tokio::test]
async fn test_nonexistent() -> Result<()> {
    let context = create_test_context("test_nothing_to_harvest");
    let harvester = Harvester::with_context(context.clone());
    assert_eq!(harvester.harvest(0).await?.len(), 0);
    assert_eq!(harvester.harvest(1).await?.len(), 0);
    assert_eq!(harvester.harvest(2).await?.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_clean_harvest() -> Result<()> {
    let context = create_test_context("test_clean_harvest");
    let producer = Producer::with_context(context.clone());
    let harvester = Harvester::with_context(context.clone());

    let job_id = JobPlan::new()
        .payload(json!({"action": "test_action"}))
        .submit(&producer)
        .await?;

    let consumer = Consumer::with_context(context.clone(), NoopWorker);
    let outcome = consumer.run_next().await?;
    assert!(matches!(outcome, Some(Outcome::Success(_))));

    // if the job is cleaned, it can't be harvested even once
    producer.clean_job(&job_id).await?;
    assert_eq!(harvester.harvest(1).await?.len(), 0);
    producer.clean_job(&job_id).await?;

    Ok(())
}
