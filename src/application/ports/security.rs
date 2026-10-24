use async_trait::async_trait;
use uuid::Uuid;

use crate::application::errors::ApplicationError;

#[async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash_password(&self, password: &str) -> Result<String, ApplicationError>;
    async fn verify_password(&self, password: &str, hash: &str) -> Result<bool, ApplicationError>;
}

pub trait TokenService: Send + Sync {
    fn generate_token(&self, user_id: Uuid) -> Result<String, ApplicationError>;
    fn verify_token(&self, token: &str) -> Result<Uuid, ApplicationError>;
}
