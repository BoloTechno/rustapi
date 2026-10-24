use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use rustapi::{
    AppState, CategoryService, InMemoryCategoryRepository, InMemoryTaskRepository, TaskService,
    create_router,
};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

fn setup_test_app() -> axum::Router {
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let cat_repo = Arc::new(InMemoryCategoryRepository::new());
    let task_service = Arc::new(TaskService::new(task_repo.clone(), cat_repo.clone()));
    let cat_service = Arc::new(CategoryService::new(cat_repo, task_repo));
    let state = AppState::new(task_service, cat_service);
    create_router(state)
}

#[tokio::test]
async fn test_full_api_workflow() {
    let app = setup_test_app();

    // 1. Test POST /api/v1/tasks (Create Valid Task)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Belajar Axum",
                        "description": "Clean Architecture REST API",
                        "priority": "high"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json_body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_body["success"], true);
    assert_eq!(json_body["data"]["title"], "Belajar Axum");
    assert_eq!(json_body["data"]["status"], "pending");
    assert_eq!(json_body["data"]["priority"], "high");

    let task_id = json_body["data"]["id"].as_str().unwrap().to_string();

    // 2. Test POST /api/v1/tasks with invalid data (empty title) -> 400 Bad Request
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "title": "   " }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 3. Test GET /api/v1/tasks/:id
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/tasks/{}", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 4. Test PATCH /api/v1/tasks/:id/complete
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/tasks/{}/complete", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json_body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_body["data"]["status"], "completed");

    // 4b. Test Idempotensi: PATCH /api/v1/tasks/:id/complete kedua kali (tetap 200 OK)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/tasks/{}/complete", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 4c. Test PATCH /api/v1/tasks/:id/reopen (status -> pending)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/tasks/{}/reopen", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json_body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_body["data"]["status"], "pending");

    // 5. Test DELETE /api/v1/tasks/:id
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/tasks/{}", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 6. Test GET /api/v1/tasks/:id (should now be 404 NOT_FOUND)
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/tasks/{}", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_unicode_title_handling() {
    let app = setup_test_app();

    // Judul dengan emoji (4 byte per emoji di UTF-8)
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Membaca buku Rust 🦀 bersama komunitas ✨",
                        "description": "Unicode test"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json_body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json_body["data"]["title"],
        "Membaca buku Rust 🦀 bersama komunitas ✨"
    );
}

#[tokio::test]
async fn test_update_task_fields() {
    let app = setup_test_app();

    // 1. Create task
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Draft Judul",
                        "description": "Deskripsi awal",
                        "priority": "low"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json_body: Value = serde_json::from_slice(&body).unwrap();
    let task_id = json_body["data"]["id"].as_str().unwrap().to_string();

    // 2. Patch title, description, and priority
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/tasks/{}", task_id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Judul Baru",
                        "description": "Deskripsi terupdate",
                        "priority": "urgent"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json_body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_body["data"]["title"], "Judul Baru");
    assert_eq!(json_body["data"]["description"], "Deskripsi terupdate");
    assert_eq!(json_body["data"]["priority"], "urgent");
}

#[tokio::test]
async fn test_filter_tasks_by_status() {
    let app = setup_test_app();

    // Create Task 1 (Pending)
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "title": "Task 1" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Create Task 2 (Will be completed)
    let res2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "title": "Task 2" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = res2.into_body().collect().await.unwrap().to_bytes();
    let json2: Value = serde_json::from_slice(&body).unwrap();
    let task2_id = json2["data"]["id"].as_str().unwrap().to_string();

    // Mark Task 2 as completed
    app.clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/tasks/{}/complete", task2_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Query status=completed
    let res_completed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/tasks?status=completed")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = res_completed
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let json_completed: Value = serde_json::from_slice(&body).unwrap();
    let completed_tasks = json_completed["data"].as_array().unwrap();
    assert_eq!(completed_tasks.len(), 1);
    assert_eq!(completed_tasks[0]["title"], "Task 2");
    assert_eq!(json_completed["pagination"]["total_items"], 1);

    // Query status=pending
    let res_pending = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/tasks?status=pending")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = res_pending.into_body().collect().await.unwrap().to_bytes();
    let json_pending: Value = serde_json::from_slice(&body).unwrap();
    let pending_tasks = json_pending["data"].as_array().unwrap();
    assert_eq!(pending_tasks.len(), 1);
    assert_eq!(pending_tasks[0]["title"], "Task 1");
    assert_eq!(json_pending["pagination"]["total_items"], 1);
}

