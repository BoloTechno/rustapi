use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::ports::TaskRepository;
use crate::domain::{Task, TaskStatus};

pub struct SqliteTaskRepository {
    pool: SqlitePool,
}

impl SqliteTaskRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Inisialisasi tabel SQLite jika belum ada (Auto-Migration)
    pub async fn init_db(&self) -> Result<(), ApplicationError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl TaskRepository for SqliteTaskRepository {
    async fn save(&self, task: &Task) -> Result<(), ApplicationError> {
        sqlx::query(
            r#"
            INSERT INTO tasks (id, title, description, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(task.id().to_string())
        .bind(task.title())
        .bind(task.description())
        .bind(task.status().to_string())
        .bind(task.created_at().to_rfc3339())
        .bind(task.updated_at().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Task>, ApplicationError> {
        let row = sqlx::query(
            r#"
            SELECT id, title, description, status, created_at, updated_at
            FROM tasks
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(map_row_to_task(r)?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self, status: Option<TaskStatus>) -> Result<Vec<Task>, ApplicationError> {
        let rows = match status {
            Some(s) => {
                sqlx::query(
                    r#"
                    SELECT id, title, description, status, created_at, updated_at
                    FROM tasks
                    WHERE status = ?
                    ORDER BY created_at DESC
                    "#,
                )
                .bind(s.to_string())
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query(
                    r#"
                    SELECT id, title, description, status, created_at, updated_at
                    FROM tasks
                    ORDER BY created_at DESC
                    "#,
                )
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut tasks = Vec::with_capacity(rows.len());
        for r in rows {
            tasks.push(map_row_to_task(r)?);
        }
        Ok(tasks)
    }

    async fn update(&self, task: &Task) -> Result<(), ApplicationError> {
        let result = sqlx::query(
            r#"
            UPDATE tasks
            SET title = ?, description = ?, status = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(task.title())
        .bind(task.description())
        .bind(task.status().to_string())
        .bind(task.updated_at().to_rfc3339())
        .bind(task.id().to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(ApplicationError::TaskNotFound(task.id()));
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, ApplicationError> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }
}

fn map_row_to_task(row: sqlx::sqlite::SqliteRow) -> Result<Task, ApplicationError> {
    let id_str: String = row
        .try_get("id")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
    let title: String = row
        .try_get("title")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
    let description: Option<String> = row
        .try_get("description")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
    let status_str: String = row
        .try_get("status")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
    let created_at_str: String = row
        .try_get("created_at")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
    let updated_at_str: String = row
        .try_get("updated_at")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

    let id = Uuid::parse_str(&id_str)
        .map_err(|e| ApplicationError::Repository(format!("Invalid UUID in SQLite: {}", e)))?;
    let status: TaskStatus = status_str.parse().map_err(ApplicationError::Domain)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| {
            ApplicationError::Repository(format!("Invalid created_at timestamp in SQLite: {}", e))
        })?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|e| {
            ApplicationError::Repository(format!("Invalid updated_at timestamp in SQLite: {}", e))
        })?
        .with_timezone(&Utc);

    Ok(Task::reconstitute(
        id,
        title,
        description,
        status,
        created_at,
        updated_at,
    ))
}
