use jono_core::{Context, Forum, JobStatus, JonoError, current_timestamp_ms, generate_job_id};

pub fn create_test_context() -> Context {
    // job id is random enough for now 🤷
    let random_topic = format!("test_ctx_{}", generate_job_id());
    Forum::new("redis://localhost:6380")
        .expect("Failed to connect to Redis")
        .topic(random_topic)
}

pub struct JobFixture {
    pub job_id: String,
    context: Context,
}

impl JobFixture {
    #[allow(dead_code)]
    pub async fn new(context: Context, status: JobStatus, score: i64) -> jono_core::Result<Self> {
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

        let mut conn = context.get_connection().await?;

        #[rustfmt::skip]
        let _: () = redis::pipe()
            .zadd(&set_key, &job_id, score)
            .hset(&metadata_key, "id", &job_id)
            .hset(&metadata_key, "payload", "{}")
            .hset(&metadata_key, "max_attempts", "1")
            .hset(&metadata_key, "initial_priority", "0")
            .hset(&metadata_key, "created_at", now.to_string())
            .hset(&metadata_key, "attempt_history", "[]")
            .hset(&metadata_key, "work_summary", "null")
            .hset(&metadata_key, "status", status.to_string())
            .query_async(&mut conn)
            .await?;

        let fixture = Self {
            job_id,
            context: context.clone(),
        };

        Ok(fixture)
    }

    fn clean_sync(&self) -> jono_core::Result<()> {
        let keys = self.context.keys();

        // this is a bit unorthodox, but this cleaning is done on Drop
        // which is synchronous; would need to figure out a nicer way
        // to do this in an async context without running into issue
        // of runtime inside runtime

        let client = redis::Client::open(self.context.forum().redis_url())?;
        let mut conn = client.get_connection()?;

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
        if let Err(err) = self.clean_sync() {
            eprintln!("Error during cleanup: {:?}", err);
        }
    }
}
