use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use rustapi::{AppState, InMemoryTaskRepository, TaskService, create_router};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

fn setup_test_app() -> axum::Router {
    let repo = Arc::new(InMemoryTaskRepository::new());
    let service = Arc::new(TaskService::new(repo));
    let state = AppState::new(service);
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
                        "description": "Clean Architecture REST API"
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
                        "description": "Deskripsi awal"
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

    // 2. Patch title and description
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
                        "description": "Deskripsi terupdate"
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
}
