use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    id: Uuid,
    email: String,
    name: Option<String>,
    password_hash: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    pub const MIN_PASSWORD_LENGTH: usize = 8;
    pub const MAX_NAME_LENGTH: usize = 100;

    /// Validasi format email domain
    pub fn validate_email(email: &str) -> Result<String, DomainError> {
        let trimmed = email.trim().to_lowercase();
        if trimmed.is_empty() || !trimmed.contains('@') {
            return Err(DomainError::InvalidEmail(trimmed));
        }

        let parts: Vec<&str> = trimmed.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(DomainError::InvalidEmail(trimmed));
        }

        let domain = parts[1];
        if domain.starts_with('.')
            || domain.ends_with('.')
            || domain.contains("..")
            || !domain.contains('.')
        {
            return Err(DomainError::InvalidEmail(trimmed));
        }

        for label in domain.split('.') {
            if label.is_empty() {
                return Err(DomainError::InvalidEmail(trimmed));
            }
        }

        Ok(trimmed)
    }

    /// Validasi kekuatan password sebelum di-hash
    pub fn validate_raw_password(password: &str) -> Result<(), DomainError> {
        if password.is_empty() {
            return Err(DomainError::EmptyPassword);
        }
        let count = password.chars().count();
        if count < Self::MIN_PASSWORD_LENGTH {
            return Err(DomainError::PasswordTooShort {
                min: Self::MIN_PASSWORD_LENGTH,
                actual: count,
            });
        }
        Ok(())
    }

    /// Membuat entitas User baru
    pub fn new(
        email: String,
        name: Option<String>,
        password_hash: String,
    ) -> Result<Self, DomainError> {
        let validated_email = Self::validate_email(&email)?;
        let trimmed_name = name.map(|n| n.trim().to_string()).filter(|n| !n.is_empty());

        if let Some(ref n) = trimmed_name {
            let count = n.chars().count();
            if count > Self::MAX_NAME_LENGTH {
                return Err(DomainError::UserNameTooLong {
                    max: Self::MAX_NAME_LENGTH,
                    actual: count,
                });
            }
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            email: validated_email,
            name: trimmed_name,
            password_hash,
            created_at: now,
            updated_at: now,
        })
    }

    /// Rekonstitusi entitas User dari database
    pub fn reconstitute(
        id: Uuid,
        email: String,
        name: Option<String>,
        password_hash: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email,
            name,
            password_hash,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_user_creation() {
        let user = User::new(
            "user@example.com".into(),
            Some("John Doe".into()),
            "hashed_pwd_123".into(),
        )
        .unwrap();

        assert_eq!(user.email(), "user@example.com");
        assert_eq!(user.name(), Some("John Doe"));
        assert_eq!(user.password_hash(), "hashed_pwd_123");
    }

    #[test]
    fn test_invalid_email_rejected() {
        assert!(User::validate_email("notanemail").is_err());
        assert!(User::validate_email("@missinguser.com").is_err());
        assert!(User::validate_email("user@nodomain").is_err());
        assert!(User::validate_email("user@.com").is_err());
        assert!(User::validate_email("user@com.").is_err());
        assert!(User::validate_email("user@domain..com").is_err());
        assert!(User::validate_email("user@.").is_err());
        assert!(User::validate_email("").is_err());
    }

    #[test]
    fn test_password_length_validation() {
        assert_eq!(
            User::validate_raw_password(""),
            Err(DomainError::EmptyPassword)
        );
        assert!(matches!(
            User::validate_raw_password("short"),
            Err(DomainError::PasswordTooShort { min: 8, actual: 5 })
        ));
        assert!(User::validate_raw_password("validpassword123").is_ok());
    }

    #[test]
    fn test_user_name_length_validation() {
        let long_name = "a".repeat(101);
        let result = User::new(
            "test@example.com".into(),
            Some(long_name),
            "password123".into(),
        );
        assert_eq!(
            result,
            Err(DomainError::UserNameTooLong {
                max: 100,
                actual: 101
            })
        );

        let valid_name = "a".repeat(100);
        assert!(
            User::new(
                "test@example.com".into(),
                Some(valid_name),
                "password123".into()
            )
            .is_ok()
        );
    }
}
