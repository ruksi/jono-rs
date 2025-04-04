use jono_core::{Context, Forum, JobStatus, JonoError, current_timestamp_ms, generate_job_id};

pub fn create_test_context(topic: impl ToString) -> Context {
    Forum::new("redis://localhost:6380")
        .expect("Failed to connect to Redis")
        .topic(topic)
}

pub struct JobFixture {
    pub job_id: String,
    context: Context,
}

impl JobFixture {
    #[allow(dead_code)]
    pub fn new(context: Context, status: JobStatus, score: i64) -> jono_core::Result<Self> {
        let keys = context.keys();
        let now = current_timestamp_ms();

        let job_id = generate_job_id();
        let metadata_key = keys.job_metadata_hash(&job_id);

        let set_key = match status {
            JobStatus::Queued => keys.queued_set(),
            JobStatus::Running => keys.running_set(),
            JobStatus::Scheduled => keys.scheduled_set(),
            JobStatus::Canceled => keys.canceled_set(),
            JobStatus::Harvestable => keys.harvestable_set(),
            _ => {
                return Err(JonoError::InvalidJob(
                    "Cannot directly create this job type".to_string(),
                ));
            }
        };

        let mut conn = context.get_connection()?;

        #[rustfmt::skip]
        let _: () = redis::pipe()
            .zadd(&set_key, &job_id, score)
            .hset(&metadata_key, "id", &job_id)
            .hset(&metadata_key, "payload", "{}")
            .hset(&metadata_key, "max_attempts", "1")
            .hset(&metadata_key, "initial_priority", "0")
            .hset(&metadata_key, "created_at", now.to_string())
            .hset(&metadata_key, "attempt_history", "[]")
            .hset(&metadata_key, "outcome", "null")
            .hset(&metadata_key, "status", status.to_string())
            .query(&mut conn)?;

        Ok(Self { job_id, context })
    }

    fn clean(&self) -> jono_core::Result<()> {
        let keys = self.context.keys();
        let mut conn = self.context.get_connection()?;

        #[rustfmt::skip]
        let _: () = redis::pipe()
            .cmd("DEL").arg(keys.queued_set())
            .cmd("DEL").arg(keys.running_set())
            .cmd("DEL").arg(keys.scheduled_set())
            .cmd("DEL").arg(keys.canceled_set())
            .cmd("DEL").arg(keys.harvestable_set())
            .cmd("DEL").arg(keys.job_metadata_hash(&self.job_id))
            .query(&mut conn)?;

        Ok(())
    }
}

impl Drop for JobFixture {
    fn drop(&mut self) {
        if let Err(e) = self.clean() {
            eprintln!("Failed to clean up job fixture: {}", e);
        }
    }
}
