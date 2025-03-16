use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

/// All the possible states that a job can be in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// The job is queued and waiting to be processed.
    Queued,
    /// The job is currently being processed by a worker.
    Running,
    /// The job has been completed successfully.
    Completed,
    /// The job has failed to complete after specified retries.
    Failed,
    /// The job has been specifically canceled.
    Canceled,
    /// The job is scheduled to run at a future time.
    Scheduled,
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            JobStatus::Queued => "queued".to_string(),
            JobStatus::Running => "running".to_string(),
            JobStatus::Completed => "completed".to_string(),
            JobStatus::Failed => "failed".to_string(),
            JobStatus::Canceled => "canceled".to_string(),
            JobStatus::Scheduled => "scheduled".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl FromStr for JobStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(JobStatus::Queued),
            "running" => Ok(JobStatus::Running),
            "completed" => Ok(JobStatus::Completed),
            "failed" => Ok(JobStatus::Failed),
            "canceled" => Ok(JobStatus::Canceled),
            "scheduled" => Ok(JobStatus::Scheduled),
            _ => Err(format!("Unknown job status: {}", s)),
        }
    }
}
