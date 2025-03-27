use jono_core::*;
use redis::{Commands, Connection};

/// Interface for post-processing completed jobs on Jono queues
pub struct Harvester {
    context: Context,
}

impl Harvester {
    pub fn with_context(context: Context) -> Self {
        Self { context }
    }

    pub fn harvest(&self, limit: usize) -> Result<Vec<JobMetadata>> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        let job_ids: Vec<String> = conn
            .zpopmin(keys.harvestable_set(), limit as isize)
            .map_err(Error::Redis)?;

        let inspector = Inspector::with_context(self.context.clone());

        let mut results = Vec::with_capacity(job_ids.len());
        for job_id in job_ids {
            if let Ok(metadata) = inspector.get_job_metadata(&job_id) {
                results.push(metadata);
            }
        }

        Ok(results)
    }

    pub fn clean_harvestable_set(&self) -> Result<usize> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();
        let now = current_timestamp_ms();

        let removed: usize = conn
            .zrembyscore(keys.harvestable_set(), "-inf", (now - 1).to_string())
            .map_err(Error::Redis)?;

        Ok(removed)
    }

    fn get_connection(&self) -> Result<Connection> {
        self.context.get_connection()
    }
}
