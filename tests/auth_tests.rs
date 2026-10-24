use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use rustapi::{
    AppState, Argon2PasswordHasher, AuthService, CategoryService, InMemoryCategoryRepository,
    InMemoryTaskRepository, InMemoryUserRepository, JwtTokenService, RateLimiter, TaskService,
    create_router,
};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

fn setup_test_app() -> axum::Router {
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let cat_repo = Arc::new(InMemoryCategoryRepository::new());
    let user_repo = Arc::new(InMemoryUserRepository::new());
    let password_hasher = Arc::new(Argon2PasswordHasher::new());
    let token_service = Arc::new(JwtTokenService::new(
        "super_secret_test_key".to_string(),
        24,
    ));
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
async fn test_auth_registration_and_validation() {
    let app = setup_test_app();

    // 1. Valid registration
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "user@example.com",
                        "password": "strongpassword123",
                        "name": "Budi Santoso"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json_res: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_res["success"], true);
    assert_eq!(json_res["data"]["user"]["email"], "user@example.com");
    assert_eq!(json_res["data"]["user"]["name"], "Budi Santoso");
    assert!(json_res["data"]["token"].is_string());
    assert!(json_res["data"]["user"]["password_hash"].is_null()); // Password hash must NEVER leak

    // 2. Duplicate email registration -> 409 Conflict
    let res_dup = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "USER@example.com",
                        "password": "different_pwd",
                        "name": "Duplicate"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_dup.status(), StatusCode::CONFLICT);
    let body_dup = res_dup.into_body().collect().await.unwrap().to_bytes();
    let json_dup: Value = serde_json::from_slice(&body_dup).unwrap();
    assert_eq!(json_dup["success"], false);
    assert_eq!(json_dup["error"]["code"], "EMAIL_ALREADY_EXISTS");

    // 3. Short password (< 8 chars) -> 400 Bad Request
    let res_short = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "other@example.com",
                        "password": "short",
                        "name": "Short"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_short.status(), StatusCode::BAD_REQUEST);
    let body_short = res_short.into_body().collect().await.unwrap().to_bytes();
    let json_short: Value = serde_json::from_slice(&body_short).unwrap();
    assert_eq!(json_short["error"]["code"], "PASSWORD_TOO_SHORT");

    // 4. Invalid email format -> 400 Bad Request
    let res_email = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "invalid-email-address",
                        "password": "strongpassword123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_email.status(), StatusCode::BAD_REQUEST);
    let body_email = res_email.into_body().collect().await.unwrap().to_bytes();
    let json_email: Value = serde_json::from_slice(&body_email).unwrap();
    assert_eq!(json_email["error"]["code"], "INVALID_EMAIL");

    // 5. Name too long (> 100 chars) -> 400 Bad Request
    let res_name = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "validname@example.com",
                        "password": "strongpassword123",
                        "name": "a".repeat(101)
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_name.status(), StatusCode::BAD_REQUEST);
    let body_name = res_name.into_body().collect().await.unwrap().to_bytes();
    let json_name: Value = serde_json::from_slice(&body_name).unwrap();
    assert_eq!(json_name["error"]["code"], "USER_NAME_TOO_LONG");
}

#[tokio::test]
async fn test_auth_login_and_get_me() {
    let app = setup_test_app();

    // 1. Register a user
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "login_test@example.com",
                        "password": "mypassword123",
                        "name": "Tester"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // 2. Login with wrong password -> 401 Unauthorized
    let res_wrong_pwd = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "login_test@example.com",
                        "password": "wrongpassword"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_wrong_pwd.status(), StatusCode::UNAUTHORIZED);
    let body_err = res_wrong_pwd
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let json_err: Value = serde_json::from_slice(&body_err).unwrap();
    assert_eq!(json_err["error"]["code"], "INVALID_CREDENTIALS");

    // 3. Login with correct credentials -> 200 OK
    let res_login = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "login_test@example.com",
                        "password": "mypassword123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_login.status(), StatusCode::OK);
    let body_login = res_login.into_body().collect().await.unwrap().to_bytes();
    let json_login: Value = serde_json::from_slice(&body_login).unwrap();
    let token = json_login["data"]["token"].as_str().unwrap().to_string();

    // 4. GET /api/v1/auth/me without header -> 401 Unauthorized
    let res_no_auth = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/auth/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_no_auth.status(), StatusCode::UNAUTHORIZED);

    // 5. GET /api/v1/auth/me with valid Bearer token -> 200 OK
    let res_me = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/auth/me")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_me.status(), StatusCode::OK);
    let body_me = res_me.into_body().collect().await.unwrap().to_bytes();
    let json_me: Value = serde_json::from_slice(&body_me).unwrap();
    assert_eq!(json_me["success"], true);
    assert_eq!(json_me["data"]["email"], "login_test@example.com");
    assert_eq!(json_me["data"]["name"], "Tester");
}

