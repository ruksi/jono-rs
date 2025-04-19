//! Provides functionality to query details jobs in the Jono queue system.

use redis::AsyncCommands;
use serde_json::Value;
use std::collections::HashMap;

use crate::{Context, JobMetadata, JobStatus, JonoError, Result};

/// Interface for querying job details
pub struct Inspector {
    context: Context,
}

impl Inspector {
    pub fn with_context(context: Context) -> Self {
        Self { context }
    }

    pub async fn job_exists(&self, job_id: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let keys = self.context.keys();

        let exists: bool = conn.exists(keys.job_metadata_hash(job_id)).await?;

        Ok(exists)
    }

    pub async fn job_is_canceled(&self, job_id: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let keys = self.context.keys();

        let score: Option<i64> = conn.zscore(keys.canceled_set(), job_id).await?;

        Ok(score.is_some())
    }

    pub async fn get_job_status(&self, job_id: &str) -> Result<JobStatus> {
        let mut conn = self.get_connection().await?;
        let keys = self.context.keys();

        let exists: bool = conn.exists(keys.job_metadata_hash(job_id)).await?;
        if !exists {
            return Err(JonoError::JobNotFound(job_id.to_string()));
        }

        let in_running_set: bool = conn
            .zscore::<_, _, Option<i64>>(keys.running_set(), job_id)
            .await?
            .is_some();
        if in_running_set {
            return Ok(JobStatus::Running);
        }

        let in_queued_set: bool = conn
            .zscore::<_, _, Option<i64>>(keys.queued_set(), job_id)
            .await?
            .is_some();
        if in_queued_set {
            return Ok(JobStatus::Queued);
        }

        let in_postponed_set: bool = conn
            .zscore::<_, _, Option<i64>>(keys.postponed_set(), job_id)
            .await?
            .is_some();
        if in_postponed_set {
            return Ok(JobStatus::Postponed);
        }

        let in_canceled_set: bool = conn
            .zscore::<_, _, Option<i64>>(keys.canceled_set(), job_id)
            .await?
            .is_some();
        if in_canceled_set {
            return Ok(JobStatus::Canceled);
        }

        let has_completed_at_field: Option<String> = conn
            .hget(keys.job_metadata_hash(job_id), "completed_at")
            .await?;
        if has_completed_at_field.is_some() {
            return Ok(JobStatus::Harvestable);
        }

        let attempt_history: Option<String> = conn
            .hget(keys.job_metadata_hash(job_id), "attempt_history")
            .await?;
        if let Some(history_str) = attempt_history {
            let history: Value = serde_json::from_str(&history_str)?;

            if let Value::Array(attempts) = history {
                let max_attempts: u32 = conn
                    .hget::<_, _, Option<String>>(keys.job_metadata_hash(job_id), "max_attempts")
                    .await?
                    .and_then(|m| m.parse().ok())
                    .unwrap_or(3);

                if attempts.len() >= max_attempts as usize {
                    return Ok(JobStatus::Failed);
                }
            }
        }

        Ok(JobStatus::Failed)
    }

    pub async fn get_job_metadata(&self, job_id: &str) -> Result<JobMetadata> {
        let mut conn = self.get_connection().await?;
        let keys = self.context.keys();

        let metadata_key = keys.job_metadata_hash(job_id);

        let exists: bool = conn.exists(&metadata_key).await?;
        if !exists {
            return Err(JonoError::JobNotFound(job_id.to_string()));
        }

        let hash: HashMap<String, String> = conn.hgetall(&metadata_key).await?;
        JobMetadata::from_hash(hash)
    }

