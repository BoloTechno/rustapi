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
            ApplicationError::Domain(DomainError::InvalidPriority(_)) => (
                StatusCode::BAD_REQUEST,
                "INVALID_PRIORITY",
                self.0.to_string(),
            ),
            ApplicationError::Domain(DomainError::EmptyCategoryName) => (
                StatusCode::BAD_REQUEST,
                "EMPTY_CATEGORY_NAME",
                self.0.to_string(),
            ),
            ApplicationError::Domain(DomainError::CategoryNameTooLong { .. }) => (
                StatusCode::BAD_REQUEST,
                "CATEGORY_NAME_TOO_LONG",
                self.0.to_string(),
            ),
            ApplicationError::Domain(DomainError::CategoryColorTooLong { .. }) => (
                StatusCode::BAD_REQUEST,
                "CATEGORY_COLOR_TOO_LONG",
                self.0.to_string(),
            ),
            ApplicationError::Domain(DomainError::InvalidEmail(_)) => {
                (StatusCode::BAD_REQUEST, "INVALID_EMAIL", self.0.to_string())
            }
            ApplicationError::Domain(DomainError::EmptyPassword) => (
                StatusCode::BAD_REQUEST,
                "EMPTY_PASSWORD",
                self.0.to_string(),
            ),
            ApplicationError::Domain(DomainError::PasswordTooShort { .. }) => (
                StatusCode::BAD_REQUEST,
                "PASSWORD_TOO_SHORT",
                self.0.to_string(),
            ),
            ApplicationError::Domain(DomainError::PasswordTooLong { .. }) => (
                StatusCode::BAD_REQUEST,
                "PASSWORD_TOO_LONG",
                self.0.to_string(),
            ),
            ApplicationError::Domain(DomainError::UserNameTooLong { .. }) => (
                StatusCode::BAD_REQUEST,
                "USER_NAME_TOO_LONG",
                self.0.to_string(),
            ),
            ApplicationError::TaskNotFound(_) => {
                (StatusCode::NOT_FOUND, "TASK_NOT_FOUND", self.0.to_string())
            }
            ApplicationError::DuplicateTaskTitle(_) => {
                (StatusCode::CONFLICT, "DUPLICATE_TITLE", self.0.to_string())
            }
            ApplicationError::CategoryNotFound(_) => (
                StatusCode::NOT_FOUND,
                "CATEGORY_NOT_FOUND",
                self.0.to_string(),
            ),
            ApplicationError::CategoryNameAlreadyExists(_) => (
                StatusCode::CONFLICT,
                "DUPLICATE_CATEGORY_NAME",
                self.0.to_string(),
            ),
            ApplicationError::EmailAlreadyExists(_) => (
                StatusCode::CONFLICT,
                "EMAIL_ALREADY_EXISTS",
                self.0.to_string(),
            ),
            ApplicationError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "INVALID_CREDENTIALS",
                "Email atau password salah".to_string(),
            ),
            ApplicationError::Unauthorized(_) => {
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", self.0.to_string())
            }
            ApplicationError::InternalError(_) | ApplicationError::Repository(_) => (
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
