use argon2::{
    Argon2,
    password_hash::{
        PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString, rand_core::OsRng,
    },
};
use async_trait::async_trait;

use crate::application::errors::ApplicationError;
use crate::application::ports::PasswordHasher;

#[derive(Default, Clone)]
pub struct Argon2PasswordHasher;

impl Argon2PasswordHasher {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    async fn hash_password(&self, password: &str) -> Result<String, ApplicationError> {
        let password = password.to_string();
        tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            argon2
                .hash_password(password.as_bytes(), &salt)
                .map(|p| p.to_string())
                .map_err(|e| {
                    ApplicationError::InternalError(format!("Failed to hash password: {e}"))
                })
        })
        .await
        .map_err(|e| ApplicationError::InternalError(format!("Join error: {e}")))?
    }

    async fn verify_password(&self, password: &str, hash: &str) -> Result<bool, ApplicationError> {
        let password = password.to_string();
        let hash = hash.to_string();
        tokio::task::spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&hash).map_err(|e| {
                ApplicationError::InternalError(format!("Invalid password hash: {e}"))
            })?;
            let is_valid = Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok();
            Ok(is_valid)
        })
        .await
        .map_err(|e| ApplicationError::InternalError(format!("Join error: {e}")))?
    }
}
