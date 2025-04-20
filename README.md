# 🚥 Jono

**Jono** is a priority queue based on Redis sorted sets and ULIDs.

+ [Redis sorted sets](https://redis.io/docs/latest/develop/data-types/sorted-sets/) provide:
    + priority ordering through member score sorting (lower score = higher priority)
    + lexicographical ordering of equally scored members
+ [ULIDs](https://github.com/ulid/spec) are lexicographically ordered based on generation time
  by definition so they can be used as member values in Redis sorted sets for FIFO ordering

These together allow for a priority queue where [`ZPOPMIN`](https://redis.io/docs/latest/commands/zpopmin/) 
is used to get the next job to be processed.

## Usage

```toml
[dependencies]
jono = "0.1.6-rc.7"
# or, for minimal stuff:
jono = { version = "0.1.6-rc.7", default-features = false, features = ["produce", "runtime-tokio", "tls-none"] }
```

```rust
use jono::prelude::*;
use jono_core::Result;
use serde_json::json;

pub async fn codebase_1() -> Result<()> { 
    // tries "JONO_REDIS_URL" > "REDIS_URL" > Err
    let context = Context::try_from_env("work-work")?;

    // to submit new jobs:
    let producer = Producer::with_context(context);
    let job_id = JobPlan::new()
        .payload(json!({"my-key": "my-value"}))
        .submit(&producer)
        .await?;
}
```

```toml
[dependencies]
jono = "0.1.6-rc.7"
# or, for minimal stuff:
jono = { version = "0.1.6-rc.7", default-features = false, features = ["consume", "runtime-tokio", "tls-none"] }
```

```rust
use jono::prelude::*;
use jono_core::Result;

pub async fn codebase_2() -> Result<()> {
    // tries "JONO_REDIS_URL" > "REDIS_URL" > Err
    let context = Context::try_from_env("work-work")?;

    // to process jobs; the worker code is further below:
    let consumer = Consumer::with_context(context, NoopWorker);
    let summary = consumer.run_next().await?;
    match summary {
        Some(WorkSummary::Success(_)) => {
            todo!("You want to do something on the worker right after?");
        }
        Some(WorkSummary::Failure(_)) => {
            todo!("... or specifically on failure?");
        }
        None => {
            todo!("... or do something if nothing was found in the queue?");
        }
    }
}

struct NoopWorker;

impl Worker for NoopWorker {
    async fn work(&self, _: &Workload) -> Result<WorkSummary> {
        Ok(WorkSummary::Success(Some(json!({"processed": true}))))
    }
}
```

```toml
[dependencies]
jono = "0.1.6-rc.7"
# or, for minimal stuff:
jono = { version = "0.1.6-rc.7", default-features = false, features = ["harvest", "runtime-tokio", "tls-none"] }
```

```rust
use jono::prelude::*;
use jono_core::Result;

pub async fn codebase_3() -> Result<()> {
    // tries "JONO_REDIS_URL" > "REDIS_URL" > Err
    let context = Context::try_from_env("work-work")?;

    // to post-process job results:
    let harvester = Harvester::with_context(context);
    let completed_jobs = harvester.harvest(3).await?;
    // do something with the completed job result
  
    // OR, use the `Reaper` trait approach, TODO
}
```
