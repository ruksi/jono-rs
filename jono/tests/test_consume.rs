#![cfg(all(feature = "produce", feature = "consume"))]

use jono::prelude::*;
use jono_core::get_redis_url;
use serde_json::json;
use std::sync::Arc;

struct NoopWorker;

impl Worker for NoopWorker {
    fn handle_job(&self, _workload: Workload) -> Result<Outcome> {
        Ok(Outcome::Success(Some(json!({"processed": true}))))
    }
}

#[test]
fn test_nonexistent_job() -> Result<()> {
    let context = create_test_context("test_consume");
    let consumer = Consumer::with_context(context, Arc::new(NoopWorker));

    assert!(consumer.acquire_next_job().is_ok_and(|v| v.is_none()));

    let imaginary_metadata = JobMetadata {
        id: "121212121212".to_string(),
        payload: json!({"action": "test_action"}),
        max_attempts: 0,
        initial_priority: 0,
        attempt_count: 0,
        attempt_history: vec![],
        outcome: None,
    };
    assert!(consumer.process_job(imaginary_metadata).is_ok_and(|out| {
        matches!(out, Outcome::Failure(msg) if msg == "Job no longer exists")
    }));

    Ok(())
}

#[test]
fn test_basics() -> Result<()> {
    let context = create_test_context("test_consume");
    let inspector = Inspector::with_context(context.clone());
    let producer = Producer::with_context(context.clone());
    let consumer = Consumer::with_context(context.clone(), Arc::new(NoopWorker));

    let job_id = producer.dispatch_job(JobPlan::new(json!({"action": "test_action"}))?)?;
    assert_eq!(inspector.get_job_status(&job_id)?, JobStatus::Queued);

    let metadata = consumer.acquire_next_job()?.unwrap();
    assert_eq!(inspector.get_job_status(&metadata.id)?, JobStatus::Running);

    let outcome = consumer.process_job(metadata)?;
    let Outcome::Success(_) = outcome else {
        panic!("Expected job to succeed but got {:?}", outcome);
    };
    let metadata = inspector.get_job_metadata(&job_id)?;
    assert_eq!(metadata.outcome.unwrap(), json!({"processed": true}));
    assert_eq!(inspector.get_job_status(&job_id)?, JobStatus::Completed);

    producer.clean_job(&job_id)?;
    Ok(())
}

fn create_test_context(topic: &str) -> Context {
    let redis_url = get_redis_url();
    let forum = Forum::new(&redis_url).expect("Failed to connect to Redis");
    forum.topic(topic)
}
