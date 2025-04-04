mod common;

use crate::common::JobFixture;
use common::create_test_context;
use jono::prelude::*;
use jono_core::generate_job_id;
use jono_core::{Result, current_timestamp_ms};

#[tokio::test]
async fn test_job_not_found_for_metadata() {
    let context = create_test_context("test_not_found");
    let inspector = Inspector::with_context(context);
    let unknown_job_id = generate_job_id();
    assert!(matches!(
        inspector.get_job_metadata(&unknown_job_id).err().unwrap(),
        JonoError::JobNotFound(_)
    ));
}

#[tokio::test]
async fn test_get_current_jobs() -> Result<()> {
    let context = create_test_context("test_sorted_sets");
    let inspector = Inspector::with_context(context.clone());

    let now = current_timestamp_ms();
    let queue_fix = JobFixture::new(context.clone(), JobStatus::Queued, 0)?;
    let run_fix = JobFixture::new(context.clone(), JobStatus::Running, now + 10000)?;
    let schedule_fix = JobFixture::new(context.clone(), JobStatus::Scheduled, now + 60000)?;
    let cancel_fix = JobFixture::new(context.clone(), JobStatus::Canceled, now + 30000)?;
    let harvest_fix = JobFixture::new(context.clone(), JobStatus::Harvestable, now)?;

    let job_ids = inspector.get_status_to_job_ids(JobFilter::default())?;
    let job_metadatas = inspector.get_status_to_job_metadata(JobFilter::default())?;

    assert_eq!(job_ids.queued, vec![queue_fix.job_id.clone()]);
    assert_eq!(job_ids.running, vec![run_fix.job_id.clone()]);
    assert_eq!(job_ids.scheduled, vec![schedule_fix.job_id.clone()]);
    assert_eq!(job_ids.canceled, vec![cancel_fix.job_id.clone()]);
    assert_eq!(job_ids.harvestable, vec![harvest_fix.job_id.clone()]);

    assert_eq!(job_metadatas.queued.len(), 1);
    assert_eq!(job_metadatas.running.len(), 1);
    assert_eq!(job_metadatas.scheduled.len(), 1);
    assert_eq!(job_metadatas.canceled.len(), 1);
    assert_eq!(job_metadatas.harvestable.len(), 1);

    Ok(())
}
