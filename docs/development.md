# Development

## Setup

```bash
cp .env.example .env
docker compose up -d redis
cargo run
```

The API starts at:

```text
http://127.0.0.1:8080
```

## Checks

Run these before a pull request:

```bash
cargo fmt -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
```

Format the code with:

```bash
cargo fmt
```

Pull requests and pushes run these Rust checks and build the Docker image through GitHub Actions.

## Smoke test

```bash
curl http://127.0.0.1:8080/live
curl http://127.0.0.1:8080/countServices
curl http://127.0.0.1:8080/getServices
curl -X POST "http://127.0.0.1:8080/setValues?key=greeting&value=hello"
curl "http://127.0.0.1:8080/getValues?key=greeting"
```

## Current gaps

- The suite includes unit tests and an in-process HTTP CORS test. It does not exercise the complete API against a running Redis server.
- Redis values never expire.
