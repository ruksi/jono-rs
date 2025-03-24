# Development

```bash
docker compose up -d
docker compose ps

# tests should pass
cargo test

# check that the documentation looks proper
cargo doc --no-deps --open

# if/when you are done
docker compose down -v
```
