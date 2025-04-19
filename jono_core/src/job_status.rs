use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

/// All the possible states that a job can be in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JobStatus {
    /// The job is postponed to run at a future time.
    Postponed,
    /// The job is queued and waiting to be processed.
    Queued,
    /// The job is currently being processed by a worker.
    Started,
    /// The job has been completed successfully.
    Harvestable,
    /// The job has failed to complete after specified retries.
    Failed,
    /// The job has been specifically canceled.
    Aborted,
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            JobStatus::Postponed => "postponed".to_string(),
            JobStatus::Queued => "queued".to_string(),
            JobStatus::Started => "started".to_string(),
            JobStatus::Harvestable => "harvestable".to_string(),
            JobStatus::Failed => "failed".to_string(),
            JobStatus::Aborted => "aborted".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl FromStr for JobStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "postponed" => Ok(JobStatus::Postponed),
            "queued" => Ok(JobStatus::Queued),
            "started" => Ok(JobStatus::Started),
            "harvestable" => Ok(JobStatus::Harvestable),
            "failed" => Ok(JobStatus::Failed),
            "aborted" => Ok(JobStatus::Aborted),
            _ => Err(format!("Unknown job status: {}", s)),
        }
    }
}
