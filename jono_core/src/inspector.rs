//! Provides functionality to query details jobs in the Jono queue system.

use redis::{Commands, Connection};
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

    pub fn job_exists(&self, job_id: &str) -> Result<bool> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        let exists: bool = conn.exists(keys.job_metadata_hash(job_id))?;

        Ok(exists)
    }

    pub fn job_is_canceled(&self, job_id: &str) -> Result<bool> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        let score: Option<i64> = conn.zscore(keys.canceled_set(), job_id)?;

        Ok(score.is_some())
    }

    pub fn get_job_status(&self, job_id: &str) -> Result<JobStatus> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        let exists: bool = conn.exists(keys.job_metadata_hash(job_id))?;
        if !exists {
            return Err(JonoError::JobNotFound(job_id.to_string()));
        }

        let in_running_set: bool = conn
            .zscore::<_, _, Option<i64>>(keys.running_set(), job_id)?
            .is_some();
        if in_running_set {
            return Ok(JobStatus::Running);
        }

        let in_queued_set: bool = conn
            .zscore::<_, _, Option<i64>>(keys.queued_set(), job_id)?
            .is_some();
        if in_queued_set {
            return Ok(JobStatus::Queued);
        }

        let in_scheduled_set: bool = conn
            .zscore::<_, _, Option<i64>>(keys.scheduled_set(), job_id)?
            .is_some();
        if in_scheduled_set {
            return Ok(JobStatus::Scheduled);
        }

        let in_canceled_set: bool = conn
            .zscore::<_, _, Option<i64>>(keys.canceled_set(), job_id)?
            .is_some();
        if in_canceled_set {
            return Ok(JobStatus::Canceled);
        }

        let has_completed_at_field: Option<String> =
            conn.hget(keys.job_metadata_hash(job_id), "completed_at")?;
        if has_completed_at_field.is_some() {
            return Ok(JobStatus::Harvestable);
        }

        let attempt_history: Option<String> =
            conn.hget(keys.job_metadata_hash(job_id), "attempt_history")?;
        if let Some(history_str) = attempt_history {
            let history: Value = serde_json::from_str(&history_str)?;

            if let Value::Array(attempts) = history {
                let max_attempts: u32 = conn
                    .hget::<_, _, Option<String>>(keys.job_metadata_hash(job_id), "max_attempts")?
                    .and_then(|m| m.parse().ok())
                    .unwrap_or(3);

                if attempts.len() >= max_attempts as usize {
                    return Ok(JobStatus::Failed);
                }
            }
        }

        Ok(JobStatus::Failed)
    }

    pub fn get_job_metadata(&self, job_id: &str) -> Result<JobMetadata> {
        let mut conn = self.get_connection()?;
        let keys = self.context.keys();

        let metadata_key = keys.job_metadata_hash(job_id);

        let exists: bool = conn.exists(&metadata_key)?;
        if !exists {
            return Err(JonoError::JobNotFound(job_id.to_string()));
        }

        let hash: HashMap<String, String> = conn.hgetall(&metadata_key)?;
        JobMetadata::from_hash(hash)
    }

    /// Get the current state of jobs by ID in the Jono system, optionally filtered by criteria in JobFilter.
    pub fn get_status_to_job_ids(&self, filter: JobFilter) -> Result<MapStatusToJobId> {
        let keys = self.context.keys();

        let states_to_fetch = match filter.states.as_deref() {
            Some(specific_states) => specific_states,
            None => &[
                JobStatus::Queued,
                JobStatus::Running,
                JobStatus::Scheduled,
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
                JobStatus::Queued => {
                    pipe.zrange(keys.queued_set(), 0, -1);
                    status_to_index.push(JobStatus::Queued);
                }
                JobStatus::Running => {
                    pipe.zrange(keys.running_set(), 0, -1);
                    status_to_index.push(JobStatus::Running);
                }
                JobStatus::Scheduled => {
                    pipe.zrange(keys.scheduled_set(), 0, -1);
                    status_to_index.push(JobStatus::Scheduled);
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

        let mut conn = self.get_connection()?;
        let mut result: Vec<Vec<String>> = pipe.query(&mut conn)?;

        for (i, status) in status_to_index.iter().enumerate() {
            if i < result.len() {
                match status {
                    JobStatus::Queued => map.queued = std::mem::take(&mut result[i]),
                    JobStatus::Running => map.running = std::mem::take(&mut result[i]),
                    JobStatus::Scheduled => map.scheduled = std::mem::take(&mut result[i]),
                    JobStatus::Canceled => map.canceled = std::mem::take(&mut result[i]),
                    JobStatus::Harvestable => map.harvestable = std::mem::take(&mut result[i]),
                    JobStatus::Failed => {} // TODO: wait for the deadletter implementation
                }
            }
        }

        Ok(map)
    }

    pub fn get_status_to_job_metadata(&self, filter: JobFilter) -> Result<MapStatusToJobMetadata> {
        let status_to_job_ids = self.get_status_to_job_ids(filter.clone())?;

        // TODO: this might be quite heavy if there are many jobs... should I add some iterable/stream interface too?

        let process = |job_ids: &[String]| -> Vec<JobMetadata> {
            if job_ids.is_empty() {
                return Vec::new();
            }

            // TODO: use Redis pipelines? they should be on the same shard
            //       as they have hash tags by topic...

            let mut metadatas: Vec<JobMetadata> = Vec::with_capacity(job_ids.len());
            for id in job_ids {
                if let Ok(md) = self.get_job_metadata(id) {
                    metadatas.push(md);
                }
            }
            metadatas
        };

        let Some(states) = filter.states.as_deref() else {
            return Ok(MapStatusToJobMetadata {
                queued: process(&status_to_job_ids.queued),
                running: process(&status_to_job_ids.running),
                scheduled: process(&status_to_job_ids.scheduled),
                canceled: process(&status_to_job_ids.canceled),
                harvestable: process(&status_to_job_ids.harvestable),
            });
        };

        let mut result = MapStatusToJobMetadata::default();

        for state in states {
            use JobStatus::*;
            match state {
                Queued => result.queued = process(&status_to_job_ids.queued),
                Running => result.running = process(&status_to_job_ids.running),
                Scheduled => result.scheduled = process(&status_to_job_ids.scheduled),
                Canceled => result.canceled = process(&status_to_job_ids.canceled),
                Harvestable => result.harvestable = process(&status_to_job_ids.harvestable),
                Failed => {} // TODO: wait for the deadletter implementation
            }
        }

        Ok(result)
    }

    fn get_connection(&self) -> Result<Connection> {
        self.context.get_connection()
    }
}

#[derive(Debug, Clone, Default)]
pub struct JobFilter {
    /// Optional list of job states to filter by, None disables this filter
    pub states: Option<Vec<JobStatus>>,
}

#[derive(Debug, Clone, Default)]
pub struct MapStatusToJobId {
    /// Jobs in waiting to be processed
    pub queued: Vec<String>,
    /// Jobs currently being processed by workers
    pub running: Vec<String>,
    /// Jobs scheduled to run at a future time
    pub scheduled: Vec<String>,
    /// Jobs that have been explicitly canceled
    pub canceled: Vec<String>,
    /// Jobs that are ready to be harvested; completed but not post-processed
    pub harvestable: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MapStatusToJobMetadata {
    /// Jobs in waiting to be processed
    pub queued: Vec<JobMetadata>,
    /// Jobs currently being processed by workers
    pub running: Vec<JobMetadata>,
    /// Jobs scheduled to run at a future time
    pub scheduled: Vec<JobMetadata>,
    /// Jobs that have been explicitly canceled
    pub canceled: Vec<JobMetadata>,
    /// Jobs that are ready to be harvested; completed but not post-processed
    pub harvestable: Vec<JobMetadata>,
}
