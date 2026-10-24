use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::ports::UserRepository;
use crate::domain::User;

#[derive(Default)]
pub struct InMemoryUserRepository {
    users: RwLock<HashMap<Uuid, User>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn save(&self, user: &User) -> Result<(), ApplicationError> {
        let mut users = self
            .users
            .write()
            .map_err(|e| ApplicationError::InternalError(format!("Lock error: {e}")))?;

        let email_lower = user.email().to_lowercase();
        for existing in users.values() {
            if existing.id() != user.id() && existing.email().to_lowercase() == email_lower {
                return Err(ApplicationError::EmailAlreadyExists(
                    user.email().to_string(),
                ));
            }
        }

        users.insert(user.id(), user.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, ApplicationError> {
        let users = self
            .users
            .read()
            .map_err(|e| ApplicationError::InternalError(format!("Lock error: {e}")))?;
        Ok(users.get(&id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApplicationError> {
        let users = self
            .users
            .read()
            .map_err(|e| ApplicationError::InternalError(format!("Lock error: {e}")))?;
        let email_lower = email.to_lowercase();
        Ok(users
            .values()
            .find(|u| u.email().to_lowercase() == email_lower)
            .cloned())
    }
}
