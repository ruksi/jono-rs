#![cfg(all(feature = "produce"))]

mod common;

use common::create_test_context;
use jono::prelude::*;
use jono_core::{current_timestamp_ms, generate_job_id};
use serde_json::json;

#[test]
fn test_submit_job() -> Result<()> {
    let context = create_test_context("test_submit");
    let producer = Producer::with_context(context.clone());
    let inspector = Inspector::with_context(context);

    let job_id = JobPlan::new()
        .payload(json!({"one": 1, "two": 2}))
        .submit(&producer)?;

    let metadata = inspector.get_job_metadata(&job_id)?;
    assert_eq!(metadata.payload, json!({"one": 1, "two": 2}));
    assert_eq!(inspector.get_job_status(&job_id)?, JobStatus::Queued);

    producer.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_submit_scheduled_job() -> Result<()> {
    let context = create_test_context("test_schedule");
    let producer = Producer::with_context(context.clone());
    let inspector = Inspector::with_context(context);

    let payload = json!({ "action": "run this later!" });
    let future_time = current_timestamp_ms() + 10000;
    let job_id = JobPlan::new()
        .payload(payload)
        .scheduled_for(future_time)
        .submit(&producer)?;

    assert!(inspector.job_exists(&job_id)?);
    inspector.get_job_metadata(&job_id)?;
    assert_eq!(inspector.get_job_status(&job_id)?, JobStatus::Scheduled);

    producer.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_cancel_job() -> Result<()> {
    let context = create_test_context("test_cancel");
    let producer = Producer::with_context(context.clone());
    let inspector = Inspector::with_context(context);

    let job_id = JobPlan::new()
        .payload(json!({ "action": "cancel this soon!" }))
        .submit(&producer)?;

    assert!(producer.cancel_job(&job_id, 0).is_ok());

    assert!(inspector.job_exists(&job_id)?);
    inspector.get_job_metadata(&job_id)?;
    assert_eq!(inspector.get_job_status(&job_id)?, JobStatus::Canceled);

    producer.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_clean_job() -> Result<()> {
    let context = create_test_context("test_clean");
    let producer = Producer::with_context(context.clone());
    let inspector = Inspector::with_context(context);

    let job_id = JobPlan::new()
        .payload(json!({ "action": "clean this soon!" }))
        .submit(&producer)?;

    // before clean
    assert!(inspector.job_exists(&job_id)?);
    assert!(inspector.get_job_metadata(&job_id).is_ok());
    assert_eq!(inspector.get_job_status(&job_id)?, JobStatus::Queued);

    // clean
    assert!(producer.clean_job(&job_id)?);

    // after clean
    assert!(!inspector.job_exists(&job_id)?);
    assert!(matches!(
        inspector.get_job_metadata(&job_id).err().unwrap(),
        Error::NotFound(_)
    ));
    assert!(matches!(
        inspector.get_job_status(&job_id).err().unwrap(),
        Error::NotFound(_)
    ));

    Ok(())
}

#[test]
fn test_job_not_found_for_cancel() {
    let context = create_test_context("test_not_found");
    let producer = Producer::with_context(context.clone());
    let unknown_job_id = generate_job_id();
    assert!(matches!(
        producer.cancel_job(&unknown_job_id, 0).err().unwrap(),
        Error::NotFound(_)
    ));
}
