pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

pub use application::{ApplicationError, CategoryService, TaskFilter, TaskService};
pub use domain::{Category, TaskPriority};
pub use infrastructure::{
    InMemoryCategoryRepository, InMemoryTaskRepository, SqliteCategoryRepository,
    SqliteTaskRepository,
};
pub use presentation::{AppState, PaginatedResponse, PaginationMeta, create_router};
