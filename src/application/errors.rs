use thiserror::Error;
use uuid::Uuid;

use crate::domain::DomainError;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Domain validation error: {0}")]
    Domain(#[from] DomainError),

    #[error("Task with ID '{0}' not found")]
    TaskNotFound(Uuid),

    #[error("Task with title '{0}' already exists")]
    DuplicateTaskTitle(String),

    #[error("Category with ID '{0}' not found")]
    CategoryNotFound(Uuid),

    #[error("Category with name '{0}' already exists")]
    CategoryNameAlreadyExists(String),

    #[error("User with email '{0}' already exists")]
    EmailAlreadyExists(String),

    #[error("Invalid email or password")]
    InvalidCredentials,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Repository error: {0}")]
    Repository(String),
}
