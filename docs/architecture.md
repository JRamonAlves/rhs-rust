# Architecture

RHS Backend is one Axum service split into two areas:

```text
src/main.rs
├── cors.rs
├── infoexchange
│   └── redis_sharing.rs
└── services
    └── data.rs
```

## `src/main.rs`

`main` wires the process together. It:

- loads `.env`
- starts logging
- starts Redis warm-up in a Tokio task
- builds the Axum router
- applies the configured browser origins
- registers OpenAPI metadata
- mounts Swagger UI
- binds HTTP to `0.0.0.0:8080` so container port publishing works

It registers these routes:

- `/live`
- `/getValues`
- `/setValues`
- `/getServices`
- `/countServices`

## `src/cors.rs`

This module builds the HTTP CORS layer. The public API accepts browser requests from any origin.

## `src/infoexchange/redis_sharing.rs`

This module owns Redis value exchange.

It uses `REDIS_URL` when configured. Otherwise it falls back to `APP_ENV`:

- `APP_ENV=production` uses `redis://redis:6379/`
- anything else uses `redis://localhost:6379/`

It keeps one shared `OnceCell<ConnectionManager>`. Handlers clone the manager handle before running Redis commands.

Public functions:

- `is_connected()`
- `connection_start_up()`
- `get_value(key)`
- `set_value(key, val)`
- `get(...)`, the Axum handler for `GET /getValues`
- `set(...)`, the Axum handler for `POST /setValues`

## `src/services/data.rs`

This module owns the service catalog.

For each request, it:

1. reads `SERVICE_PATH`
2. reads the file at that path
3. parses it as `Vec<Service>`
4. returns the list or the count

The catalog is not cached. If `data.json` changes on disk, the next request reads the new file.

Public functions:

- `is_service_path_set()`
- `count_services(...)`
- `get_services(...)`

## OpenAPI

Utoipa builds the OpenAPI document from annotations in the Rust code. Swagger UI mounts at `/docs` and reads `/api-docs/openapi.json`.

## Build and deploy

The Dockerfile has two stages:

1. `rust:1.89-bookworm` builds the release binary.
2. `debian:bookworm-slim` runs the binary as the non-root `app` user.

The GitHub Actions workflow checks Rust and builds the Docker image without publishing it. The deployment Compose file builds a local image and connects it to Redis.
