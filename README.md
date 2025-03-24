# 🚥 Jono

**Jono** is a priority queue based on Redis sorted sets and ULIDs.

+ [Redis sorted sets](https://redis.io/docs/latest/develop/data-types/sorted-sets/) provide:
    + priority ordering through member score sorting (lower score = higher priority)
    + lexicographical ordering of equally scored members
+ [ULIDs](https://github.com/ulid/spec) are lexicographically ordered based on generation time
  by definition so they can be used as member values in Redis sorted sets for FIFO ordering

These together allow for a simple priority queue where
[`ZPOPMIN`](https://redis.io/docs/latest/commands/zpopmin/) can be used to
atomically get the next job to be processed.

## Usage

```rust
use jono::prelude::*;
use jono_core::get_redis_url;
use serde_json::json;

pub fn main() -> Result<()> {
    // creating the context, should be done on both producer and consumer code
    // checks "JONO_REDIS_URL", then "REDIS_URL" and defaults to given fallback
    let redis_url = get_redis_url("redis://localhost:6379");
    let forum = Forum::new(&redis_url)?;
    let context = forum.topic("workwork");

    // to submit new jobs:
    let producer = Producer::with_context(context.clone());
    let job_id = JobPlan::new()
        .payload(json!({"my-key": "my-value"}))
        .dispatch(&producer)?;

    // to process jobs (worker code is further below):
    let consumer = Consumer::with_context(context.clone(), NoopWorker);
    let outcome = consumer.run_next()?;
    match outcome {
        Some(Outcome::Success(_)) => {
            todo!("You want to do something on the worker post-job?");
        }
        Some(Outcome::Failure(_)) => {
            todo!("... or specifically on failure?");
        }
        None => {
            todo!("... or do something if nothing was found in the queue?");
        }
    }

    // TODO: add `jono_collect` example
}

struct NoopWorker;

impl Worker for NoopWorker {
    fn process(&self, _: &Workload) -> Result<Outcome> {
        Ok(Outcome::Success(Some(json!({"processed": true}))))
    }
}
```