use axum::http::{
    HeaderValue, Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, ORIGIN},
};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

pub fn create_cors_layer() -> CorsLayer {
    let allowed_methods = [
        Method::GET,
        Method::POST,
        Method::PATCH,
        Method::DELETE,
        Method::OPTIONS,
        Method::HEAD,
    ];

    let allowed_headers = [
        AUTHORIZATION,
        CONTENT_TYPE,
        ACCEPT,
        ORIGIN,
        axum::http::header::HeaderName::from_static("x-requested-with"),
    ];

    let layer = CorsLayer::new()
        .allow_methods(allowed_methods)
        .allow_headers(allowed_headers);

    if let Ok(origins_str) = std::env::var("CORS_ALLOWED_ORIGINS") {
        let trimmed = origins_str.trim();
        if trimmed == "*" || trimmed.is_empty() {
            layer.allow_origin(Any)
        } else {
            let origins: Vec<HeaderValue> = trimmed
                .split(',')
                .filter_map(|s| {
                    let s_trimmed = s.trim();
                    if s_trimmed.is_empty() {
                        None
                    } else {
                        HeaderValue::from_str(s_trimmed).ok()
                    }
                })
                .collect();

            if origins.is_empty() {
                layer.allow_origin(Any)
            } else {
                layer.allow_origin(AllowOrigin::list(origins))
            }
        }
    } else {
        // Fallback default untuk lingkungan pengembangan (mengizinkan seluruh origin)
        layer.allow_origin(Any)
    }
}
