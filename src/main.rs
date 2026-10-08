//! Application entry point for the RHS backend HTTP server.
//!
//! This module assembles the Axum router, generates the OpenAPI document with
//! Utoipa, exposes Swagger UI, and starts the Tokio-powered HTTP server.

use axum::{
    Json, Router,
    routing::{get, post},
};
use dotenvy::dotenv;
use log::info;
use serde::Serialize;
use tower_http::trace::TraceLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

/// Cross-origin access configuration for browser clients.
mod cors;

/// Information-exchange endpoints and their backing integrations.
pub mod infoexchange;

/// Application service modules.
pub mod services;

/// Complete OpenAPI description of the HTTP API.
///
/// Runtime routing remains owned by Axum. This type separately collects the
/// Utoipa operation metadata used to generate the OpenAPI JSON document and
/// Swagger UI.
#[derive(OpenApi)]
#[openapi(
    info(title = "Axum Production API", version = "1.0.0"),
    paths(
        live_test,
        infoexchange::redis_sharing::get,
        infoexchange::redis_sharing::set,
        services::data::count_services,
        services::data::get_services
    ),
    tags(
        (name = infoexchange::redis_sharing::API_TAG,
            description = infoexchange::redis_sharing::API_TAG_DESCRIPTION),
        (name = services::data::API_TAG,
            description = services::data::API_TAG_DESCRIPTION)
    )
)]
struct ApiDoc;

/// Health information returned by `GET /live`.
#[derive(Serialize, ToSchema)]
struct LiveStatus {
    /// Whether the shared Redis connection has been established.
    redis_connected: bool,

    /// Whether `SERVICE_PATH` is configured in the environment.
    service_path_set: bool,
}

/// Reports the state of the application's startup dependencies.
///
/// Redis status reflects whether the shared connection has been established;
/// this endpoint does not send a command to Redis. Service path status only
/// checks whether the environment variable exists.
#[utoipa::path(
    get,
    path = "/live",
    responses((status = 200, description = "Application dependency status", body = LiveStatus))
)]
async fn live_test() -> Json<LiveStatus> {
    info!("Live endpoint pinged");

    Json(LiveStatus {
        redis_connected: infoexchange::redis_sharing::is_connected(),
        service_path_set: services::data::is_service_path_set(),
    })
}

/// Initializes application infrastructure and serves HTTP requests.
///
/// Startup performs the following work:
///
/// 1. Initializes structured logging.
/// 2. Registers the Axum routes and request tracing middleware.
/// 3. Generates and serves the OpenAPI document and Swagger UI.
/// 4. Binds the TCP listener and runs the server until it exits.
#[tokio::main]
async fn main() {
    // Sets up the env into the app
    dotenv().ok();

    // Install the global tracing subscriber before emitting application logs.
    tracing_subscriber::fmt().init();

    // Warm up Redis concurrently while the HTTP application is assembled.
    let redis_startup = tokio::spawn(async {
        match infoexchange::redis_sharing::connection_start_up().await {
            Ok(()) => log::info!("Redis connection established"),
            Err(error) => log::error!("Redis startup connection failed: {error}"),
        }
    });

    // Axum owns runtime routing independently of the OpenAPI description.
    let router = Router::new()
        .route("/live", get(live_test))
        .route("/getValues", get(infoexchange::redis_sharing::get))
        .route("/setValues", post(infoexchange::redis_sharing::set))
        .route("/getServices", get(services::data::get_services))
        .route("/countServices", get(services::data::count_services))
        // Setup the cors
        .layer(cors::layer())
        .layer(TraceLayer::new_for_http());

    // Generate the OpenAPI document from the operations registered on ApiDoc.
    let api = ApiDoc::openapi();

    // Merge Swagger UI and its OpenAPI JSON endpoint into the application.
    let app = router.merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", api));

    // Listen on every interface so published container ports are reachable.
    let bind_address = "0.0.0.0:8080";

    // Create the asynchronous TCP listener used by Axum.
    let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();

    // Announce the address only after the listener has bound successfully.
    println!("Serving at http://{bind_address}");
    println!("Swagger at http://{bind_address}/docs");

    // Serve requests until the server encounters an unrecoverable error.
    axum::serve(listener, app).await.unwrap();

    if let Err(error) = redis_startup.await {
        log::error!("Redis startup task failed: {error}");
    }
}
