mod common;

use crate::common::JobFixture;
use common::create_test_context;
use jono::prelude::*;
use jono_core::generate_job_id;
use jono_core::{Result, current_timestamp_ms};

#[tokio::test]
async fn test_job_not_found_for_metadata() {
    let context = create_test_context();
    let inspector = Inspector::with_context(context);
    let unknown_job_id = generate_job_id();
    assert!(matches!(
        inspector
            .get_job_metadata(&unknown_job_id)
            .await
            .err()
            .unwrap(),
        JonoError::JobNotFound(_)
    ));
}

#[tokio::test]
async fn test_job_maps_on_all() -> Result<()> {
    let context = create_test_context();
    let inspector = Inspector::with_context(context.clone());

    let now = current_timestamp_ms();
    let queue_fix = JobFixture::new(context.clone(), JobStatus::Queued, 0).await?;
    let run_fix = JobFixture::new(context.clone(), JobStatus::Running, now + 10000).await?;
    let schedule_fix = JobFixture::new(context.clone(), JobStatus::Scheduled, now + 60000).await?;
    let cancel_fix = JobFixture::new(context.clone(), JobStatus::Canceled, now + 30000).await?;
    let harvest_fix = JobFixture::new(context.clone(), JobStatus::Harvestable, now).await?;

    let job_ids = inspector
        .get_status_to_job_ids(JobFilter::default())
        .await?;
    let job_metadatas = inspector
        .get_status_to_job_metadata(JobFilter::default())
        .await?;

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

#[tokio::test]
async fn test_job_maps_on_queued() -> Result<()> {
    let context = create_test_context();
    let inspector = Inspector::with_context(context.clone());

    let queue_fix = JobFixture::new(context.clone(), JobStatus::Queued, 0).await?;

    let job_ids = inspector
        .get_status_to_job_ids(JobFilter::default())
        .await?;
    let job_metadatas = inspector
        .get_status_to_job_metadata(JobFilter::default())
        .await?;

    assert_eq!(job_ids.queued, vec![queue_fix.job_id.clone()]);
    assert!(job_ids.running.is_empty());
    assert!(job_ids.scheduled.is_empty());
    assert!(job_ids.canceled.is_empty());
    assert!(job_ids.harvestable.is_empty());

    assert_eq!(job_metadatas.queued.len(), 1);
    assert_eq!(job_metadatas.running.len(), 0);
    assert_eq!(job_metadatas.scheduled.len(), 0);
    assert_eq!(job_metadatas.canceled.len(), 0);
    assert_eq!(job_metadatas.harvestable.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_job_maps_on_running() -> Result<()> {
    let context = create_test_context();
    let inspector = Inspector::with_context(context.clone());

    let now = current_timestamp_ms();
    let run_fix = JobFixture::new(context.clone(), JobStatus::Running, now + 10000).await?;

    let job_ids = inspector
        .get_status_to_job_ids(JobFilter::default())
        .await?;
    let job_metadatas = inspector
        .get_status_to_job_metadata(JobFilter::default())
        .await?;

    assert!(job_ids.queued.is_empty());
    assert_eq!(job_ids.running, vec![run_fix.job_id.clone()]);
    assert!(job_ids.scheduled.is_empty());
    assert!(job_ids.canceled.is_empty());
    assert!(job_ids.harvestable.is_empty());

    assert_eq!(job_metadatas.queued.len(), 0);
    assert_eq!(job_metadatas.running.len(), 1);
    assert_eq!(job_metadatas.scheduled.len(), 0);
    assert_eq!(job_metadatas.canceled.len(), 0);
    assert_eq!(job_metadatas.harvestable.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_job_maps_on_scheduled() -> Result<()> {
    let context = create_test_context();
    let inspector = Inspector::with_context(context.clone());

    let now = current_timestamp_ms();
    let schedule_fix = JobFixture::new(context.clone(), JobStatus::Scheduled, now + 60000).await?;

    let job_ids = inspector
        .get_status_to_job_ids(JobFilter::default())
        .await?;
    let job_metadatas = inspector
        .get_status_to_job_metadata(JobFilter::default())
        .await?;

    assert!(job_ids.queued.is_empty());
    assert!(job_ids.running.is_empty());
    assert_eq!(job_ids.scheduled, vec![schedule_fix.job_id.clone()]);
    assert!(job_ids.canceled.is_empty());
    assert!(job_ids.harvestable.is_empty());

    assert_eq!(job_metadatas.queued.len(), 0);
    assert_eq!(job_metadatas.running.len(), 0);
    assert_eq!(job_metadatas.scheduled.len(), 1);
    assert_eq!(job_metadatas.canceled.len(), 0);
    assert_eq!(job_metadatas.harvestable.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_job_maps_on_canceled() -> Result<()> {
    let context = create_test_context();
    let inspector = Inspector::with_context(context.clone());

    let now = current_timestamp_ms();
    let cancel_fix = JobFixture::new(context.clone(), JobStatus::Canceled, now + 30000).await?;

    let job_ids = inspector
        .get_status_to_job_ids(JobFilter::default())
        .await?;
    let job_metadatas = inspector
        .get_status_to_job_metadata(JobFilter::default())
        .await?;

    assert!(job_ids.queued.is_empty());
    assert!(job_ids.running.is_empty());
    assert!(job_ids.scheduled.is_empty());
    assert_eq!(job_ids.canceled, vec![cancel_fix.job_id.clone()]);
    assert!(job_ids.harvestable.is_empty());

    assert_eq!(job_metadatas.queued.len(), 0);
    assert_eq!(job_metadatas.running.len(), 0);
    assert_eq!(job_metadatas.scheduled.len(), 0);
    assert_eq!(job_metadatas.canceled.len(), 1);
    assert_eq!(job_metadatas.harvestable.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_job_maps_on_harvestable() -> Result<()> {
    let context = create_test_context();
    let inspector = Inspector::with_context(context.clone());

    let now = current_timestamp_ms();
    let harvest_fix = JobFixture::new(context.clone(), JobStatus::Harvestable, now).await?;

    let job_ids = inspector
        .get_status_to_job_ids(JobFilter::default())
        .await?;
    let job_metadatas = inspector
        .get_status_to_job_metadata(JobFilter::default())
        .await?;

    assert!(job_ids.queued.is_empty());
    assert!(job_ids.running.is_empty());
    assert!(job_ids.scheduled.is_empty());
    assert!(job_ids.canceled.is_empty());
    assert_eq!(job_ids.harvestable, vec![harvest_fix.job_id.clone()]);

    assert_eq!(job_metadatas.queued.len(), 0);
    assert_eq!(job_metadatas.running.len(), 0);
    assert_eq!(job_metadatas.scheduled.len(), 0);
    assert_eq!(job_metadatas.canceled.len(), 0);
    assert_eq!(job_metadatas.harvestable.len(), 1);

    Ok(())
}
