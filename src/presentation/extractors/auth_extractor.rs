use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use uuid::Uuid;

use crate::application::ApplicationError;
use crate::presentation::errors::ApiError;
use crate::presentation::state::AppState;

#[derive(Debug, Clone, Copy)]
pub struct AuthenticatedUser(pub Uuid);

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|val| val.to_str().ok())
            .ok_or_else(|| {
                ApiError(ApplicationError::Unauthorized(
                    "Header Authorization wajib disertakan".to_string(),
                ))
            })?;

        let token = if let Some(stripped) = auth_header.strip_prefix("Bearer ") {
            stripped.trim()
        } else {
            return Err(ApiError(ApplicationError::Unauthorized(
                "Format header Authorization harus 'Bearer <token>'".to_string(),
            )));
        };

        let user_id = state.token_service.verify_token(token).map_err(ApiError)?;

        Ok(AuthenticatedUser(user_id))
    }
}
