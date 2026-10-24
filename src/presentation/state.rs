use std::sync::Arc;

use crate::application::ports::TokenService;
use crate::application::{AuthService, CategoryService, TaskService};

#[derive(Clone)]
pub struct AppState {
    pub task_service: Arc<TaskService>,
    pub category_service: Arc<CategoryService>,
    pub auth_service: Arc<AuthService>,
    pub token_service: Arc<dyn TokenService>,
}

impl AppState {
    pub fn new(
        task_service: Arc<TaskService>,
        category_service: Arc<CategoryService>,
        auth_service: Arc<AuthService>,
        token_service: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            task_service,
            category_service,
            auth_service,
            token_service,
        }
    }
}
