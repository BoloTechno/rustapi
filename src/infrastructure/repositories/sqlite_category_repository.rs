use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::ports::CategoryRepository;
use crate::domain::Category;

pub struct SqliteCategoryRepository {
    pool: SqlitePool,
}

impl SqliteCategoryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Inisialisasi tabel categories jika belum ada
    pub async fn init_db(&self) -> Result<(), ApplicationError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS categories (
                id TEXT PRIMARY KEY NOT NULL,
                user_id TEXT,
                name TEXT NOT NULL,
                color TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        // Verifikasi migrasi kolom user_id
        let rows = sqlx::query("PRAGMA table_info(categories);")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut has_user_id = false;
        for row in rows {
            let col_name: String = row
                .try_get("name")
                .map_err(|e| ApplicationError::Repository(e.to_string()))?;
            if col_name == "user_id" {
                has_user_id = true;
            }
        }

        if !has_user_id {
            sqlx::query("ALTER TABLE categories ADD COLUMN user_id TEXT;")
                .execute(&self.pool)
                .await
                .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        }

        // Backfill user_id untuk baris data lawas agar tidak NULL
        sqlx::query(
            "UPDATE categories SET user_id = '00000000-0000-0000-0000-000000000000' WHERE user_id IS NULL;",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        // Hapus index unik global lama jika ada, ganti dengan index unik per user
        let _ = sqlx::query("DROP INDEX IF EXISTS idx_categories_name;")
            .execute(&self.pool)
            .await;

        sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_categories_user_name ON categories(user_id, name COLLATE NOCASE);",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl CategoryRepository for SqliteCategoryRepository {
    async fn save(&self, category: &Category) -> Result<(), ApplicationError> {
        sqlx::query(
            r#"
            INSERT INTO categories (id, user_id, name, color, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(category.id().to_string())
        .bind(category.user_id().to_string())
        .bind(category.name())
        .bind(category.color())
        .bind(category.created_at().to_rfc3339())
        .bind(category.updated_at().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("UNIQUE constraint failed") {
                ApplicationError::CategoryNameAlreadyExists(category.name().to_string())
            } else {
                ApplicationError::Repository(err_str)
            }
        })?;

        Ok(())
    }

    async fn find_by_id(
        &self,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Category>, ApplicationError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, name, color, created_at, updated_at
            FROM categories
            WHERE user_id = ? AND id = ?
            "#,
        )
        .bind(user_id.to_string())
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(map_row_to_category(r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_name(
        &self,
        user_id: Uuid,
        name: &str,
    ) -> Result<Option<Category>, ApplicationError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, name, color, created_at, updated_at
            FROM categories
            WHERE user_id = ? AND name = ? COLLATE NOCASE
            "#,
        )
        .bind(user_id.to_string())
        .bind(name.trim())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(map_row_to_category(r)?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self, user_id: Uuid) -> Result<Vec<Category>, ApplicationError> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, name, color, created_at, updated_at
            FROM categories
            WHERE user_id = ?
            ORDER BY name ASC
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            list.push(map_row_to_category(r)?);
        }
        Ok(list)
    }

    async fn update(&self, category: &Category) -> Result<(), ApplicationError> {
        let result = sqlx::query(
            r#"
            UPDATE categories
            SET name = ?, color = ?, updated_at = ?
            WHERE user_id = ? AND id = ?
            "#,
        )
        .bind(category.name())
        .bind(category.color())
        .bind(category.updated_at().to_rfc3339())
        .bind(category.user_id().to_string())
        .bind(category.id().to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("UNIQUE constraint failed") {
                ApplicationError::CategoryNameAlreadyExists(category.name().to_string())
            } else {
                ApplicationError::Repository(err_str)
            }
        })?;

        if result.rows_affected() == 0 {
            return Err(ApplicationError::CategoryNotFound(category.id()));
        }
        Ok(())
    }

    async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<bool, ApplicationError> {
        let result = sqlx::query("DELETE FROM categories WHERE user_id = ? AND id = ?")
            .bind(user_id.to_string())
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }
}

fn map_row_to_category(row: sqlx::sqlite::SqliteRow) -> Result<Category, ApplicationError> {
    let id_str: String = row
        .try_get("id")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
    let user_id_str: Option<String> = row.try_get("user_id").unwrap_or(None);
    let name: String = row
        .try_get("name")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
    let color: Option<String> = row.try_get("color").unwrap_or(None);
    let created_at_str: String = row
        .try_get("created_at")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
    let updated_at_str: String = row
        .try_get("updated_at")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

    let id = Uuid::parse_str(&id_str)
        .map_err(|e| ApplicationError::Repository(format!("Invalid UUID in categories: {}", e)))?;
    let user_id = user_id_str
        .and_then(|u| Uuid::parse_str(&u).ok())
        .unwrap_or_default();
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| {
            ApplicationError::Repository(format!(
                "Invalid created_at timestamp in categories: {}",
                e
            ))
        })?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|e| {
            ApplicationError::Repository(format!(
                "Invalid updated_at timestamp in categories: {}",
                e
            ))
        })?
        .with_timezone(&Utc);

    Ok(Category::reconstitute(
        id, user_id, name, color, created_at, updated_at,
    ))
}