    /// Get the current state of jobs by ID in the Jono system, optionally filtered by criteria in JobFilter.
    pub async fn get_status_to_job_ids(&self, filter: JobFilter) -> Result<MapStatusToJobId> {
        let keys = self.context.keys();

        let states_to_fetch = match filter.states.as_deref() {
            Some(specific_states) => specific_states,
            None => &[
                JobStatus::Postponed,
                JobStatus::Queued,
                JobStatus::Running,
                JobStatus::Canceled,
                JobStatus::Harvestable,
            ],
        };

        let mut map = MapStatusToJobId::default();
        if states_to_fetch.is_empty() {
            return Ok(map);
        }

        let mut pipe = redis::pipe();
        pipe.atomic();

        let mut status_to_index = Vec::new();

        for status in states_to_fetch {
            match status {
                JobStatus::Postponed => {
                    pipe.zrange(keys.postponed_set(), 0, -1);
                    status_to_index.push(JobStatus::Postponed);
                }
                JobStatus::Queued => {
                    pipe.zrange(keys.queued_set(), 0, -1);
                    status_to_index.push(JobStatus::Queued);
                }
                JobStatus::Running => {
                    pipe.zrange(keys.running_set(), 0, -1);
                    status_to_index.push(JobStatus::Running);
                }
                JobStatus::Canceled => {
                    pipe.zrange(keys.canceled_set(), 0, -1);
                    status_to_index.push(JobStatus::Canceled);
                }
                JobStatus::Harvestable => {
                    pipe.zrange(keys.harvestable_set(), 0, -1);
                    status_to_index.push(JobStatus::Harvestable);
                }
                JobStatus::Failed => {} // TODO: wait for the deadletter implementation
            }
        }

        if status_to_index.is_empty() {
            return Ok(map);
        }

        let mut conn = self.get_connection().await?;
        let mut result: Vec<Vec<String>> = pipe.query_async(&mut conn).await?;

        for (i, status) in status_to_index.iter().enumerate() {
            if i < result.len() {
                match status {
                    JobStatus::Postponed => map.postponed = std::mem::take(&mut result[i]),
                    JobStatus::Queued => map.queued = std::mem::take(&mut result[i]),
                    JobStatus::Running => map.running = std::mem::take(&mut result[i]),
                    JobStatus::Canceled => map.canceled = std::mem::take(&mut result[i]),
                    JobStatus::Harvestable => map.harvestable = std::mem::take(&mut result[i]),
                    JobStatus::Failed => {} // TODO: wait for the deadletter implementation
                }
            }
        }

        Ok(map)
    }

    pub async fn get_status_to_job_metadata(
        &self,
        filter: JobFilter,
    ) -> Result<MapStatusToJobMetadata> {
        let status_to_job_ids = self.get_status_to_job_ids(filter.clone()).await?;

        // TODO: this might be quite heavy if there are many jobs... should I add some iterable/stream interface too?

        let process = async |job_ids: &[String]| -> Vec<JobMetadata> {
            if job_ids.is_empty() {
                return Vec::new();
            }

            // TODO: use Redis pipelines? they should be on the same shard
            //       as they have hash tags by topic...

            let mut metadatas: Vec<JobMetadata> = Vec::with_capacity(job_ids.len());
            for id in job_ids {
                if let Ok(md) = self.get_job_metadata(id).await {
                    metadatas.push(md);
                }
            }
            metadatas
        };

        let Some(states) = filter.states.as_deref() else {
            return Ok(MapStatusToJobMetadata {
                postponed: process(&status_to_job_ids.postponed).await,
                queued: process(&status_to_job_ids.queued).await,
                running: process(&status_to_job_ids.running).await,
                canceled: process(&status_to_job_ids.canceled).await,
                harvestable: process(&status_to_job_ids.harvestable).await,
            });
        };

        let mut result = MapStatusToJobMetadata::default();

        for state in states {
            use JobStatus::*;
            match state {
                Postponed => result.postponed = process(&status_to_job_ids.postponed).await,
                Queued => result.queued = process(&status_to_job_ids.queued).await,
                Running => result.running = process(&status_to_job_ids.running).await,
                Canceled => result.canceled = process(&status_to_job_ids.canceled).await,
                Harvestable => result.harvestable = process(&status_to_job_ids.harvestable).await,
                Failed => {} // TODO: wait for the deadletter implementation
            }
        }

        Ok(result)
    }

    async fn get_connection(&self) -> Result<impl redis::aio::ConnectionLike> {
        self.context.get_connection().await
    }
}

#[derive(Debug, Clone, Default)]
pub struct JobFilter {
    /// Optional list of job states to filter by, None disables this filter
    pub states: Option<Vec<JobStatus>>,
}

#[derive(Debug, Clone, Default)]
pub struct MapStatusToJobId {
    /// Jobs postponed for running at a future time
    pub postponed: Vec<String>,
    /// Jobs in waiting to be processed
    pub queued: Vec<String>,
    /// Jobs currently being processed by workers
    pub running: Vec<String>,
    /// Jobs that have been explicitly canceled
    pub canceled: Vec<String>,
    /// Jobs that are ready to be harvested; completed but not post-processed
    pub harvestable: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MapStatusToJobMetadata {
    /// Jobs postponed for running at a future time
    pub postponed: Vec<JobMetadata>,
    /// Jobs in waiting to be processed
    pub queued: Vec<JobMetadata>,
    /// Jobs currently being processed by workers
    pub running: Vec<JobMetadata>,
    /// Jobs that have been explicitly canceled
    pub canceled: Vec<JobMetadata>,
    /// Jobs that are ready to be harvested; completed but not post-processed
    pub harvestable: Vec<JobMetadata>,
}
