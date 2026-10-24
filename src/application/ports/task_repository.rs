use async_trait::async_trait;
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::domain::{Task, TaskPriority, TaskStatus};

#[derive(Debug, Clone, Default)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub category_id: Option<Uuid>,
    pub search: Option<String>,
    pub page: u64,
    pub per_page: u64,
}

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn save(&self, task: &Task) -> Result<(), ApplicationError>;
    async fn find_by_id(&self, user_id: Uuid, id: Uuid) -> Result<Option<Task>, ApplicationError>;
    async fn find_by_title(
        &self,
        user_id: Uuid,
        title: &str,
    ) -> Result<Option<Task>, ApplicationError>;
    async fn find_with_filter(
        &self,
        user_id: Uuid,
        filter: &TaskFilter,
    ) -> Result<(Vec<Task>, u64), ApplicationError>;
    async fn update(&self, task: &Task) -> Result<(), ApplicationError>;
    async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<bool, ApplicationError>;
    async fn clear_category_id(
        &self,
        user_id: Uuid,
        category_id: Uuid,
    ) -> Result<u64, ApplicationError>;
}
