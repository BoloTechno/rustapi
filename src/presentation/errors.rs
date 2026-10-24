use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::application::ApplicationError;
use crate::domain::DomainError;

#[derive(Debug)]
pub struct ApiError(pub ApplicationError);

impl From<ApplicationError> for ApiError {
    fn from(err: ApplicationError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self.0 {
            ApplicationError::Domain(DomainError::EmptyTitle) => {
                (StatusCode::BAD_REQUEST, "EMPTY_TITLE", self.0.to_string())
            }
            ApplicationError::Domain(DomainError::TitleTooLong { .. }) => (
                StatusCode::BAD_REQUEST,
                "TITLE_TOO_LONG",
                self.0.to_string(),
            ),
            ApplicationError::Domain(DomainError::InvalidStateTransition(_)) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_STATE_TRANSITION",
                self.0.to_string(),
            ),
            ApplicationError::TaskNotFound(_) => {
                (StatusCode::NOT_FOUND, "TASK_NOT_FOUND", self.0.to_string())
            }
            ApplicationError::Repository(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Terjadi kesalahan internal server".to_string(),
            ),
        };

        let body = Json(json!({
            "success": false,
            "error": {
                "code": error_code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}
