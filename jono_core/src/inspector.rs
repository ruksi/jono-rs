//! Provides functionality to query details jobs in the Jono queue system.

use redis::{Commands, Connection};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;

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
            return Ok(JobStatus::Completed);
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

    /// Get the current state of all jobs in the Jono system.
    pub fn get_current_jobs(&self) -> Result<CurrentJobs> {
        let keys = self.context.keys();
        let mut conn = self.get_connection()?;

        let as_text: String = GET_CURRENT_JOBS_SCRIPT
            .key(keys.queued_set())
            .key(keys.running_set())
            .key(keys.scheduled_set())
            .key(keys.canceled_set())
            .key(keys.harvestable_set())
            .invoke(&mut conn)?;

        let as_json: Value = serde_json::from_str(&as_text)?;
        let current_jobs = CurrentJobs::from_json_value(&as_json);

        Ok(current_jobs)
    }

    fn get_connection(&self) -> Result<Connection> {
        self.context.get_connection()
    }
}

static GET_CURRENT_JOBS_SCRIPT: LazyLock<redis::Script> = LazyLock::new(|| {
    redis::Script::new(
        // language=Lua
        r#"
            local result = {}
            result['queued'] = redis.call('ZRANGE', KEYS[1], 0, -1)
            result['running'] = redis.call('ZRANGE', KEYS[2], 0, -1)
            result['scheduled'] = redis.call('ZRANGE', KEYS[3], 0, -1)
            result['canceled'] = redis.call('ZRANGE', KEYS[4], 0, -1)
            result['harvestable'] = redis.call('ZRANGE', KEYS[5], 0, -1)
            return cjson.encode(result)
        "#,
    )
});

/// Represents the current state of all jobs in the Jono system.
#[derive(Debug, Clone)]
pub struct CurrentJobs {
    /// Jobs in the queued set waiting to be processed
    pub queued: Vec<String>,
    /// Jobs currently being processed by workers
    pub running: Vec<String>,
    /// Jobs scheduled to run at a future time
    pub scheduled: Vec<String>,
    /// Jobs that have been explicitly canceled
    pub canceled: Vec<String>,
    /// Jobs that are ready to be harvested (completed jobs)
    pub harvestable: Vec<String>,
}

impl CurrentJobs {
    pub fn from_json_value(value: &Value) -> Self {
        Self {
            queued: extract_string_array(&value, "queued"),
            running: extract_string_array(&value, "running"),
            scheduled: extract_string_array(&value, "scheduled"),
            canceled: extract_string_array(&value, "canceled"),
            harvestable: extract_string_array(&value, "harvestable"),
        }
    }
}

fn extract_string_array(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}
