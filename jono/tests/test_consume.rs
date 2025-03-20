#[cfg(all(feature = "produce", feature = "consume"))]
mod jono_tests {
    use jono::prelude::*;
    use jono_core::get_redis_url;
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    #[rustfmt::skip]
    fn test_() -> JonoResult<()> {
        struct MockHandler;
        impl Worker for MockHandler {
            fn handle_job(&self, _workload: Workload) -> JonoResult<Outcome> {
                Ok(Outcome::Success(Some(json!({"processed": true}))))
            }
        }

        let context = create_test_context("test_consume");
        let consumer = Consumer::with_context(context.clone(), Arc::new(MockHandler));

        assert!(consumer.acquire_next_job().is_ok_and(|v| v.is_none()));
    
        let job_metadata = JobMetadata {
            id: "121212121212".to_string(),
            payload: json!({"action": "test_action"}),
            max_attempts: 0,
            initial_priority: 0,
            attempt_count: 0,
            attempt_history: vec![],
            outcome: None,
        };
        assert!(consumer.process_job(job_metadata).is_ok_and(|out| {
            matches!(out, Outcome::Failure(msg) if msg == "Job no longer exists")
        }));

        Ok(())
    }

    #[test]
    fn test_normal() -> JonoResult<()> {
        struct MockHandler;
        impl Worker for MockHandler {
            fn handle_job(&self, _workload: Workload) -> JonoResult<Outcome> {
                Ok(Outcome::Success(Some(json!({"processed": true}))))
            }
        }

        let context = create_test_context("test_consume");
        let inspector = Inspector::with_context(context.clone());
        let producer = Producer::with_context(context.clone());
        let consumer = Consumer::with_context(context.clone(), Arc::new(MockHandler));

        let job_id = producer.dispatch_job(JobPlan::new(json!({"action": "test_action"}))?)?;
        assert_eq!(inspector.get_job_status(&job_id)?, JobStatus::Queued);

        let metadata = consumer.acquire_next_job()?.unwrap();
        assert_eq!(inspector.get_job_status(&metadata.id)?, JobStatus::Running);

        let outcome = consumer.process_job(metadata)?;
        let Outcome::Success(_) = outcome else {
            panic!("Expected job to succeed but got {:?}", outcome);
        };

        assert_eq!(inspector.get_job_status(&job_id)?, JobStatus::Completed);
        let metadata = inspector.get_job_metadata(&job_id)?;
        assert_eq!(metadata.outcome.unwrap(), json!({"processed": true}));

        producer.clean_job(&job_id)?;
        Ok(())
    }

    fn create_test_context(topic: &str) -> JonoContext {
        let redis_url = get_redis_url();
        let forum = JonoForum::new(&redis_url).expect("Failed to connect to Redis");
        forum.topic(topic)
    }
}
