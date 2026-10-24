use crate::application::TaskService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub task_service: Arc<TaskService>,
}

impl AppState {
    pub fn new(task_service: Arc<TaskService>) -> Self {
        Self { task_service }
    }
}
