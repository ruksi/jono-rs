mod common;

use common::create_test_context;
use jono::prelude::*;
use jono_core::generate_job_id;
use jono_core::{Result, current_timestamp_ms};

#[test]
fn test_job_not_found_for_metadata() {
    let context = create_test_context("test_not_found");
    let inspector = Inspector::with_context(context);
    let unknown_job_id = generate_job_id();
    assert!(matches!(
        inspector.get_job_metadata(&unknown_job_id).err().unwrap(),
        JonoError::JobNotFound(_)
    ));
}

#[test]
fn test_get_current_jobs() -> Result<()> {
    let context = create_test_context("test_sorted_sets");
    let inspector = Inspector::with_context(context.clone());

    let mut conn = context.get_connection()?;
    let keys = context.keys();
    let now = current_timestamp_ms();

    let queued_job_id = generate_job_id();
    let queued_key = keys.job_metadata_hash(&queued_job_id);
    let priority = 0;
    redis::cmd("ZADD")
        .arg(keys.queued_set())
        .arg(priority)
        .arg(&queued_job_id)
        .query::<()>(&mut conn)?;
    let _: () = redis::pipe()
        .hset(&queued_key, "id", &queued_job_id)
        .hset(&queued_key, "payload", "{}")
        .hset(&queued_key, "max_attempts", "1")
        .hset(&queued_key, "initial_priority", "0")
        .hset(&queued_key, "created_at", now.to_string())
        .hset(&queued_key, "attempt_history", "[]")
        .hset(&queued_key, "outcome", "null")
        .query(&mut conn)?;

    let running_job_id = generate_job_id();
    let running_expiry = now + 10000;
    redis::cmd("ZADD")
        .arg(keys.running_set())
        .arg(running_expiry)
        .arg(&running_job_id)
        .query::<()>(&mut conn)?;

    let scheduled_job_id = generate_job_id();
    let scheduled_time = now + 60000;
    redis::cmd("ZADD")
        .arg(keys.scheduled_set())
        .arg(scheduled_time)
        .arg(&scheduled_job_id)
        .query::<()>(&mut conn)?;

    let canceled_job_id = generate_job_id();
    let grace_time = now + 30000;
    redis::cmd("ZADD")
        .arg(keys.canceled_set())
        .arg(grace_time)
        .arg(&canceled_job_id)
        .query::<()>(&mut conn)?;

    let harvestable_job_id = generate_job_id();
    redis::cmd("ZADD")
        .arg(keys.harvestable_set())
        .arg(now)
        .arg(&harvestable_job_id)
        .query::<()>(&mut conn)?;

    let job_ids = inspector.get_status_to_job_ids(JobFilter::default())?;
    let job_metadatas = inspector.get_status_to_job_metadata(JobFilter::default())?;

    // do the cleaning before asserts _just in case_
    #[rustfmt::skip]
    redis::pipe()
        .cmd("DEL").arg(keys.queued_set()).ignore()
        .cmd("DEL").arg(queued_key).ignore()
        .cmd("DEL").arg(keys.running_set()).ignore()
        .cmd("DEL").arg(keys.scheduled_set()).ignore()
        .cmd("DEL").arg(keys.canceled_set()).ignore()
        .cmd("DEL").arg(keys.harvestable_set()).ignore()
        .query::<()>(&mut conn)?;

    assert_eq!(job_ids.queued, vec![queued_job_id]);
    assert_eq!(job_ids.running, vec![running_job_id]);
    assert_eq!(job_ids.scheduled, vec![scheduled_job_id]);
    assert_eq!(job_ids.canceled, vec![canceled_job_id]);
    assert_eq!(job_ids.harvestable, vec![harvestable_job_id]);

    assert_eq!(job_metadatas.queued.len(), 1);
    assert_eq!(job_metadatas.running.len(), 0);
    assert_eq!(job_metadatas.scheduled.len(), 0);
    assert_eq!(job_metadatas.canceled.len(), 0);
    assert_eq!(job_metadatas.harvestable.len(), 0);

    Ok(())
}
