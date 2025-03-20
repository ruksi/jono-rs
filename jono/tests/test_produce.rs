#![cfg(all(feature = "produce"))]

use jono::prelude::*;
use jono_core::{current_timestamp_ms, generate_job_id, get_redis_url};
use serde_json::json;

#[test]
fn test_dispatch_job() -> Result<()> {
    let context = create_test_context("test_dispatch");
    let producer = Producer::with_context(context.clone());
    let inspector = Inspector::with_context(context);

    let payload = json!({"action": "test_action", "one": 1, "two": 2, "three": 3});
    let plan = JobPlan::new(&payload)?;
    let job_id = plan.id.clone();
    producer.clean_job(&job_id)?;
    let job_id = producer.dispatch_job(plan)?;

    let metadata = inspector.get_job_metadata(&job_id)?;
    assert_eq!(metadata.payload, payload);
    assert_eq!(inspector.get_job_status(&job_id)?, JobStatus::Queued);

    producer.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_dispatch_scheduled_job() -> Result<()> {
    let context = create_test_context("test_schedule");
    let producer = Producer::with_context(context.clone());
    let inspector = Inspector::with_context(context);

    let payload = json!({ "action": "run this later!" });
    let future_time = current_timestamp_ms() + 10000;
    let plan = JobPlan::new(payload)?.schedule_for(future_time);
    let job_id = plan.id.clone();
    producer.clean_job(&job_id)?;
    let job_id = producer.dispatch_job(plan)?;

    assert!(inspector.job_exists(&job_id)?);
    inspector.get_job_metadata(&job_id)?;
    let status = inspector.get_job_status(&job_id)?;
    assert_eq!(status, JobStatus::Scheduled);

    producer.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_cancel_job() -> Result<()> {
    let context = create_test_context("test_cancel");
    let producer = Producer::with_context(context.clone());
    let inspector = Inspector::with_context(context);

    let payload = json!({ "action": "cancel this soon!" });
    let plan = JobPlan::new(payload)?;
    let job_id = plan.id.clone();
    producer.clean_job(&job_id)?;
    let job_id = producer.dispatch_job(plan)?;

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

    let payload = json!({ "action": "clean this soon!" });
    let plan = JobPlan::new(payload)?;
    let job_id = plan.id.clone();
    producer.clean_job(&job_id)?;
    let job_id = producer.dispatch_job(plan)?;

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
fn test_job_not_found() {
    let context = create_test_context("test_not_found");
    let producer = Producer::with_context(context.clone());
    let inspector = Inspector::with_context(context);

    let unknown_job_id = generate_job_id();
    assert!(matches!(
        inspector.get_job_metadata(&unknown_job_id).err().unwrap(),
        Error::NotFound(_)
    ));
    assert!(matches!(
        producer.cancel_job(&unknown_job_id, 0).err().unwrap(),
        Error::NotFound(_)
    ));
}

fn create_test_context(topic: &str) -> Context {
    let redis_url = get_redis_url();
    let forum = Forum::new(&redis_url).expect("Failed to connect to Redis");
    forum.topic(topic)
}
