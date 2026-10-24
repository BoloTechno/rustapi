use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
};
use rustapi::{
    AppState, Argon2PasswordHasher, AuthService, CategoryService, InMemoryCategoryRepository,
    InMemoryTaskRepository, InMemoryUserRepository, JwtTokenService, TaskService, create_router,
};
use tower::ServiceExt;
use tower_http::timeout::TimeoutLayer;

fn setup_test_app() -> Router {
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let cat_repo = Arc::new(InMemoryCategoryRepository::new());
    let user_repo = Arc::new(InMemoryUserRepository::new());
    let password_hasher = Arc::new(Argon2PasswordHasher::new());
    let token_service = Arc::new(JwtTokenService::new("test_secret".to_string(), 24));
    let auth_service = Arc::new(AuthService::new(
        user_repo,
        password_hasher,
        token_service.clone(),
    ));
    let task_service = Arc::new(TaskService::new(task_repo.clone(), cat_repo.clone()));
    let cat_service = Arc::new(CategoryService::new(cat_repo, task_repo));
    let state = AppState::new(task_service, cat_service, auth_service, token_service);

    create_router(state)
}

#[tokio::test]
async fn test_cors_preflight_and_headers() {
    let app = setup_test_app();

    // 1. Uji Pre-flight OPTIONS request
    let preflight_req = Request::builder()
        .method("OPTIONS")
        .uri("/api/v1/tasks")
        .header("Origin", "http://localhost:5173")
        .header("Access-Control-Request-Method", "POST")
        .header(
            "Access-Control-Request-Headers",
            "authorization,content-type",
        )
        .body(Body::empty())
        .unwrap();

    let preflight_res = app.clone().oneshot(preflight_req).await.unwrap();
    assert!(
        preflight_res.status() == StatusCode::OK
            || preflight_res.status() == StatusCode::NO_CONTENT
    );

    let allow_origin = preflight_res
        .headers()
        .get("access-control-allow-origin")
        .expect("Header access-control-allow-origin wajib ada");
    assert_eq!(allow_origin, "*");

    let allow_methods = preflight_res
        .headers()
        .get("access-control-allow-methods")
        .expect("Header access-control-allow-methods wajib ada")
        .to_str()
        .unwrap();
    assert!(allow_methods.contains("POST"));
    assert!(allow_methods.contains("GET"));

    // 2. Uji CORS header pada GET /health biasa
    let req = Request::builder()
        .method("GET")
        .uri("/health")
        .header("Origin", "http://localhost:5173")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().contains_key("access-control-allow-origin"));
}

#[tokio::test]
async fn test_request_timeout_layer() {
    // Router uji dengan batas timeout sangat pendek (50ms)
    let slow_app = Router::new()
        .route("/fast", get(|| async { "Cepat selesai!" }))
        .route(
            "/slow",
            get(|| async {
                tokio::time::sleep(Duration::from_millis(150)).await;
                "Lambat..."
            }),
        )
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_millis(50),
        ));

    // Request cepat (< 50ms) harus sukses 200 OK
    let fast_req = Request::builder()
        .method("GET")
        .uri("/fast")
        .body(Body::empty())
        .unwrap();
    let fast_res = slow_app.clone().oneshot(fast_req).await.unwrap();
    assert_eq!(fast_res.status(), StatusCode::OK);

    // Request lambat (150ms > 50ms) harus diputus dengan HTTP 408 Request Timeout
    let slow_req = Request::builder()
        .method("GET")
        .uri("/slow")
        .body(Body::empty())
        .unwrap();
    let slow_res = slow_app.oneshot(slow_req).await.unwrap();
    assert_eq!(slow_res.status(), StatusCode::REQUEST_TIMEOUT);
}

#[test]
fn test_parse_timeout_seconds() {
    use rustapi::presentation::middleware::parse_timeout_seconds;

    // Nilai valid > 0
    assert_eq!(parse_timeout_seconds(Some("15")), 15);
    assert_eq!(parse_timeout_seconds(Some(" 60  ")), 60);

    // Kasus batas: 0 harus fallback ke 30 detik (bukan 0 detik)
    assert_eq!(parse_timeout_seconds(Some("0")), 30);
    assert_eq!(parse_timeout_seconds(Some(" 0 ")), 30);

    // Nilai tidak valid / kosong / None harus fallback ke 30 detik
    assert_eq!(parse_timeout_seconds(Some("invalid")), 30);
    assert_eq!(parse_timeout_seconds(Some("")), 30);
    assert_eq!(parse_timeout_seconds(Some("-5")), 30);
    assert_eq!(parse_timeout_seconds(None), 30);
}
