#![cfg(all(feature = "produce", feature = "consume", feature = "harvest"))]

mod common;
mod reapers;
mod workers;

use common::create_test_context;
use jono::prelude::*;
use jono_core::Result;
use serde_json::json;
use workers::NoopWorker;

#[tokio::test]
async fn test_reaping() -> Result<()> {
    let context = create_test_context();
    let producer = Producer::with_context(context.clone());

    let job_id = JobPlan::new()
        .payload(json!({"action": "reap me!"}))
        .submit(&producer)
        .await?;

    let consumer = Consumer::with_context(context.clone(), NoopWorker);
    let work_summary = consumer.run_next().await?;
    assert!(matches!(work_summary, Some(WorkSummary::Success(_))));

    struct MyReaper;

    impl Reaper for MyReaper {
        async fn reap(&self, reapload: &Reapload) -> Result<ReapSummary> {
            Ok(ReapSummary::Success(Some(json!({
                "reaped": true,
                "job_id": reapload.job_id
            }))))
        }
    }

    let harvester = Harvester::with_context(context.clone(), MyReaper);

    // the job should now be in the completed set,
    // ready for reaping
    let reap_summaries = harvester.reap_next_batch().await?;
    assert_eq!(reap_summaries.len(), 1);

    if let ReapSummary::Success(Some(data)) = &reap_summaries[0] {
        assert_eq!(data["reaped"], json!(true));
        assert_eq!(data["job_id"], json!(job_id));
    } else {
        panic!("Expected success summary with data");
    }

    producer.clean_job(&job_id).await?;
    Ok(())
}
