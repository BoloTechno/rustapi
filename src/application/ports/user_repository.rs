use async_trait::async_trait;
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::domain::User;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: &User) -> Result<(), ApplicationError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, ApplicationError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApplicationError>;
}
