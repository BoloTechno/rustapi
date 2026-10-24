use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::presentation::dto::{
    ApiResponse, CreateTaskRequest, TaskFilterQuery, TaskResponse, UpdateTaskRequest,
};
use crate::presentation::errors::ApiError;
use crate::presentation::state::AppState;

/// POST /api/v1/tasks - Membuat task baru
pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<ApiResponse<TaskResponse>>), ApiError> {
    let task = state
        .task_service
        .create_task(payload.title, payload.description)
        .await
        .map_err(ApiError)?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(TaskResponse::from(task))),
    ))
}

/// GET /api/v1/tasks - Mendapatkan daftar task dengan opsi filter status (?status=pending)
pub async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<TaskFilterQuery>,
) -> Result<Json<ApiResponse<Vec<TaskResponse>>>, ApiError> {
    let tasks = state
        .task_service
        .list_tasks(query.status)
        .await
        .map_err(ApiError)?;

    let response_data: Vec<TaskResponse> = tasks.into_iter().map(TaskResponse::from).collect();
    Ok(Json(ApiResponse::success(response_data)))
}

/// GET /api/v1/tasks/:id - Mengambil detail satu task berdasarkan ID
pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<TaskResponse>>, ApiError> {
    let task = state
        .task_service
        .get_task_by_id(id)
        .await
        .map_err(ApiError)?;

    Ok(Json(ApiResponse::success(TaskResponse::from(task))))
}

/// PATCH /api/v1/tasks/:id - Memperbarui judul atau deskripsi task
pub async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<ApiResponse<TaskResponse>>, ApiError> {
    let task = state
        .task_service
        .update_task(id, payload.title, payload.description)
        .await
        .map_err(ApiError)?;

    Ok(Json(ApiResponse::success(TaskResponse::from(task))))
}

/// PATCH /api/v1/tasks/:id/complete - Menyelesaikan task
pub async fn complete_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<TaskResponse>>, ApiError> {
    let task = state
        .task_service
        .complete_task(id)
        .await
        .map_err(ApiError)?;

    Ok(Json(ApiResponse::success(TaskResponse::from(task))))
}

/// PATCH /api/v1/tasks/:id/reopen - Membuka kembali task yang selesai (status -> Pending)
pub async fn reopen_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<TaskResponse>>, ApiError> {
    let task = state.task_service.reopen_task(id).await.map_err(ApiError)?;

    Ok(Json(ApiResponse::success(TaskResponse::from(task))))
}

/// DELETE /api/v1/tasks/:id - Menghapus task berdasarkan ID
pub async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.task_service.delete_task(id).await.map_err(ApiError)?;
    Ok(StatusCode::NO_CONTENT)
}