#[tokio::test]
async fn test_health_check_endpoints() {
    let app = setup_test_app();

    // 1. GET /health
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json_res: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_res["status"], "ok");
    assert_eq!(json_res["version"], env!("CARGO_PKG_VERSION"));

    // 2. GET /api/v1/health
    let res2 = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_pagination_and_keyword_search() {
    let app = setup_test_app();

    for i in 1..=7 {
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        json!({
                            "title": format!("Pekerjaan nomor {i}"),
                            "description": format!("Detail riset #{i}"),
                            "priority": if i == 7 { "urgent" } else { "medium" }
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    // 1. Page 1, limit 3
    let res_page1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/tasks?page=1&per_page=3")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_page1.status(), StatusCode::OK);
    let body = res_page1.into_body().collect().await.unwrap().to_bytes();
    let json_page1: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_page1["data"].as_array().unwrap().len(), 3);
    assert_eq!(json_page1["pagination"]["page"], 1);
    assert_eq!(json_page1["pagination"]["per_page"], 3);
    assert_eq!(json_page1["pagination"]["total_items"], 7);
    assert_eq!(json_page1["pagination"]["total_pages"], 3);

    // 2. Search keyword "nomor 5"
    let res_search = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/tasks?search=nomor%205")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_search.status(), StatusCode::OK);
    let body = res_search.into_body().collect().await.unwrap().to_bytes();
    let json_search: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_search["data"].as_array().unwrap().len(), 1);
    assert_eq!(json_search["data"][0]["title"], "Pekerjaan nomor 5");

    // 3. Filter priority=urgent
    let res_urgent = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/tasks?priority=urgent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_urgent.status(), StatusCode::OK);
    let body = res_urgent.into_body().collect().await.unwrap().to_bytes();
    let json_urgent: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_urgent["data"].as_array().unwrap().len(), 1);
    assert_eq!(json_urgent["data"][0]["title"], "Pekerjaan nomor 7");
}

#[tokio::test]
async fn test_unique_task_title_conflict() {
    let app = setup_test_app();

    // 1. Buat task pertama
    let res1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Judul Harus Unik"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res1.status(), StatusCode::CREATED);

    // 2. Coba buat task kedua dengan judul yang sama persis (case-insensitive) -> 409 Conflict
    let res2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "  judul harus unik  "
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res2.status(), StatusCode::CONFLICT);
    let body = res2.into_body().collect().await.unwrap().to_bytes();
    let json_err: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_err["success"], false);
    assert_eq!(json_err["error"]["code"], "DUPLICATE_TITLE");

    // 3. Buat task ketiga dengan judul lain
    let res3 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Judul Ketiga"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body3 = res3.into_body().collect().await.unwrap().to_bytes();
    let json3: Value = serde_json::from_slice(&body3).unwrap();
    let task3_id = json3["data"]["id"].as_str().unwrap();

    // 4. Update task ketiga dengan judul task pertama -> 409 Conflict
    let res_patch = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/tasks/{}", task3_id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Judul Harus Unik"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_patch.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_huge_page_number_does_not_overflow() {
    let app = setup_test_app();

    let res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/tasks?page=18446744073709551615&per_page=100")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["data"].as_array().unwrap().len(), 0);
}
