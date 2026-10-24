use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum DomainError {
    #[error("Task title cannot be empty")]
    EmptyTitle,
    #[error("Task title cannot exceed {max} characters (got {actual})")]
    TitleTooLong { max: usize, actual: usize },
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),
    #[error("Invalid task priority: {0}")]
    InvalidPriority(String),
    #[error("Category name cannot be empty")]
    EmptyCategoryName,
    #[error("Category name cannot exceed {max} characters (got {actual})")]
    CategoryNameTooLong { max: usize, actual: usize },
    #[error("Category color cannot exceed {max} characters (got {actual})")]
    CategoryColorTooLong { max: usize, actual: usize },
    #[error("Invalid email address: '{0}'")]
    InvalidEmail(String),
    #[error("Password cannot be empty")]
    EmptyPassword,
    #[error("Password must be at least {min} characters (got {actual})")]
    PasswordTooShort { min: usize, actual: usize },
    #[error("Password cannot exceed {max} characters (got {actual})")]
    PasswordTooLong { max: usize, actual: usize },
    #[error("User name cannot exceed {max} characters (got {actual})")]
    UserNameTooLong { max: usize, actual: usize },
}
