# Configuration

The app loads `.env` at startup with `dotenvy`.

## Variables

| Variable | Required | Default | Used by | Description |
| --- | --- | --- | --- | --- |
| `SERVICE_PATH` | Yes for `/getServices` and `/countServices` | none | Service catalog | Path to the JSON service file. |
| `REDIS_URL` | No | based on `APP_ENV` | Redis | Redis connection URL. Takes precedence over `APP_ENV`. |
| `APP_ENV` | No | `development` | Redis | `production` uses `redis://redis:6379/`. Every other value uses `redis://localhost:6379/`. |

## Local `.env`

```env
SERVICE_PATH="./data.json"
REDIS_URL="redis://localhost:6379/"
APP_ENV="development"
```

## Service catalog file

`SERVICE_PATH` must point to a JSON array. Each service uses this shape:

```json
{
  "name": "Jellyfin",
  "description": "Media server for movies, anime, and TV libraries.",
  "category": "Movies and Series",
  "url": "https://example.test:15443",
  "port": 15443
}
```

Accepted `category` values:

- `Movies and Series`
- `Services`
- `Photos`
- `Comics`

The Rust struct ignores extra fields in the input. The response only includes `name`, `description`, `category`, `url`, and `port`.

## Redis

For local Cargo development, start Redis with:

```bash
docker compose up -d redis
```

The app starts a Redis connection task during startup. If Redis is down, the HTTP server still starts. Redis endpoints return `500` until Redis accepts a connection.

Compose only manages supporting services. When running the backend in a separate container, set `REDIS_URL` to an address reachable from that container.
