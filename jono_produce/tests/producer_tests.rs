use jono_core::{
    current_timestamp_ms, generate_job_id, get_redis_url, JobStatus, JonoError, JonoResult,
};
use jono_produce::{JobPlan, Producer};
use serde_json::json;

fn create_test_producer(topic: &str) -> Producer {
    let redis_url = get_redis_url();
    Producer::new(&redis_url, topic).expect("Failed to create Jono Producer")
}

#[test]
fn test_dispatch_job() -> JonoResult<()> {
    let producer = create_test_producer("test_dispatch");
    let payload = json!({
        "action": "test_action",
        "data": "test_data"
    });

    let plan = JobPlan::new(&payload)?;
    let job_id = plan.id.clone();
    producer.clean_job(&job_id)?;
    let job_id = producer.dispatch(plan)?;

    let metadata = producer.get_job_metadata(&job_id)?;
    assert_eq!(metadata.status, JobStatus::Queued);
    assert_eq!(metadata.payload, payload);

    producer.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_cancel_dispatched_job() -> JonoResult<()> {
    let producer = create_test_producer("test_cancel");
    let payload = json!({ "action": "cancel this soon!" });

    let plan = JobPlan::new(payload)?;
    let job_id = plan.id.clone();
    producer.clean_job(&job_id)?;
    let job_id = producer.dispatch(plan)?;

    let cancel_ok = producer.cancel(&job_id, 0)?;
    assert!(cancel_ok);

    let metadata = producer.get_job_metadata(&job_id)?;
    assert_eq!(metadata.status, JobStatus::Cancelled);

    producer.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_dispatch_scheduled_job() -> JonoResult<()> {
    let producer = create_test_producer("test_schedule");
    let payload = json!({ "action": "scheduled_action" });
    let future_time = current_timestamp_ms() + 10000;

    let plan = JobPlan::new(payload)?.schedule_for(future_time);
    let job_id = plan.id.clone();
    producer.clean_job(&job_id)?;
    let job_id = producer.dispatch(plan)?;

    let metadata = producer.get_job_metadata(&job_id)?;
    assert_eq!(metadata.status, JobStatus::Scheduled);

    producer.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_job_not_found() {
    let producer = create_test_producer("test_not_found");
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
    let producer = create_test_producer("test_clean");
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
    Ok(())
}
