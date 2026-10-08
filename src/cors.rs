//! Cross-origin access configuration for browser clients.

use axum::http::{Method, header};
use tower_http::cors::{Any, CorsLayer};

/// Builds a CORS layer that allows requests from any origin.
pub fn layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE])
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::Body, http::Request, routing::get};
    use tower::ServiceExt;

    #[tokio::test]
    async fn any_origin_receives_wildcard_cors_header() {
        let app = Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(layer());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header(header::ORIGIN, "https://example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&header::HeaderValue::from_static("*"))
        );
    }
}
