use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::application::ports::TaskFilter;
use crate::presentation::dto::{
    ApiResponse, CreateTaskRequest, PaginatedResponse, TaskQuery, TaskResponse, UpdateTaskRequest,
};
use crate::presentation::errors::ApiError;
use crate::presentation::extractors::AuthenticatedUser;
use crate::presentation::state::AppState;

/// POST /api/v1/tasks - Membuat task baru
pub async fn create_task(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<ApiResponse<TaskResponse>>), ApiError> {
    let task = state
        .task_service
        .create_task(
            user_id,
            payload.title,
            payload.description,
            payload.priority,
            payload.due_date,
            payload.category_id,
        )
        .await
        .map_err(ApiError)?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(TaskResponse::from(task))),
    ))
}

/// GET /api/v1/tasks - Mendapatkan daftar task dengan opsi filter, search, dan pagination
pub async fn list_tasks(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Query(query): Query<TaskQuery>,
) -> Result<Json<PaginatedResponse<TaskResponse>>, ApiError> {
    let page = query.page.unwrap_or(1).clamp(1, 1_000_000);
    let per_page = query.per_page.unwrap_or(10).clamp(1, 100);

    let filter = TaskFilter {
        status: query.status,
        priority: query.priority,
        category_id: query.category_id,
        search: query.search.filter(|s| !s.trim().is_empty()),
        page,
        per_page,
    };

    let (tasks, total) = state
        .task_service
        .list_tasks(user_id, filter)
        .await
        .map_err(ApiError)?;

    let response_data: Vec<TaskResponse> = tasks.into_iter().map(TaskResponse::from).collect();
    Ok(Json(PaginatedResponse::new(
        response_data,
        page,
        per_page,
        total,
    )))
}

/// GET /api/v1/tasks/:id - Mengambil detail satu task berdasarkan ID
pub async fn get_task(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<TaskResponse>>, ApiError> {
    let task = state
        .task_service
        .get_task_by_id(user_id, id)
        .await
        .map_err(ApiError)?;

    Ok(Json(ApiResponse::success(TaskResponse::from(task))))
}

/// PATCH /api/v1/tasks/:id - Memperbarui judul, deskripsi, prioritas, atau due date task
pub async fn update_task(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<ApiResponse<TaskResponse>>, ApiError> {
    let due_date_update = if payload.clear_due_date == Some(true) {
        Some(None)
    } else {
        payload.due_date.map(Some)
    };

    let category_id_update = if payload.clear_category_id == Some(true) {
        Some(None)
    } else {
        payload.category_id.map(Some)
    };

    let task = state
        .task_service
        .update_task(
            user_id,
            id,
            payload.title,
            payload.description,
            payload.priority,
            due_date_update,
            category_id_update,
        )
        .await
        .map_err(ApiError)?;

    Ok(Json(ApiResponse::success(TaskResponse::from(task))))
}

/// PATCH /api/v1/tasks/:id/complete - Menyelesaikan task
pub async fn complete_task(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<TaskResponse>>, ApiError> {
    let task = state
        .task_service
        .complete_task(user_id, id)
        .await
        .map_err(ApiError)?;

    Ok(Json(ApiResponse::success(TaskResponse::from(task))))
}

/// PATCH /api/v1/tasks/:id/reopen - Membuka kembali task yang selesai (status -> Pending)
pub async fn reopen_task(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<TaskResponse>>, ApiError> {
    let task = state
        .task_service
        .reopen_task(user_id, id)
        .await
        .map_err(ApiError)?;

    Ok(Json(ApiResponse::success(TaskResponse::from(task))))
}

/// DELETE /api/v1/tasks/:id - Menghapus task berdasarkan ID
pub async fn delete_task(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .task_service
        .delete_task(user_id, id)
        .await
        .map_err(ApiError)?;
    Ok(StatusCode::NO_CONTENT)
}
