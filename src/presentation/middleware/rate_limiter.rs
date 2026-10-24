use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tokio::sync::Mutex;

use crate::presentation::state::AppState;

#[derive(Clone)]
pub struct RateLimiter {
    max_requests: usize,
    window: Duration,
    records: Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            records: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn check(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let cutoff = now.checked_sub(self.window).unwrap_or(now);
        let mut map = self.records.lock().await;

        let timestamps = map.entry(ip).or_default();
        timestamps.retain(|&t| t > cutoff);

        if timestamps.len() >= self.max_requests {
            false
        } else {
            timestamps.push(now);
            true
        }
    }
}

pub fn extract_client_ip(req: &Request) -> IpAddr {
    // 1. Periksa header X-Forwarded-For (sering digunakan di balik reverse proxy)
    if let Some(header_val) = req.headers().get("x-forwarded-for")
        && let Ok(header_str) = header_val.to_str()
        && let Some(first_ip) = header_str.split(',').next()
        && let Ok(ip) = first_ip.trim().parse::<IpAddr>()
    {
        return ip;
    }

    // 2. Periksa header X-Real-IP
    if let Some(header_val) = req.headers().get("x-real-ip")
        && let Ok(header_str) = header_val.to_str()
        && let Ok(ip) = header_str.trim().parse::<IpAddr>()
    {
        return ip;
    }

    // 3. Fallback default ke localhost
    IpAddr::from([127, 0, 0, 1])
}

pub async fn rate_limit_auth(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let client_ip = extract_client_ip(&req);

    if !state.auth_rate_limiter.check(client_ip).await {
        let body = Json(json!({
            "success": false,
            "error": {
                "code": "TOO_MANY_REQUESTS",
                "message": "Terlalu banyak permintaan. Silakan tunggu beberapa saat lagi."
            }
        }));
        return (StatusCode::TOO_MANY_REQUESTS, body).into_response();
    }

    next.run(req).await
}
