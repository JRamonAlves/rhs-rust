# API reference

Local base URL:

```text
http://127.0.0.1:8080
```

Swagger UI is at `/docs`. The generated OpenAPI JSON is at `/api-docs/openapi.json`.

## `GET /live`

Returns dependency state.

### `200`

```json
{
  "redis_connected": true,
  "service_path_set": true
}
```

Fields:

- `redis_connected`: `true` after the shared Redis connection manager has started.
- `service_path_set`: `true` when `SERVICE_PATH` exists in the process environment.

This endpoint does not ping Redis or read the service file. It only reports process state.

## `GET /getServices`

Reads the JSON file at `SERVICE_PATH` and returns the service list.

### `200`

```json
[
  {
    "name": "Jellyfin",
    "url": "https://example.test:15443",
    "port": 15443,
    "description": "Media server for movies, anime, and TV libraries.",
    "category": "Movies and Series"
  }
]
```

### `500`

The handler returns `500` when:

- `SERVICE_PATH` is missing
- the file cannot be read
- the file is not a JSON array of services

## `GET /countServices`

Reads the JSON file at `SERVICE_PATH` and returns the number of services.

### `200`

```json
14
```

### `500`

The handler returns `500` when the catalog cannot be loaded.

## `GET /getValues`

Reads a string from Redis.

### Query parameters

| Name | Required | Description |
| --- | --- | --- |
| `key` | Yes | Redis key to read. |

Example:

```bash
curl "http://127.0.0.1:8080/getValues?key=greeting"
```

### `200`

```json
"hello"
```

A missing Redis key returns JSON `null`.

### `500`

The handler returns `500` when Redis fails the command.

## `POST /setValues`

Stores a string in Redis without an expiration.

### Query parameters

| Name | Required | Description |
| --- | --- | --- |
| `key` | Yes | Redis key to write. |
| `value` | Yes | Value to store. |

Example:

```bash
curl -X POST "http://127.0.0.1:8080/setValues?key=greeting&value=hello"
```

### `204`

The handler stored the value.

### `500`

The handler returns `500` when Redis fails the command.
