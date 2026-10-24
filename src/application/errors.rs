use thiserror::Error;
use uuid::Uuid;

use crate::domain::DomainError;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Domain validation error: {0}")]
    Domain(#[from] DomainError),

    #[error("Task with ID '{0}' not found")]
    TaskNotFound(Uuid),

    #[allow(dead_code)]
    #[error("Repository error: {0}")]
    Repository(String),
}
