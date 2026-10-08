//! Redis-backed endpoints for exchanging string values.
//!
//! This module owns the Redis configuration and connection lifecycle. The
//! connection manager is initialized lazily on the first command and shared by
//! all later requests.

use axum::{Json, extract::Query, http::StatusCode};
use redis::{AsyncTypedCommands, Client, RedisResult, aio::ConnectionManager};
use serde::Deserialize;
use std::env;
use tokio::sync::OnceCell;
use utoipa::IntoParams;

/// OpenAPI tag assigned to the Redis information-exchange endpoints.
pub static API_TAG: &str = "info-exchange";

/// Human-readable description of the Redis OpenAPI tag.
pub static API_TAG_DESCRIPTION: &str = "Redis information exchange endpoints";

const PRODUCTION_REDIS_URL: &str = "redis://redis:6379/";
const DEVELOPMENT_REDIS_URL: &str = "redis://localhost:6379/";

/// Selects the Redis URL from explicit configuration or an `APP_ENV` fallback.
fn redis_url_for_config(redis_url: Option<&str>, app_env: Option<&str>) -> String {
    if let Some(redis_url) = redis_url.filter(|url| !url.trim().is_empty()) {
        return redis_url.to_owned();
    }

    match app_env.map(str::to_lowercase).as_deref() {
        Some("production") => PRODUCTION_REDIS_URL.to_owned(),
        _ => DEVELOPMENT_REDIS_URL.to_owned(),
    }
}

/// Selects the Redis URL from the current process environment.
///
/// `REDIS_URL` takes precedence. When it is unset, `APP_ENV=production` uses
/// the Redis service hostname and all other environments use localhost.
fn redis_url() -> String {
    let configured_url = env::var("REDIS_URL").ok();
    let app_env = env::var("APP_ENV").ok();
    redis_url_for_config(configured_url.as_deref(), app_env.as_deref())
}

/// Lazily initialized, process-wide Redis connection manager.
///
/// Clones of [`ConnectionManager`] are lightweight handles that share its
/// managed, multiplexed connection.
static CONNECTION: OnceCell<ConnectionManager> = OnceCell::const_new();

/// Reports whether the shared Redis connection has been established.
///
/// This check does not attempt to connect or send a command to Redis.
pub fn is_connected() -> bool {
    CONNECTION.get().is_some()
}

/// Returns a handle to the shared Redis connection manager.
///
/// The first call creates the client and establishes the connection. Later
/// calls reuse the initialized manager.
///
/// # Errors
///
/// Returns an error if the Redis URL is invalid or the initial connection
/// cannot be established.
async fn connection() -> RedisResult<ConnectionManager> {
    CONNECTION
        .get_or_try_init(|| async {
            let client = Client::open(redis_url())?;
            log::info!("Connecting to Redis server");
            ConnectionManager::new(client).await
        })
        .await
        .cloned()
}

/// Establishes the shared Redis connection during application startup.
///
/// Calling this function eagerly initializes [`CONNECTION`] so the first Redis
/// request does not have to wait for the connection to be created.
///
/// # Errors
///
/// Returns an error if the Redis URL is invalid or the initial connection
/// cannot be established.
pub async fn connection_start_up() -> RedisResult<()> {
    connection().await.map(|_| ())
}

/// Retrieves the string stored under `key`.
///
/// A key that does not exist produces `Ok(None)`.
///
/// # Errors
///
/// Returns an error if Redis cannot execute or decode the command.
pub async fn get_value(key: &str) -> RedisResult<Option<String>> {
    let mut connection = connection().await?;
    connection.get(key).await
}

/// Stores `val` under `key`.
///
/// # Errors
///
/// Returns an error if Redis cannot execute the command.
pub async fn set_value(key: &str, val: &str) -> RedisResult<()> {
    let mut connection = connection().await?;
    connection.set(key, val).await
}

/// Query parameters accepted by `GET /getValues`.
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetValuesQuery {
    /// Redis key whose value should be retrieved.
    key: String,
}

/// Retrieves a Redis value through HTTP.
///
/// Returns the stored value as JSON, JSON `null` when the key is missing, or
/// HTTP 500 when the Redis operation fails.
#[utoipa::path(
    get,
    path = "/getValues",
    tag = "info-exchange",
    params(GetValuesQuery),
    responses(
        (status = 200, body = Option<String>),
        (status = 500, description = "redis error")
    )
)]
pub async fn get(Query(query): Query<GetValuesQuery>) -> Result<Json<Option<String>>, StatusCode> {
    let key = &query.key;
    log::info!("GET value from key {key}");

    get_value(&query.key).await.map(Json).map_err(|error| {
        log::error!("Redis failed: {error}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

/// Query parameters accepted by `POST /setValues`.
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SetValuesParams {
    /// Redis key under which the value should be stored.
    key: String,

    /// String value to store.
    value: String,
}

/// Stores a Redis value through HTTP.
///
/// Returns HTTP 204 after a successful write or HTTP 500 when the Redis
/// operation fails.
#[utoipa::path(
    post,
    path = "/setValues",
    tag = "info-exchange",
    params(SetValuesParams),
    responses(
        (status = 204, description = "Value stored"),
        (status = 500, description = "Redis error")
    )
)]
pub async fn set(Query(params): Query<SetValuesParams>) -> Result<StatusCode, StatusCode> {
    let val = &params.value;
    let key = &params.key;
    log::info!("POST value {val} at key {key}");

    set_value(&params.key, &params.value)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|error| {
            log::error!("Redis SET failed: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redis_url_uses_service_name_in_production() {
        assert_eq!(
            redis_url_for_config(None, Some("production")),
            PRODUCTION_REDIS_URL
        );
        assert_eq!(
            redis_url_for_config(None, Some("PRODUCTION")),
            PRODUCTION_REDIS_URL
        );
    }

    #[test]
    fn redis_url_uses_localhost_by_default() {
        assert_eq!(redis_url_for_config(None, None), DEVELOPMENT_REDIS_URL);
        assert_eq!(
            redis_url_for_config(None, Some("development")),
            DEVELOPMENT_REDIS_URL
        );
        assert_eq!(
            redis_url_for_config(None, Some("test")),
            DEVELOPMENT_REDIS_URL
        );
    }

    #[test]
    fn explicit_redis_url_takes_precedence() {
        assert_eq!(
            redis_url_for_config(Some("redis://cache.internal:6380/"), Some("production")),
            "redis://cache.internal:6380/"
        );
    }

    #[test]
    fn empty_redis_url_uses_environment_fallback() {
        assert_eq!(
            redis_url_for_config(Some("  "), Some("production")),
            PRODUCTION_REDIS_URL
        );
    }
}
