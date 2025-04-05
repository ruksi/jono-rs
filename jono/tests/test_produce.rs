#![cfg(all(feature = "produce"))]

mod common;

use common::create_test_context;
use jono::prelude::*;
use jono_core::{Result, current_timestamp_ms, generate_job_id};
use serde_json::json;

#[tokio::test]
async fn test_submit_job() -> Result<()> {
    let context = create_test_context("test_submit");
    let producer = Producer::with_context(context.clone());
    let inspector = Inspector::with_context(context);

    let job_id = JobPlan::new()
        .payload(json!({"one": 1, "two": 2}))
        .submit(&producer)
        .await?;

    let metadata = inspector.get_job_metadata(&job_id).await?;
    assert_eq!(metadata.payload, json!({"one": 1, "two": 2}));
    assert_eq!(inspector.get_job_status(&job_id).await?, JobStatus::Queued);

    producer.clean_job(&job_id).await?;
    Ok(())
}

#[tokio::test]
async fn test_submit_scheduled_job() -> Result<()> {
    let context = create_test_context("test_schedule");
    let producer = Producer::with_context(context.clone());
    let inspector = Inspector::with_context(context);

    let payload = json!({ "action": "run this later!" });
    let future_time = current_timestamp_ms() + 10000;
    let job_id = JobPlan::new()
        .payload(payload)
        .scheduled_for(future_time)
        .submit(&producer)
        .await?;

    assert!(inspector.job_exists(&job_id).await?);
    inspector.get_job_metadata(&job_id).await?;
    assert_eq!(
        inspector.get_job_status(&job_id).await?,
        JobStatus::Scheduled
    );

    producer.clean_job(&job_id).await?;
    Ok(())
}

#[tokio::test]
async fn test_cancel_job() -> Result<()> {
    let context = create_test_context("test_cancel");
    let producer = Producer::with_context(context.clone());
    let inspector = Inspector::with_context(context);

    let job_id = JobPlan::new()
        .payload(json!({ "action": "cancel this soon!" }))
        .submit(&producer)
        .await?;

    assert!(producer.cancel_job(&job_id, 0).await.is_ok());

    assert!(inspector.job_exists(&job_id).await?);
    inspector.get_job_metadata(&job_id).await?;
    assert_eq!(
        inspector.get_job_status(&job_id).await?,
        JobStatus::Canceled
    );

    producer.clean_job(&job_id).await?;
    Ok(())
}

#[tokio::test]
async fn test_clean_job() -> Result<()> {
    let context = create_test_context("test_clean");
    let producer = Producer::with_context(context.clone());
    let inspector = Inspector::with_context(context);

    let job_id = JobPlan::new()
        .payload(json!({ "action": "clean this soon!" }))
        .submit(&producer)
        .await?;

    // before clean
    assert!(inspector.job_exists(&job_id).await?);
    assert!(inspector.get_job_metadata(&job_id).await.is_ok());
    assert_eq!(inspector.get_job_status(&job_id).await?, JobStatus::Queued);

    // clean
    assert!(producer.clean_job(&job_id).await?);

    // after clean
    assert!(!inspector.job_exists(&job_id).await?);
    assert!(matches!(
        inspector.get_job_metadata(&job_id).await.err().unwrap(),
        JonoError::JobNotFound(_)
    ));
    assert!(matches!(
        inspector.get_job_status(&job_id).await.err().unwrap(),
        JonoError::JobNotFound(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_job_not_found_for_cancel() {
    let context = create_test_context("test_not_found");
    let producer = Producer::with_context(context.clone());
    let unknown_job_id = generate_job_id();
    assert!(matches!(
        producer.cancel_job(&unknown_job_id, 0).await.err().unwrap(),
        JonoError::JobNotFound(_)
    ));
}
