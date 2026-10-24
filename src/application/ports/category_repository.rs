use async_trait::async_trait;
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::domain::Category;

#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn save(&self, category: &Category) -> Result<(), ApplicationError>;
    async fn find_by_id(
        &self,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Category>, ApplicationError>;
    async fn find_by_name(
        &self,
        user_id: Uuid,
        name: &str,
    ) -> Result<Option<Category>, ApplicationError>;
    async fn find_all(&self, user_id: Uuid) -> Result<Vec<Category>, ApplicationError>;
    async fn update(&self, category: &Category) -> Result<(), ApplicationError>;
    async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<bool, ApplicationError>;
}
