use jono_core::*;
use redis::AsyncCommands;

/// Interface for post-processing completed jobs on Jono queues
pub struct Harvester {
    context: Context,
}

impl Harvester {
    pub fn with_context(context: Context) -> Self {
        Self { context }
    }

    /// Harvest jobs that have been completed and are ready for post-processing (just-once)
    pub async fn harvest(&self, limit: usize) -> Result<Vec<JobMetadata>> {
        let mut conn = self.get_connection().await?;
        let keys = self.context.keys();

        let job_ids: Vec<String> = conn.zpopmin(keys.harvestable_set(), limit as isize).await?;

        let inspector = Inspector::with_context(self.context.clone());

        let mut results = Vec::with_capacity(job_ids.len());
        for job_id in job_ids {
            if let Ok(metadata) = inspector.get_job_metadata(&job_id).await {
                results.push(metadata);
            }
        }

        Ok(results)
    }

    /// Clean up expired entries from the harvestable set (they weren't post-processed)
    pub async fn clean_expired_harvest(&self) -> Result<usize> {
        let mut conn = self.get_connection().await?;
        let keys = self.context.keys();
        let now = current_timestamp_ms();

        let removed: usize = conn
            .zrembyscore(keys.harvestable_set(), "-inf", (now - 1).to_string())
            .await?;

        Ok(removed)
    }

    async fn get_connection(&self) -> Result<impl redis::aio::ConnectionLike> {
        self.context.get_connection().await
    }
}
