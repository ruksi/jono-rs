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
    let priority = 0;
    redis::cmd("ZADD")
        .arg(keys.queued_set())
        .arg(priority)
        .arg(&queued_job_id)
        .query::<()>(&mut conn)?;

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

    let current_jobs = inspector.get_current_jobs()?;

    // do the cleaning before asserts _just in case_
    #[rustfmt::skip]
    redis::pipe()
        .cmd("DEL").arg(keys.queued_set()).ignore()
        .cmd("DEL").arg(keys.running_set()).ignore()
        .cmd("DEL").arg(keys.scheduled_set()).ignore()
        .cmd("DEL").arg(keys.canceled_set()).ignore()
        .cmd("DEL").arg(keys.harvestable_set()).ignore()
        .query::<()>(&mut conn)?;

    assert_eq!(current_jobs.queued, vec![queued_job_id]);
    assert_eq!(current_jobs.running, vec![running_job_id]);
    assert_eq!(current_jobs.scheduled, vec![scheduled_job_id]);
    assert_eq!(current_jobs.canceled, vec![canceled_job_id]);
    assert_eq!(current_jobs.harvestable, vec![harvestable_job_id]);

    Ok(())
}
