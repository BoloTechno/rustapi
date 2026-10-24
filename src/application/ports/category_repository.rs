use async_trait::async_trait;
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::domain::Category;

#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn save(&self, category: &Category) -> Result<(), ApplicationError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Category>, ApplicationError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Category>, ApplicationError>;
    async fn find_all(&self) -> Result<Vec<Category>, ApplicationError>;
    async fn update(&self, category: &Category) -> Result<(), ApplicationError>;
    async fn delete(&self, id: Uuid) -> Result<bool, ApplicationError>;
}
