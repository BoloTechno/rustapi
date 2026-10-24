pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

pub use application::{
    ApplicationError, AuthService, CategoryRepository, CategoryService, PasswordHasher, TaskFilter,
    TaskRepository, TaskService, TokenService, UserRepository,
};
pub use domain::{Category, DomainError, Task, TaskPriority, TaskStatus, User};
pub use infrastructure::{
    Argon2PasswordHasher, InMemoryCategoryRepository, InMemoryTaskRepository,
    InMemoryUserRepository, JwtTokenService, SqliteCategoryRepository, SqliteTaskRepository,
    SqliteUserRepository,
};
pub use presentation::{
    ApiError, AppState, AuthenticatedUser, PaginatedResponse, PaginationMeta, create_router,
};
