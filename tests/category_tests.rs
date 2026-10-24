use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use rustapi::{
    AppState, Argon2PasswordHasher, AuthService, CategoryService, InMemoryCategoryRepository,
    InMemoryTaskRepository, InMemoryUserRepository, JwtTokenService, TaskService, TokenService,
    create_router,
};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

fn setup_test_app() -> (axum::Router, String) {
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
    let state = AppState::new(
        task_service,
        cat_service,
        auth_service,
        token_service.clone(),
    );
    let token = token_service.generate_token(uuid::Uuid::new_v4()).unwrap();
    (create_router(state), token)
}

#[tokio::test]
async fn test_category_crud_and_task_relation() {
    let (app, token) = setup_test_app();
    let auth_header = format!("Bearer {token}");

    // 1. Create Category: "Pekerjaan" (#007ACC)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/categories")
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Pekerjaan",
                        "color": "#007ACC"
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
    assert_eq!(json_res["data"]["name"], "Pekerjaan");
    assert_eq!(json_res["data"]["color"], "#007ACC");
    let cat_id = json_res["data"]["id"].as_str().unwrap().to_string();

    // 2. Test Duplicate Category Name -> 409 Conflict
    let res_dup = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/categories")
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "pekerjaan"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_dup.status(), StatusCode::CONFLICT);

    // 3. Create Task associated with Category
    let res_task = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks")
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Review PR Hexagonal",
                        "category_id": cat_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_task.status(), StatusCode::CREATED);
    let body = res_task.into_body().collect().await.unwrap().to_bytes();
    let task_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(task_json["data"]["category_id"], cat_id);
    let task_id = task_json["data"]["id"].as_str().unwrap().to_string();

    // 4. Filter Tasks by Category ID
    let res_filter = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/tasks?category_id={}", cat_id))
                .header("Authorization", &auth_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_filter.status(), StatusCode::OK);
    let body = res_filter.into_body().collect().await.unwrap().to_bytes();
    let filter_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(filter_json["data"].as_array().unwrap().len(), 1);
    assert_eq!(filter_json["data"][0]["title"], "Review PR Hexagonal");

    // 5. Update Category (ganti nama & color)
    let res_patch = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/categories/{}", cat_id))
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Kantor & Proyek",
                        "color": "#10B981"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_patch.status(), StatusCode::OK);
    let body = res_patch.into_body().collect().await.unwrap().to_bytes();
    let patch_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(patch_json["data"]["name"], "Kantor & Proyek");
    assert_eq!(patch_json["data"]["color"], "#10B981");

    // 5b. Clear Category Color (clear_color: true)
    let res_clear_color = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/categories/{}", cat_id))
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "clear_color": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_clear_color.status(), StatusCode::OK);
    let clear_color_body = res_clear_color
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let clear_color_json: Value = serde_json::from_slice(&clear_color_body).unwrap();
    assert_eq!(clear_color_json["data"]["color"], Value::Null);

    // 6. Delete Category
    let res_del = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/categories/{}", cat_id))
                .header("Authorization", &auth_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_del.status(), StatusCode::NO_CONTENT);

    // 7. Verify Task otomatis terlepas dari kategori (category_id menjadi null saat kategori dihapus)
    let res_get_task = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/tasks/{}", task_id))
                .header("Authorization", &auth_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_get_task.status(), StatusCode::OK);
    let task_body = res_get_task.into_body().collect().await.unwrap().to_bytes();
    let task_json: Value = serde_json::from_slice(&task_body).unwrap();
    assert_eq!(task_json["data"]["category_id"], Value::Null);

    // 8. Verify Task tetap bisa di-update normal
    let res_clear = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/tasks/{}", task_id))
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Task Setelah Kategori Dihapus"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_clear.status(), StatusCode::OK);
    let body = res_clear.into_body().collect().await.unwrap().to_bytes();
    let clear_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(clear_json["data"]["title"], "Task Setelah Kategori Dihapus");
    assert_eq!(clear_json["data"]["category_id"], Value::Null);
}

#[tokio::test]
async fn test_reject_nonexistent_category_id_on_create_and_update() {
    let (app, token) = setup_test_app();
    let auth_header = format!("Bearer {token}");
    let random_category_id = uuid::Uuid::new_v4();

    // 1. Coba create task dengan category_id acak yang tidak ada
    let res_create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks")
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Task dengan kategori gaib",
                        "category_id": random_category_id.to_string()
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_create.status(), StatusCode::NOT_FOUND);
    let body = res_create.into_body().collect().await.unwrap().to_bytes();
    let err_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(err_json["success"], false);
    assert_eq!(err_json["error"]["code"], "CATEGORY_NOT_FOUND");

    // 2. Buat task normal tanpa category
    let res_valid = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks")
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Task normal tanpa kategori"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_valid.status(), StatusCode::CREATED);
    let valid_body = res_valid.into_body().collect().await.unwrap().to_bytes();
    let valid_json: Value = serde_json::from_slice(&valid_body).unwrap();
    let task_id = valid_json["data"]["id"].as_str().unwrap();

    // 3. Coba PATCH task dengan category_id acak yang tidak ada
    let res_patch = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/tasks/{}", task_id))
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "category_id": random_category_id.to_string()
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_patch.status(), StatusCode::NOT_FOUND);
    let patch_body = res_patch.into_body().collect().await.unwrap().to_bytes();
    let patch_err_json: Value = serde_json::from_slice(&patch_body).unwrap();
    assert_eq!(patch_err_json["success"], false);
    assert_eq!(patch_err_json["error"]["code"], "CATEGORY_NOT_FOUND");
}

#[tokio::test]
async fn test_category_color_too_long_rejected() {
    let (app, token) = setup_test_app();
    let auth_header = format!("Bearer {token}");

    // 1. Rejection saat POST kategori dengan warna melebihi 16 karakter
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/categories")
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Warna Gagal",
                        "color": "#1234567890ABCDEF1"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let err_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(err_json["success"], false);
    assert_eq!(err_json["error"]["code"], "CATEGORY_COLOR_TOO_LONG");
}
