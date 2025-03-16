use jono_core::{
    current_timestamp_ms, generate_job_id, get_redis_url, JobStatus, JobStatusReader, JonoContext,
    JonoError, JonoForum, JonoResult,
};
use jono_produce::{JobPlan, Producer};
use serde_json::json;

#[test]
fn test_dispatch_job() -> JonoResult<()> {
    let context = create_test_context("test_dispatch");
    let producer = Producer::with_context(context.clone());
    let payload = json!({
        "action": "test_action",
        "data": "test_data"
    });

    let plan = JobPlan::new(&payload)?;
    let job_id = plan.id.clone();
    producer.clean_job(&job_id)?;
    let job_id = producer.dispatch(plan)?;

    let metadata = producer.get_job_metadata(&job_id)?;
    assert_eq!(metadata.payload, payload);

    let status_reader = JobStatusReader::with_context(context);
    let status = status_reader.get_job_status(&job_id)?;
    assert_eq!(status, JobStatus::Queued);

    producer.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_cancel_dispatched_job() -> JonoResult<()> {
    let context = create_test_context("test_cancel");
    let producer = Producer::with_context(context.clone());
    let payload = json!({ "action": "cancel this soon!" });

    let plan = JobPlan::new(payload)?;
    let job_id = plan.id.clone();
    producer.clean_job(&job_id)?;
    let job_id = producer.dispatch(plan)?;

    let cancel_ok = producer.cancel(&job_id, 0)?;
    assert!(cancel_ok);

    let _metadata = producer.get_job_metadata(&job_id)?;
    let status_reader = JobStatusReader::with_context(context);
    let status = status_reader.get_job_status(&job_id)?;
    assert_eq!(status, JobStatus::Canceled);

    producer.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_dispatch_scheduled_job() -> JonoResult<()> {
    let context = create_test_context("test_schedule");
    let producer = Producer::with_context(context.clone());
    let payload = json!({ "action": "scheduled_action" });
    let future_time = current_timestamp_ms() + 10000;

    let plan = JobPlan::new(payload)?.schedule_for(future_time);
    let job_id = plan.id.clone();
    producer.clean_job(&job_id)?;
    let job_id = producer.dispatch(plan)?;

    let _metadata = producer.get_job_metadata(&job_id)?;
    let status_reader = JobStatusReader::with_context(context);
    let status = status_reader.get_job_status(&job_id)?;
    assert_eq!(status, JobStatus::Scheduled);

    producer.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_job_not_found() {
    let context = create_test_context("test_not_found");
    let producer = Producer::with_context(context);
    let unknown_job_id = generate_job_id();

    let metadata_result = producer.get_job_metadata(&unknown_job_id);
    assert!(matches!(
        metadata_result.err().unwrap(),
        JonoError::NotFound(_)
    ));

    let cancel_result = producer.cancel(&unknown_job_id, 0);
    assert!(matches!(
        cancel_result.err().unwrap(),
        JonoError::NotFound(_)
    ));
}

#[test]
fn test_clean_job() -> JonoResult<()> {
    let context = create_test_context("test_clean");
    let producer = Producer::with_context(context.clone());
    let payload = json!({ "action": "clean this soon!" });

    let plan = JobPlan::new(payload)?;
    let job_id = plan.id.clone();
    producer.clean_job(&job_id)?;
    let job_id = producer.dispatch(plan)?;

    let exists_before = producer.get_job_metadata(&job_id).is_ok();
    assert!(exists_before);

    let clean_ok = producer.clean_job(&job_id)?;
    assert!(clean_ok);

    let exists_after = producer.get_job_metadata(&job_id);
    assert!(matches!(
        exists_after.err().unwrap(),
        JonoError::NotFound(_)
    ));

    let status_reader = JobStatusReader::with_context(context);
    let status_result = status_reader.get_job_status(&job_id);
    assert!(matches!(
        status_result.err().unwrap(),
        JonoError::NotFound(_)
    ));

    Ok(())
}

fn create_test_context(topic: &str) -> JonoContext {
    let redis_url = get_redis_url();
    let forum = JonoForum::new(&redis_url).expect("Failed to connect to Redis");
    forum.topic(topic)
}
