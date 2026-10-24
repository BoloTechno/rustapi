use axum::{Json, extract::State, http::StatusCode};

use crate::presentation::dto::{
    ApiResponse, AuthResponse, LoginRequest, RegisterRequest, UserResponse,
};
use crate::presentation::errors::ApiError;
use crate::presentation::extractors::AuthenticatedUser;
use crate::presentation::state::AppState;

/// POST /api/v1/auth/register
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<ApiResponse<AuthResponse>>), ApiError> {
    let (user, token) = state
        .auth_service
        .register(payload.email, payload.name, payload.password)
        .await
        .map_err(ApiError)?;

    let response = AuthResponse {
        token,
        user: UserResponse::from(user),
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, ApiError> {
    let (user, token) = state
        .auth_service
        .login(payload.email, payload.password)
        .await
        .map_err(ApiError)?;

    let response = AuthResponse {
        token,
        user: UserResponse::from(user),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// GET /api/v1/auth/me
pub async fn get_me(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Json<ApiResponse<UserResponse>>, ApiError> {
    let user = state.auth_service.get_me(user_id).await.map_err(ApiError)?;

    Ok(Json(ApiResponse::success(UserResponse::from(user))))
}
