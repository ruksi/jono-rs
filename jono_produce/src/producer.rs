use crate::JobPlan;
use jono_core::*;
use redis::{Commands, Connection};
use tracing::info;

/// Interface for submitting jobs to Jono queues
pub struct Producer {
    context: Context,
}

impl Producer {
    pub fn with_context(context: Context) -> Self {
        Self { context }
    }

    /// Submit a new job to the queue
    pub fn submit_job(&self, job_plan: JobPlan) -> Result<String> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();
        let now = current_timestamp_ms();

        let job_id = generate_job_id();
        let metadata_key = keys.job_metadata_hash(&job_id);

        let _: () = redis::pipe()
            .hset(&metadata_key, "id", &job_id)
            .hset(
                &metadata_key,
                "payload",
                serde_json::to_string(&job_plan.get_payload())?,
            )
            .hset(
                &metadata_key,
                "max_attempts",
                job_plan.get_max_attempts().to_string(),
            )
            .hset(
                &metadata_key,
                "initial_priority",
                job_plan.get_priority().to_string(),
            )
            .hset(&metadata_key, "created_at", now.to_string())
            .hset(&metadata_key, "attempt_history", "[]")
            .hset(&metadata_key, "outcome", "null")
            .query(&mut conn)
            .map_err(Error::Redis)?;

        let scheduled_for = job_plan.get_scheduled_for();
        if scheduled_for > 0 && scheduled_for > now {
            let _: () = conn
                .zadd::<_, _, _, ()>(keys.scheduled_set(), &job_id, scheduled_for)
                .map_err(Error::Redis)?;

            info!(
                job_id = %job_id,
                scheduled_for = %job_plan.get_scheduled_for(),
                "Job scheduled for later execution"
            );
        } else {
            let _: () = conn
                .zadd::<_, _, _, ()>(keys.queued_set(), &job_id, job_plan.get_priority())
                .map_err(Error::Redis)?;

            info!(
                job_id = %job_id,
                priority = %job_plan.get_priority(),
                "Job sent to queue"
            );
        }

        Ok(job_id)
    }

    /// Cancel a job if it hasn't started processing yet
    pub fn cancel_job(&self, job_id: &str, grace_period_ms: i64) -> Result<bool> {
        let mut conn = self.get_connection()?;
        let now = current_timestamp_ms();
        let keys = self.context.keys();
        let metadata_key = keys.job_metadata_hash(job_id);

        let exists: bool = conn.exists(metadata_key).map_err(Error::Redis)?;
        if !exists {
            return Err(Error::NotFound(format!("Job {} not found", job_id)));
        }

        #[rustfmt::skip]
        let (removed_from_queued, removed_from_scheduled, last_heartbeat): (u32, u32, Option<i64>) =
            redis::pipe()
                .zrem(keys.queued_set(), job_id)
                .zrem(keys.scheduled_set(), job_id)
                .zscore(keys.running_set(), job_id)
                .query(&mut conn)
                .map_err(Error::Redis)?;

        if last_heartbeat.is_some() {
            let grace_end = now + grace_period_ms;
            let _: () = conn
                .zadd::<_, _, _, ()>(keys.canceled_set(), job_id, grace_end)
                .map_err(Error::Redis)?;
            info!(
                job_id = %job_id,
                grace_end = %grace_end,
                "Running job marked for cancellation with grace period"
            );
            return Ok(true);
        }

        if removed_from_queued > 0 || removed_from_scheduled > 0 {
            let _: () = conn
                .zadd::<_, _, _, ()>(keys.canceled_set(), job_id, now)
                .map_err(Error::Redis)?;
            info!(job_id = %job_id, "Job canceled successfully");
            return Ok(true);
        }

        info!(job_id = %job_id, "Job not found in any active queue");
        Ok(false)
    }

    pub fn clean_job(&self, job_id: &str) -> Result<bool> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        #[rustfmt::skip]
        let (metadata_deleted,): (u32,) = redis::pipe()
            .zrem(keys.queued_set(), job_id).ignore()
            .zrem(keys.running_set(), job_id).ignore()
            .zrem(keys.scheduled_set(), job_id).ignore()
            .zrem(keys.canceled_set(), job_id).ignore()
            .del(keys.job_metadata_hash(job_id))
            .query(&mut conn)
            .map_err(Error::Redis)?;

        Ok(metadata_deleted > 0)
    }

    fn get_connection(&self) -> Result<Connection> {
        self.context.get_connection()
    }
}
