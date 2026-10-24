use crate::application::{CategoryService, TaskService};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub task_service: Arc<TaskService>,
    pub category_service: Arc<CategoryService>,
}

impl AppState {
    pub fn new(task_service: Arc<TaskService>, category_service: Arc<CategoryService>) -> Self {
        Self {
            task_service,
            category_service,
        }
    }
}