#[tokio::test]
async fn test_multi_tenancy_data_isolation() {
    let app = setup_test_app();

    // 1. Register User A
    let res_a = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "alice@example.com",
                        "password": "alicepassword123",
                        "name": "Alice"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body_a = res_a.into_body().collect().await.unwrap().to_bytes();
    let token_a = serde_json::from_slice::<Value>(&body_a).unwrap()["data"]["token"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Register User B
    let res_b = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "bob@example.com",
                        "password": "bobpassword123",
                        "name": "Bob"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body_b = res_b.into_body().collect().await.unwrap().to_bytes();
    let token_b = serde_json::from_slice::<Value>(&body_b).unwrap()["data"]["token"]
        .as_str()
        .unwrap()
        .to_string();

    // 3. Alice creates a Task
    let res_alice_task = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks")
                .header("Authorization", format!("Bearer {token_a}"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Rahasia Alice",
                        "description": "Hanya untuk Alice"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_alice_task.status(), StatusCode::CREATED);
    let body_alice_task = res_alice_task
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let alice_task_id = serde_json::from_slice::<Value>(&body_alice_task).unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 4. Bob tries to access Alice's Task -> 404 NOT_FOUND (Isolated)
    let res_bob_get = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/tasks/{alice_task_id}"))
                .header("Authorization", format!("Bearer {token_b}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_bob_get.status(), StatusCode::NOT_FOUND);

    // 5. Bob tries to update Alice's Task -> 404 NOT_FOUND
    let res_bob_patch = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/tasks/{alice_task_id}"))
                .header("Authorization", format!("Bearer {token_b}"))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "title": "Hacked by Bob" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_bob_patch.status(), StatusCode::NOT_FOUND);

    // 6. Bob tries to delete Alice's Task -> 404 NOT_FOUND
    let res_bob_delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/tasks/{alice_task_id}"))
                .header("Authorization", format!("Bearer {token_b}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_bob_delete.status(), StatusCode::NOT_FOUND);

    // 7. Bob lists tasks -> empty
    let res_bob_list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/tasks")
                .header("Authorization", format!("Bearer {token_b}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_bob_list.status(), StatusCode::OK);
    let body_bob_list = res_bob_list.into_body().collect().await.unwrap().to_bytes();
    let json_bob_list: Value = serde_json::from_slice(&body_bob_list).unwrap();
    assert_eq!(json_bob_list["data"].as_array().unwrap().len(), 0);

    // 8. Title uniqueness is scoped PER USER: Bob CAN create a task with identical title "Rahasia Alice"
    let res_bob_task = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks")
                .header("Authorization", format!("Bearer {token_b}"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Rahasia Alice",
                        "description": "Bob punya task dengan judul yang sama"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_bob_task.status(), StatusCode::CREATED);

    // 9. Categories are also isolated: Alice and Bob can each create "Pekerjaan"
    let res_cat_alice = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/categories")
                .header("Authorization", format!("Bearer {token_a}"))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "name": "Pekerjaan" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_cat_alice.status(), StatusCode::CREATED);

    let res_cat_bob = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/categories")
                .header("Authorization", format!("Bearer {token_b}"))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "name": "Pekerjaan" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_cat_bob.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_auth_rate_limiting() {
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

    // Atur rate limiter dengan kapasitas 3 request per jendela waktu
    let state = AppState::new(task_service, cat_service, auth_service, token_service)
        .with_rate_limiter(RateLimiter::new(3, std::time::Duration::from_secs(60)));
    let app = create_router(state);

    let client_ip = "192.168.1.100";

    // Kirim 3 request beruntun dari IP yang sama (semua harus diizinkan lewat ke handler)
    for _ in 0..3 {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header("X-Forwarded-For", client_ip)
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        json!({ "email": "any@example.com", "password": "wrong" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Handler menolak kredensial, tapi BUKAN 429
        assert_ne!(res.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    // Request ke-4 dari IP yang sama harus DITOLAK oleh rate limiter (HTTP 429)
    let res_blocked = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("X-Forwarded-For", client_ip)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "email": "any@example.com", "password": "wrong" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_blocked.status(), StatusCode::TOO_MANY_REQUESTS);
    let body = res_blocked.into_body().collect().await.unwrap().to_bytes();
    let json_res: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_res["success"], false);
    assert_eq!(json_res["error"]["code"], "TOO_MANY_REQUESTS");

    // Request dari IP BERBEDA tidak terblokir
    let res_other_ip = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("X-Forwarded-For", "192.168.1.101")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "email": "any@example.com", "password": "wrong" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(res_other_ip.status(), StatusCode::TOO_MANY_REQUESTS);
}
