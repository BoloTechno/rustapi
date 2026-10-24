use axum::Json;
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct HealthStatus {
    pub status: &'static str,
    pub version: &'static str,
    pub timestamp: String,
}

pub async fn health_check() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        timestamp: Utc::now().to_rfc3339(),
    })
}
