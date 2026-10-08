//! HTTP endpoints for reading the configured service catalog.
//!
//! Service data is loaded from the JSON file referenced by `SERVICE_PATH`.

use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::{env, fs::read_to_string, io};
use utoipa::ToSchema;

/// OpenAPI tag assigned to the service catalog endpoints.
pub static API_TAG: &str = "services";

/// Human-readable description of the service catalog OpenAPI tag.
pub static API_TAG_DESCRIPTION: &str = "Service catalog endpoints";

/// Reports whether `SERVICE_PATH` is configured in the environment.
pub fn is_service_path_set() -> bool {
    env::var_os("SERVICE_PATH").is_some()
}

/// Errors that can occur while loading the service catalog.
#[derive(Debug)]
enum DataError {
    MissingPath(env::VarError),
    Read(io::Error),
    InvalidJson(serde_json::Error),
}

impl std::fmt::Display for DataError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPath(error) => {
                write!(formatter, "SERVICE_PATH is not configured: {error}")
            }
            Self::Read(error) => write!(formatter, "Could not read service catalog: {error}"),
            Self::InvalidJson(error) => {
                write!(formatter, "Service catalog contains invalid JSON: {error}")
            }
        }
    }
}

/// Category used to group services in the catalog.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, ToSchema)]
pub enum ServiceCategory {
    /// Movie and television media services.
    #[serde(rename = "Movies and Series")]
    MoviesAndSeries,

    /// General-purpose applications.
    Services,

    /// Photo storage and management applications.
    Photos,

    /// Comic, manga, and ebook applications.
    Comics,
}

/// Service exposed by the catalog.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, ToSchema)]
pub struct Service {
    /// Human-readable service name.
    name: String,

    /// URL used to access the service.
    url: String,

    /// Network port exposed by the service.
    port: usize,

    /// Short description of the service's purpose.
    description: String,

    /// Group to which the service belongs.
    category: ServiceCategory,
}

/// Loaded service catalog.
#[derive(Debug)]
struct ServiceCatalog {
    services: Vec<Service>,
}

impl ServiceCatalog {
    /// Loads the catalog from the path named by `SERVICE_PATH`.
    fn load_from_env() -> Result<Self, DataError> {
        let path = env::var("SERVICE_PATH").map_err(DataError::MissingPath)?;
        Self::load_from_path(&path)
    }

    /// Loads the catalog from a JSON file.
    fn load_from_path(path: &str) -> Result<Self, DataError> {
        let raw_data = read_to_string(path).map_err(DataError::Read)?;
        Self::from_json(&raw_data)
    }

    /// Parses the catalog from a JSON string.
    fn from_json(raw_data: &str) -> Result<Self, DataError> {
        serde_json::from_str(raw_data)
            .map(|services| Self { services })
            .map_err(DataError::InvalidJson)
    }

    /// Returns the number of services in the catalog.
    fn len(&self) -> usize {
        self.services.len()
    }

    /// Consumes the catalog and returns its services.
    fn into_services(self) -> Vec<Service> {
        self.services
    }
}

/// Returns the number of services in the configured catalog.
#[utoipa::path(
    get,
    path = "/countServices",
    tag = "services",
    responses(
        (status = 200, description = "Number of configured services", body = usize),
        (status = 500, description = "Service catalog could not be read")
    )
)]
pub async fn count_services() -> Result<Json<usize>, StatusCode> {
    log::info!("GET service count");

    ServiceCatalog::load_from_env()
        .map(|catalog| Json(catalog.len()))
        .map_err(|error| {
            log::error!("Service catalog failed: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// Returns every service in the configured catalog.
#[utoipa::path(
    get,
    path = "/getServices",
    tag = "services",
    responses(
        (status = 200, description = "Configured services", body = Vec<Service>),
        (status = 500, description = "Service catalog could not be read")
    )
)]
pub async fn get_services() -> Result<Json<Vec<Service>>, StatusCode> {
    log::info!("GET services");

    ServiceCatalog::load_from_env()
        .map(ServiceCatalog::into_services)
        .map(Json)
        .map_err(|error| {
            log::error!("Service catalog failed: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::write, path::PathBuf};

    #[test]
    fn catalog_parses_services_and_counts_them() {
        let catalog = ServiceCatalog::from_json(
            r#"[
                {
                    "name": "Jellyfin",
                    "url": "https://example.test:15443",
                    "port": 15443,
                    "description": "Media server",
                    "category": "Movies and Series"
                },
                {
                    "name": "Immich",
                    "url": "https://example.test:2443",
                    "port": 2443,
                    "description": "Photos",
                    "category": "Photos"
                }
            ]"#,
        )
        .expect("catalog should parse");

        assert_eq!(catalog.len(), 2);
        assert_eq!(catalog.into_services()[0].name, "Jellyfin");
    }

    #[test]
    fn catalog_rejects_unknown_categories() {
        let error = ServiceCatalog::from_json(
            r#"[
                {
                    "name": "Unknown",
                    "url": "https://example.test",
                    "port": 443,
                    "description": "Unknown",
                    "category": "Other"
                }
            ]"#,
        )
        .expect_err("unknown category should fail");

        assert!(matches!(error, DataError::InvalidJson(_)));
    }

    #[test]
    fn catalog_loads_from_path() {
        let path = temp_file_path("rhs-backend-services.json");
        write(
            &path,
            r#"[
                {
                    "name": "Kan",
                    "url": "https://example.test:3443",
                    "port": 3443,
                    "description": "Boards",
                    "category": "Services"
                }
            ]"#,
        )
        .expect("test file should be written");

        let catalog = ServiceCatalog::load_from_path(path.to_str().unwrap())
            .expect("catalog should load from path");

        assert_eq!(catalog.len(), 1);

        std::fs::remove_file(path).ok();
    }

    fn temp_file_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("{}-{}", std::process::id(), name));
        path
    }
}
