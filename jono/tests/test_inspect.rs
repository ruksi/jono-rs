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
    let postpone_fix = JobFixture::new(context.clone(), JobStatus::Postponed, now + 60000).await?;
    let queue_fix = JobFixture::new(context.clone(), JobStatus::Queued, 0).await?;
    let start_fix = JobFixture::new(context.clone(), JobStatus::Started, now + 10000).await?;
    let abort_fix = JobFixture::new(context.clone(), JobStatus::Aborted, now + 30000).await?;
    let complete_fix = JobFixture::new(context.clone(), JobStatus::Completed, now).await?;

    let job_ids = inspector
        .get_status_to_job_ids(JobFilter::default())
        .await?;
    let job_metadatas = inspector
        .get_status_to_job_metadata(JobFilter::default())
        .await?;

    assert_eq!(job_ids.postponed, vec![postpone_fix.job_id.clone()]);
    assert_eq!(job_ids.queued, vec![queue_fix.job_id.clone()]);
    assert_eq!(job_ids.started, vec![start_fix.job_id.clone()]);
    assert_eq!(job_ids.aborted, vec![abort_fix.job_id.clone()]);
    assert_eq!(job_ids.completed, vec![complete_fix.job_id.clone()]);

    assert_eq!(job_metadatas.postponed.len(), 1);
    assert_eq!(job_metadatas.queued.len(), 1);
    assert_eq!(job_metadatas.started.len(), 1);
    assert_eq!(job_metadatas.aborted.len(), 1);
    assert_eq!(job_metadatas.completed.len(), 1);

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

    assert!(job_ids.postponed.is_empty());
    assert_eq!(job_ids.queued, vec![queue_fix.job_id.clone()]);
    assert!(job_ids.started.is_empty());
    assert!(job_ids.aborted.is_empty());
    assert!(job_ids.completed.is_empty());

    assert_eq!(job_metadatas.postponed.len(), 0);
    assert_eq!(job_metadatas.queued.len(), 1);
    assert_eq!(job_metadatas.started.len(), 0);
    assert_eq!(job_metadatas.aborted.len(), 0);
    assert_eq!(job_metadatas.completed.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_job_maps_on_started() -> Result<()> {
    let context = create_test_context();
    let inspector = Inspector::with_context(context.clone());

    let now = current_timestamp_ms();
    let start_fix = JobFixture::new(context.clone(), JobStatus::Started, now + 10000).await?;

    let job_ids = inspector
        .get_status_to_job_ids(JobFilter::default())
        .await?;
    let job_metadatas = inspector
        .get_status_to_job_metadata(JobFilter::default())
        .await?;

    assert!(job_ids.postponed.is_empty());
    assert!(job_ids.queued.is_empty());
    assert_eq!(job_ids.started, vec![start_fix.job_id.clone()]);
    assert!(job_ids.aborted.is_empty());
    assert!(job_ids.completed.is_empty());

    assert_eq!(job_metadatas.postponed.len(), 0);
    assert_eq!(job_metadatas.queued.len(), 0);
    assert_eq!(job_metadatas.started.len(), 1);
    assert_eq!(job_metadatas.aborted.len(), 0);
    assert_eq!(job_metadatas.completed.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_job_map_on_postponed() -> Result<()> {
    let context = create_test_context();
    let inspector = Inspector::with_context(context.clone());

    let now = current_timestamp_ms();
    let postpone_fix = JobFixture::new(context.clone(), JobStatus::Postponed, now + 60000).await?;

    let job_ids = inspector
        .get_status_to_job_ids(JobFilter::default())
        .await?;
    let job_metadatas = inspector
        .get_status_to_job_metadata(JobFilter::default())
        .await?;

    assert_eq!(job_ids.postponed, vec![postpone_fix.job_id.clone()]);
    assert!(job_ids.queued.is_empty());
    assert!(job_ids.started.is_empty());
    assert!(job_ids.aborted.is_empty());
    assert!(job_ids.completed.is_empty());

    assert_eq!(job_metadatas.postponed.len(), 1);
    assert_eq!(job_metadatas.queued.len(), 0);
    assert_eq!(job_metadatas.started.len(), 0);
    assert_eq!(job_metadatas.aborted.len(), 0);
    assert_eq!(job_metadatas.completed.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_job_maps_on_aborted() -> Result<()> {
    let context = create_test_context();
    let inspector = Inspector::with_context(context.clone());

    let now = current_timestamp_ms();
    let cancel_fix = JobFixture::new(context.clone(), JobStatus::Aborted, now + 30000).await?;

    let job_ids = inspector
        .get_status_to_job_ids(JobFilter::default())
        .await?;
    let job_metadatas = inspector
        .get_status_to_job_metadata(JobFilter::default())
        .await?;

    assert!(job_ids.postponed.is_empty());
    assert!(job_ids.queued.is_empty());
    assert!(job_ids.started.is_empty());
    assert_eq!(job_ids.aborted, vec![cancel_fix.job_id.clone()]);
    assert!(job_ids.completed.is_empty());

    assert_eq!(job_metadatas.postponed.len(), 0);
    assert_eq!(job_metadatas.queued.len(), 0);
    assert_eq!(job_metadatas.started.len(), 0);
    assert_eq!(job_metadatas.aborted.len(), 1);
    assert_eq!(job_metadatas.completed.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_job_maps_on_completed() -> Result<()> {
    let context = create_test_context();
    let inspector = Inspector::with_context(context.clone());

    let now = current_timestamp_ms();
    let complete_fix = JobFixture::new(context.clone(), JobStatus::Completed, now).await?;

    let job_ids = inspector
        .get_status_to_job_ids(JobFilter::default())
        .await?;
    let job_metadatas = inspector
        .get_status_to_job_metadata(JobFilter::default())
        .await?;

    assert!(job_ids.postponed.is_empty());
    assert!(job_ids.queued.is_empty());
    assert!(job_ids.started.is_empty());
    assert!(job_ids.aborted.is_empty());
    assert_eq!(job_ids.completed, vec![complete_fix.job_id.clone()]);

    assert_eq!(job_metadatas.postponed.len(), 0);
    assert_eq!(job_metadatas.queued.len(), 0);
    assert_eq!(job_metadatas.started.len(), 0);
    assert_eq!(job_metadatas.aborted.len(), 0);
    assert_eq!(job_metadatas.completed.len(), 1);

    Ok(())
}
