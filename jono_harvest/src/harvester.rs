use crate::{HarvestConfig, ReapSummary, Reaper, Reapload};
use jono_core::*;
use redis::AsyncCommands;
use std::thread;

/// Interface for post-processing completed jobs on Jono queues
pub struct Harvester<R: Reaper> {
    context: Context,
    config: HarvestConfig,
    reaper: R,
}

impl<R: Reaper> Harvester<R> {
    pub fn with_context(context: Context, reaper: R) -> Self {
        Self {
            context,
            config: HarvestConfig::default(),
            reaper,
        }
    }

    pub fn with_config(mut self, config: HarvestConfig) -> Self {
        self.config = config;
        self
    }

    pub async fn reap(&self) -> Result<()> {
        let mut consecutive_errors = 0;

        loop {
            match self.reap_next_batch().await {
                Ok(reap_summaries) if !reap_summaries.is_empty() => {
                    consecutive_errors = 0;
                }
                Ok(_) => {
                    // No jobs were harvested
                    thread::sleep(self.config.get_polling_interval());
                }
                Err(e) => {
                    consecutive_errors += 1;
                    eprintln!("Error processing harvested jobs: {}", e);

                    if consecutive_errors >= self.config.get_max_consecutive_errors() {
                        return Err(JonoError::TooManyErrors(consecutive_errors));
                    }
                    thread::sleep(self.config.get_polling_interval());
                }
            }
        }
    }

    pub async fn reap_next_batch(&self) -> Result<Vec<ReapSummary>> {
        let harvested = self.harvest(self.config.get_batch_size()).await?;
        if harvested.is_empty() {
            return Ok(Vec::new());
        }

        let mut reap_summaries = Vec::with_capacity(harvested.len());

        for job_metadata in harvested {
            let reapload = Reapload::from_metadata(job_metadata);
            match self.reaper.reap(&reapload).await {
                Ok(summary) => reap_summaries.push(summary),
                Err(e) => return Err(e),
            }
        }

        Ok(reap_summaries)
    }

    /// Harvest jobs that have been completed and are ready for post-processing (just-once)
    pub async fn harvest(&self, limit: usize) -> Result<Vec<JobMetadata>> {
        let mut conn = self.get_connection().await?;
        let keys = self.context.keys();

        let job_ids: Vec<String> = conn.zpopmin(keys.completed_set(), limit as isize).await?;

        let inspector = Inspector::with_context(self.context.clone());

        let mut results = Vec::with_capacity(job_ids.len());
        for job_id in job_ids {
            if let Ok(metadata) = inspector.get_job_metadata(&job_id).await {
                results.push(metadata);
            }
        }

        Ok(results)
    }

    /// Clean up expired entries from the completed set (they weren't post-processed)
    pub async fn clean_expired_completed(&self) -> Result<usize> {
        let mut conn = self.get_connection().await?;
        let keys = self.context.keys();
        let now = current_timestamp_ms();

        let removed: usize = conn
            .zrembyscore(keys.completed_set(), "-inf", (now - 1).to_string())
            .await?;

        Ok(removed)
    }

    async fn get_connection(&self) -> Result<impl redis::aio::ConnectionLike> {
        self.context.get_connection().await
    }
}
