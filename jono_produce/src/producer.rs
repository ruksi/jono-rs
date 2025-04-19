use crate::JobPlan;
use jono_core::*;
use redis::AsyncCommands;
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
    pub async fn submit_job(&self, job_plan: JobPlan) -> Result<String> {
        let mut conn = self.get_connection().await?;
        let keys = self.context.keys();
        let now = current_timestamp_ms();

        let job_id = generate_job_id();
        let metadata_key = keys.job_metadata_hash(&job_id);

        let _: () = redis::pipe()
            .atomic()
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
            .hset(&metadata_key, "work_summary", "null")
            .query_async(&mut conn)
            .await?;

        let postponed_to = job_plan.get_postponed_to();
        if postponed_to > 0 && postponed_to > now {
            let _: () = conn
                .zadd::<_, _, _, ()>(keys.postponed_set(), &job_id, postponed_to)
                .await?;

            info!(
                job_id = %job_id,
                postponed_to = %job_plan.get_postponed_to(),
                "Job postponed for later execution"
            );
        } else {
            let _: () = conn
                .zadd::<_, _, _, ()>(keys.queued_set(), &job_id, job_plan.get_priority())
                .await?;

            info!(
                job_id = %job_id,
                priority = %job_plan.get_priority(),
                "Job sent to queue"
            );
        }

        Ok(job_id)
    }

    /// Cancel a job if it hasn't started processing yet
    pub async fn cancel_job(&self, job_id: &str, grace_period_ms: i64) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let now = current_timestamp_ms();
        let keys = self.context.keys();
        let metadata_key = keys.job_metadata_hash(job_id);

        let exists: bool = conn.exists(metadata_key).await?;
        if !exists {
            return Err(JonoError::JobNotFound(job_id.to_string()));
        }

        #[rustfmt::skip]
        let (removed_from_postponed, removed_from_queued, last_heartbeat): (u32, u32, Option<i64>) =
            redis::pipe()
                .atomic()
                .zrem(keys.postponed_set(), job_id)
                .zrem(keys.queued_set(), job_id)
                .zscore(keys.started_set(), job_id)
                .query_async(&mut conn)
                .await?;

        if last_heartbeat.is_some() {
            let grace_end = now + grace_period_ms;
            let _: () = conn
                .zadd::<_, _, _, ()>(keys.aborted_set(), job_id, grace_end)
                .await?;
            info!(
                job_id = %job_id,
                grace_end = %grace_end,
                "Running job marked for cancellation with grace period"
            );
            return Ok(true);
        }

        if removed_from_queued > 0 || removed_from_postponed > 0 {
            let _: () = conn
                .zadd::<_, _, _, ()>(keys.aborted_set(), job_id, now)
                .await?;
            info!(job_id = %job_id, "Job canceled successfully");
            return Ok(true);
        }

        info!(job_id = %job_id, "Job not found in any active queue");
        Ok(false)
    }

    pub async fn clean_job(&self, job_id: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let keys = self.context.keys();

        #[rustfmt::skip]
        let (metadata_deleted,): (u32,) = redis::pipe()
            .atomic()
            .zrem(keys.postponed_set(), job_id).ignore()
            .zrem(keys.queued_set(), job_id).ignore()
            .zrem(keys.started_set(), job_id).ignore()
            .zrem(keys.aborted_set(), job_id).ignore()
            .zrem(keys.harvestable_set(), job_id).ignore()
            .del(keys.job_metadata_hash(job_id))
            .query_async(&mut conn)
            .await?;

        Ok(metadata_deleted > 0)
    }

    async fn get_connection(&self) -> Result<impl redis::aio::ConnectionLike> {
        self.context.get_connection().await
    }
}
