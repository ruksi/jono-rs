# Jono

Jono is a priority job queue with ULIDs and Redis sorted sets.

+ ULIDs are used because they are lexicographically sortable.
+ Redis sorted sets are used because they foremost allow priority ordering through their score and
  secondly because after that the entities are lexicographically sorted.

## Development

```
docker compose up -d
docker compose ps

# tests should pass
cargo test

# check that the documentation looks proper
cargo doc --workspace --no-deps --open

# if/when you are done
docker compose down -v
```
