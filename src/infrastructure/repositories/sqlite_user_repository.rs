use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::ports::UserRepository;
use crate::domain::User;

pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn init_db(&self) -> Result<(), ApplicationError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY NOT NULL,
                email TEXT NOT NULL,
                name TEXT,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email COLLATE NOCASE);",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(())
    }

    fn map_row_to_user(row: sqlx::sqlite::SqliteRow) -> Result<User, ApplicationError> {
        let id_str: String = row
            .try_get("id")
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| ApplicationError::Repository(format!("Invalid UUID in database: {e}")))?;

        let email: String = row
            .try_get("email")
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let name: Option<String> = row
            .try_get("name")
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let password_hash: String = row
            .try_get("password_hash")
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let created_at_str: String = row
            .try_get("created_at")
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| ApplicationError::Repository(format!("Invalid created_at: {e}")))?
            .with_timezone(&Utc);

        let updated_at_str: String = row
            .try_get("updated_at")
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| ApplicationError::Repository(format!("Invalid updated_at: {e}")))?
            .with_timezone(&Utc);

        Ok(User::reconstitute(
            id,
            email,
            name,
            password_hash,
            created_at,
            updated_at,
        ))
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn save(&self, user: &User) -> Result<(), ApplicationError> {
        let query = r#"
            INSERT INTO users (id, email, name, password_hash, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
        "#;

        let result = sqlx::query(query)
            .bind(user.id().to_string())
            .bind(user.email())
            .bind(user.name())
            .bind(user.password_hash())
            .bind(user.created_at().to_rfc3339())
            .bind(user.updated_at().to_rfc3339())
            .execute(&self.pool)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Err(
                ApplicationError::EmailAlreadyExists(user.email().to_string()),
            ),
            Err(e) => Err(ApplicationError::Repository(e.to_string())),
        }
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, ApplicationError> {
        let row = sqlx::query("SELECT * FROM users WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        row.map(Self::map_row_to_user).transpose()
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApplicationError> {
        let row = sqlx::query("SELECT * FROM users WHERE email = ? COLLATE NOCASE")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        row.map(Self::map_row_to_user).transpose()
    }
}
