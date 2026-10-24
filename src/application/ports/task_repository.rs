use async_trait::async_trait;
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::domain::{Task, TaskStatus};

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn save(&self, task: &Task) -> Result<(), ApplicationError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Task>, ApplicationError>;
    async fn find_all(&self, status: Option<TaskStatus>) -> Result<Vec<Task>, ApplicationError>;
    async fn update(&self, task: &Task) -> Result<(), ApplicationError>;
    async fn delete(&self, id: Uuid) -> Result<bool, ApplicationError>;
}
