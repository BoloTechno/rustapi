use axum::http::StatusCode;
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

pub fn parse_timeout_seconds(raw: Option<&str>) -> u64 {
    raw.and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|&s| s > 0)
        .unwrap_or(30)
}

pub fn create_timeout_layer() -> TimeoutLayer {
    let raw = std::env::var("REQUEST_TIMEOUT_SECS").ok();
    let secs = parse_timeout_seconds(raw.as_deref());

    TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(secs))
}
