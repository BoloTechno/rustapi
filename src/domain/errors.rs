use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum DomainError {
    #[error("Task title cannot be empty")]
    EmptyTitle,
    #[error("Task title cannot exceed {max} characters (got {actual})")]
    TitleTooLong { max: usize, actual: usize },
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),
}
