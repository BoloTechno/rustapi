use std::sync::Arc;
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::ports::{PasswordHasher, TokenService, UserRepository};
use crate::domain::User;

pub struct AuthService {
    user_repository: Arc<dyn UserRepository>,
    password_hasher: Arc<dyn PasswordHasher>,
    token_service: Arc<dyn TokenService>,
}

impl AuthService {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        password_hasher: Arc<dyn PasswordHasher>,
        token_service: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            user_repository,
            password_hasher,
            token_service,
        }
    }

    /// Use case: Registrasi pengguna baru
    pub async fn register(
        &self,
        email: String,
        name: Option<String>,
        raw_password: String,
    ) -> Result<(User, String), ApplicationError> {
        let validated_email = User::validate_email(&email)?;
        User::validate_raw_password(&raw_password)?;

        if self
            .user_repository
            .find_by_email(&validated_email)
            .await?
            .is_some()
        {
            return Err(ApplicationError::EmailAlreadyExists(validated_email));
        }

        let password_hash = self.password_hasher.hash_password(&raw_password)?;
        let user = User::new(validated_email, name, password_hash)?;
        self.user_repository.save(&user).await?;

        let token = self.token_service.generate_token(user.id())?;
        Ok((user, token))
    }

    /// Use case: Login pengguna
    pub async fn login(
        &self,
        email: String,
        raw_password: String,
    ) -> Result<(User, String), ApplicationError> {
        let validated_email =
            User::validate_email(&email).map_err(|_| ApplicationError::InvalidCredentials)?;

        let user = self
            .user_repository
            .find_by_email(&validated_email)
            .await?
            .ok_or(ApplicationError::InvalidCredentials)?;

        let is_valid = self
            .password_hasher
            .verify_password(&raw_password, user.password_hash())?;

        if !is_valid {
            return Err(ApplicationError::InvalidCredentials);
        }

        let token = self.token_service.generate_token(user.id())?;
        Ok((user, token))
    }

    /// Use case: Mengambil profil user yang sedang aktif
    pub async fn get_me(&self, user_id: Uuid) -> Result<User, ApplicationError> {
        self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ApplicationError::Unauthorized("User tidak ditemukan".to_string()))
    }
}
