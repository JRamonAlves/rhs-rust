# RHS Backend

RHS Backend is a Rust API for a personal home lab. It lists service links and shares text between devices through Redis.

## Problem

Personal devices need one place to find home-lab services and exchange short text values. The API provides a service catalog and a key-value exchange endpoint for a separate web client. The service is in personal use and remains under development.

## Stack and architecture

```text
Web client -> Axum REST API -> Redis, shared text
                           -> JSON file, service catalog
                           -> Utoipa OpenAPI and Swagger UI
```

Rust and Tokio run the HTTP service. Serde parses the service catalog, and Docker Compose provides Redis. The [React web client](https://github.com/JRamonAlves/rhs-web) is maintained separately.

## Personal contribution

José Ramon Severo Alves develops the RHS for his own home lab and uses it to learn Rust. This snapshot contains the Rust backend, method documentation, OpenAPI definitions, and tests. Its deployment normally sits inside a private Tailscale network.

## Security

The application has no authentication or application-level encryption. In personal deployment, access control and transport protection depend on the private Tailscale network. CORS accepts any origin; it does not restrict access. The process binds to `0.0.0.0:8080`, so deployment must restrict which clients can reach it. Do not expose this API or Redis directly to the internet.

The included `data.json` contains sample localhost URLs, not the personal service addresses. The publication snapshot omits private network identifiers and the original repository history.

Publishing this source does not make the running service public. Both the web client and API belong behind private Tailnet access controls. Serve them through an HTTPS proxy available only to authorized Tailscale devices and keep Redis unreachable from public networks. The frontend is not an authentication boundary; an allowed device can call the API directly.

## What it exposes

- `GET /live`, dependency state
- `GET /getServices`, service catalog from `SERVICE_PATH`
- `GET /countServices`, number of services in the catalog
- `GET /getValues?key=<key>`, read a Redis value
- `POST /setValues?key=<key>&value=<value>`, write a Redis value
- `/docs`, Swagger UI
- `/api-docs/openapi.json`, OpenAPI JSON

## Requirements

- Rust with edition 2024 support
- Docker and Docker Compose for local Redis
- Redis on `localhost:6379` when running the app locally

## Configuration

The app loads `.env` when it starts.

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `SERVICE_PATH` | Yes for service endpoints | none | JSON file used by `/getServices` and `/countServices`. |
| `REDIS_URL` | No | based on `APP_ENV` | Redis connection URL. Takes precedence over `APP_ENV`. |
| `APP_ENV` | No | `development` | `production` uses Redis at `redis:6379`. Every other value uses `localhost:6379`. |

Local example:

```env
SERVICE_PATH="./data.json"
REDIS_URL="redis://localhost:6379/"
APP_ENV="development"
```

## Run locally

Start Redis:

```bash
docker compose up -d redis
```

Then run the API:

```bash
cp .env.example .env
cargo run
```

The server binds to:

```text
http://127.0.0.1:8080
```

Swagger UI is at:

```text
http://127.0.0.1:8080/docs
```

## Useful commands

```bash
cargo fmt
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo run
cargo build --release --locked
```

The current suite has eight tests covering Redis URL selection, catalog loading and validation, and wildcard CORS behavior. These tests do not require a running Redis server. HTTP endpoint and Redis round-trip checks are described in [Development](docs/development.md).

## Technical decisions and limitations

- A shared Redis connection manager provides clones for endpoint handlers.
- The service catalog is read on each request, allowing changes without restarting the process.
- OpenAPI annotations sit beside handlers; Swagger UI exposes the generated specification at `/docs`.
- Values have no expiry, and key and value are passed as query parameters. The API also logs written values. Avoid passwords and other sensitive text because browser tools, proxy logs, and backend logs can retain it.
- The test suite does not cover complete Redis or deployment integration. `/live` reports initialized state, not an active Redis health probe.

## Docker image

Build it:

```bash
docker build -t rhs-backend .
```

Run it:

```bash
docker run --rm \
  -p 127.0.0.1:8080:8080 \
  --add-host=host.docker.internal:host-gateway \
  -e SERVICE_PATH=/app/data.json \
  -e REDIS_URL=redis://host.docker.internal:6379/ \
  -v "$PWD/data.json:/app/data.json:ro" \
  rhs-backend
```

This command expects Redis on port 6379 of the host. The binary listens on `0.0.0.0:8080`, so the published API is available at `http://127.0.0.1:8080`.

## Deploy with Compose

Build the local image and start it with Redis:

```bash
docker compose -f deployment/docker-compose.yaml up -d --build
```

Stop it with:

```bash
docker compose -f deployment/docker-compose.yaml down
```

## License

MIT, see [LICENSE](LICENSE). You may study, use, modify, and redistribute this project, including commercially, while preserving the copyright and license notice. Third-party dependencies retain their own licenses.

## More docs

- [API reference](docs/api.md)
- [Configuration](docs/configuration.md)
- [Architecture](docs/architecture.md)
- [Development](docs/development.md)
