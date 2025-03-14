use jono_core::{
    current_timestamp_ms, generate_job_id, get_redis_url, JobStatus, JonoError, JonoResult,
};
use jono_dispatch::{Dispatcher, JobPlan};
use serde_json::json;

fn create_test_dispatcher(topic: &str) -> Dispatcher {
    let redis_url = get_redis_url();
    Dispatcher::new(&redis_url, topic).expect("Failed to create dispatcher")
}

#[test]
fn test_dispatch_job() -> JonoResult<()> {
    let dispatcher = create_test_dispatcher("test_dispatch");
    let payload = json!({
        "action": "test_action",
        "data": "test_data"
    });

    let plan = JobPlan::new(&payload)?;
    let job_id = plan.id.clone();
    dispatcher.clean_job(&job_id)?;
    let job_id = dispatcher.dispatch(plan)?;

    let metadata = dispatcher.get_job_metadata(&job_id)?;
    assert_eq!(metadata.status, JobStatus::Queued);
    assert_eq!(metadata.payload, payload);

    dispatcher.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_cancel_dispatched_job() -> JonoResult<()> {
    let dispatcher = create_test_dispatcher("test_cancel");
    let payload = json!({ "action": "cancel this soon!" });

    let plan = JobPlan::new(payload)?;
    let job_id = plan.id.clone();
    dispatcher.clean_job(&job_id)?;
    let job_id = dispatcher.dispatch(plan)?;

    let cancel_ok = dispatcher.cancel(&job_id, 0)?;
    assert!(cancel_ok);

    let metadata = dispatcher.get_job_metadata(&job_id)?;
    assert_eq!(metadata.status, JobStatus::Cancelled);

    dispatcher.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_dispatch_scheduled_job() -> JonoResult<()> {
    let dispatcher = create_test_dispatcher("test_schedule");
    let payload = json!({ "action": "scheduled_action" });
    let future_time = current_timestamp_ms() + 10000;

    let plan = JobPlan::new(payload)?.schedule_for(future_time);
    let job_id = plan.id.clone();
    dispatcher.clean_job(&job_id)?;
    let job_id = dispatcher.dispatch(plan)?;

    let metadata = dispatcher.get_job_metadata(&job_id)?;
    assert_eq!(metadata.status, JobStatus::Scheduled);

    dispatcher.clean_job(&job_id)?;
    Ok(())
}

#[test]
fn test_job_not_found() {
    let dispatcher = create_test_dispatcher("test_not_found");
    let unknown_job_id = generate_job_id();

    let metadata_result = dispatcher.get_job_metadata(&unknown_job_id);
    assert!(matches!(
        metadata_result.err().unwrap(),
        JonoError::NotFound(_)
    ));

    let cancel_result = dispatcher.cancel(&unknown_job_id, 0);
    assert!(matches!(
        cancel_result.err().unwrap(),
        JonoError::NotFound(_)
    ));
}

#[test]
fn test_clean_job() -> JonoResult<()> {
    let dispatcher = create_test_dispatcher("test_clean");
    let payload = json!({ "action": "clean this soon!" });

    let plan = JobPlan::new(payload)?;
    let job_id = plan.id.clone();
    dispatcher.clean_job(&job_id)?;
    let job_id = dispatcher.dispatch(plan)?;

    let exists_before = dispatcher.get_job_metadata(&job_id).is_ok();
    assert!(exists_before);

    let clean_ok = dispatcher.clean_job(&job_id)?;
    assert!(clean_ok);

    let exists_after = dispatcher.get_job_metadata(&job_id);
    assert!(matches!(
        exists_after.err().unwrap(),
        JonoError::NotFound(_)
    ));
    Ok(())
}
