use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::presentation::dto::{
    ApiResponse, CategoryResponse, CreateCategoryRequest, UpdateCategoryRequest,
};
use crate::presentation::errors::ApiError;
use crate::presentation::state::AppState;

/// POST /api/v1/categories - Membuat kategori baru
pub async fn create_category(
    State(state): State<AppState>,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<ApiResponse<CategoryResponse>>), ApiError> {
    let category = state
        .category_service
        .create_category(payload.name, payload.color)
        .await
        .map_err(ApiError)?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(CategoryResponse::from(category))),
    ))
}

/// GET /api/v1/categories - Mengambil seluruh daftar kategori
pub async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<CategoryResponse>>>, ApiError> {
    let categories = state
        .category_service
        .list_categories()
        .await
        .map_err(ApiError)?;

    let response_data: Vec<CategoryResponse> =
        categories.into_iter().map(CategoryResponse::from).collect();
    Ok(Json(ApiResponse::success(response_data)))
}

/// GET /api/v1/categories/:id - Mengambil detail satu kategori
pub async fn get_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<CategoryResponse>>, ApiError> {
    let category = state
        .category_service
        .get_category_by_id(id)
        .await
        .map_err(ApiError)?;

    Ok(Json(ApiResponse::success(CategoryResponse::from(category))))
}

/// PATCH /api/v1/categories/:id - Memperbarui nama atau kode warna kategori
pub async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> Result<Json<ApiResponse<CategoryResponse>>, ApiError> {
    let color_update = if payload.clear_color == Some(true) {
        Some(None)
    } else {
        payload.color.map(Some)
    };

    let category = state
        .category_service
        .update_category(id, payload.name, color_update)
        .await
        .map_err(ApiError)?;

    Ok(Json(ApiResponse::success(CategoryResponse::from(category))))
}

/// DELETE /api/v1/categories/:id - Menghapus kategori
pub async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .category_service
        .delete_category(id)
        .await
        .map_err(ApiError)?;
    Ok(StatusCode::NO_CONTENT)
}
