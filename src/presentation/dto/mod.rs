pub mod auth_dto;
pub mod category_dto;
pub mod task_dto;

pub use auth_dto::{AuthResponse, LoginRequest, RegisterRequest, UserResponse};
pub use category_dto::{CategoryResponse, CreateCategoryRequest, UpdateCategoryRequest};
pub use task_dto::{
    ApiResponse, CreateTaskRequest, PaginatedResponse, PaginationMeta, TaskQuery, TaskResponse,
    UpdateTaskRequest,
};
