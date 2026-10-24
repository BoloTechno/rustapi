use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::ports::{TaskFilter, TaskRepository};
use crate::domain::{Task, TaskPriority, TaskStatus};

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
                user_id TEXT,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL,
                priority TEXT NOT NULL DEFAULT 'medium',
                due_date TEXT,
                category_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        // Verifikasi dan migrasikan kolom jika tabel sudah ada dari versi sebelumnya
        let rows = sqlx::query("PRAGMA table_info(tasks);")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut has_user_id = false;
        let mut has_priority = false;
        let mut has_due_date = false;
        let mut has_category_id = false;
        for row in rows {
            let col_name: String = row
                .try_get("name")
                .map_err(|e| ApplicationError::Repository(e.to_string()))?;
            if col_name == "user_id" {
                has_user_id = true;
            }
            if col_name == "priority" {
                has_priority = true;
            }
            if col_name == "due_date" {
                has_due_date = true;
            }
            if col_name == "category_id" {
                has_category_id = true;
            }
        }

        if !has_user_id {
            sqlx::query("ALTER TABLE tasks ADD COLUMN user_id TEXT;")
                .execute(&self.pool)
                .await
                .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        }

        // Backfill user_id untuk baris data lawas agar tidak NULL
        sqlx::query(
            "UPDATE tasks SET user_id = '00000000-0000-0000-0000-000000000000' WHERE user_id IS NULL;",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        if !has_priority {
            sqlx::query("ALTER TABLE tasks ADD COLUMN priority TEXT NOT NULL DEFAULT 'medium';")
                .execute(&self.pool)
                .await
                .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        }

        if !has_due_date {
            sqlx::query("ALTER TABLE tasks ADD COLUMN due_date TEXT;")
                .execute(&self.pool)
                .await
                .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        }

        if !has_category_id {
            sqlx::query("ALTER TABLE tasks ADD COLUMN category_id TEXT;")
                .execute(&self.pool)
                .await
                .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        }

        // Migrasi index: hapus index global lama, buat composite unique index per user
        let _ = sqlx::query("DROP INDEX IF EXISTS idx_tasks_title;")
            .execute(&self.pool)
            .await;

        sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_tasks_user_title ON tasks(user_id, title COLLATE NOCASE);",
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
            INSERT INTO tasks (id, user_id, title, description, status, priority, due_date, category_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(task.id().to_string())
        .bind(task.user_id().to_string())
        .bind(task.title())
        .bind(task.description())
        .bind(task.status().to_string())
        .bind(task.priority().to_string())
        .bind(task.due_date().map(|d| d.to_rfc3339()))
        .bind(task.category_id().map(|c| c.to_string()))
        .bind(task.created_at().to_rfc3339())
        .bind(task.updated_at().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("UNIQUE constraint failed") && (err_str.contains("tasks.title") || err_str.contains("idx_tasks_user_title")) {
                ApplicationError::DuplicateTaskTitle(task.title().to_string())
            } else {
                ApplicationError::Repository(err_str)
            }
        })?;

        Ok(())
    }

    async fn find_by_id(&self, user_id: Uuid, id: Uuid) -> Result<Option<Task>, ApplicationError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, title, description, status, priority, due_date, category_id, created_at, updated_at
            FROM tasks
            WHERE user_id = ? AND id = ?
            "#,
        )
        .bind(user_id.to_string())
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(map_row_to_task(r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_title(
        &self,
        user_id: Uuid,
        title: &str,
    ) -> Result<Option<Task>, ApplicationError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, title, description, status, priority, due_date, category_id, created_at, updated_at
            FROM tasks
            WHERE user_id = ? AND title = ? COLLATE NOCASE
            "#,
        )
        .bind(user_id.to_string())
        .bind(title.trim())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(map_row_to_task(r)?)),
            None => Ok(None),
        }
    }

    async fn find_with_filter(
        &self,
        user_id: Uuid,
        filter: &TaskFilter,
    ) -> Result<(Vec<Task>, u64), ApplicationError> {
        let mut where_clauses = vec!["user_id = ?".to_string()];
        let mut binds: Vec<String> = vec![user_id.to_string()];

        if let Some(status) = filter.status {
            where_clauses.push("status = ?".to_string());
            binds.push(status.to_string());
        }

        if let Some(priority) = filter.priority {
            where_clauses.push("priority = ?".to_string());
            binds.push(priority.to_string());
        }

        if let Some(category_id) = filter.category_id {
            where_clauses.push("category_id = ?".to_string());
            binds.push(category_id.to_string());
        }

        if let Some(ref search) = filter.search {
            where_clauses.push("(title LIKE ? OR description LIKE ?)".to_string());
            let pattern = format!("%{}%", search);
            binds.push(pattern.clone());
            binds.push(pattern);
        }

        let where_sql = format!("WHERE {}", where_clauses.join(" AND "));

        // 1. Hitung total item yang cocok
        let count_sql = format!("SELECT COUNT(*) as count FROM tasks {}", where_sql);
        let mut count_query = sqlx::query(&count_sql);
        for b in &binds {
            count_query = count_query.bind(b);
        }
        let count_row = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let total: i64 = count_row
            .try_get("count")
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let total = total.max(0) as u64;

        // 2. Ambil data dengan pagination
        let page = if filter.page == 0 { 1 } else { filter.page };
        let per_page = if filter.per_page == 0 {
            10
        } else {
            filter.per_page.min(100)
        };
        let offset = (page - 1).saturating_mul(per_page);

        let select_sql = format!(
            r#"
            SELECT id, user_id, title, description, status, priority, due_date, category_id, created_at, updated_at
            FROM tasks
            {}
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            where_sql
        );

        let mut data_query = sqlx::query(&select_sql);
        for b in &binds {
            data_query = data_query.bind(b);
        }
        let safe_offset = offset.min(i64::MAX as u64) as i64;
        data_query = data_query.bind(per_page as i64).bind(safe_offset);

        let rows = data_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let mut tasks = Vec::with_capacity(rows.len());
        for r in rows {
            tasks.push(map_row_to_task(r)?);
        }

        Ok((tasks, total))
    }

    async fn update(&self, task: &Task) -> Result<(), ApplicationError> {
        let result = sqlx::query(
            r#"
            UPDATE tasks
            SET title = ?, description = ?, status = ?, priority = ?, due_date = ?, category_id = ?, updated_at = ?
            WHERE user_id = ? AND id = ?
            "#,
        )
        .bind(task.title())
        .bind(task.description())
        .bind(task.status().to_string())
        .bind(task.priority().to_string())
        .bind(task.due_date().map(|d| d.to_rfc3339()))
        .bind(task.category_id().map(|c| c.to_string()))
        .bind(task.updated_at().to_rfc3339())
        .bind(task.user_id().to_string())
        .bind(task.id().to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("UNIQUE constraint failed") && (err_str.contains("tasks.title") || err_str.contains("idx_tasks_user_title")) {
                ApplicationError::DuplicateTaskTitle(task.title().to_string())
            } else {
                ApplicationError::Repository(err_str)
            }
        })?;

        if result.rows_affected() == 0 {
            return Err(ApplicationError::TaskNotFound(task.id()));
        }
        Ok(())
    }

    async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<bool, ApplicationError> {
        let result = sqlx::query("DELETE FROM tasks WHERE user_id = ? AND id = ?")
            .bind(user_id.to_string())
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    async fn clear_category_id(
        &self,
        user_id: Uuid,
        category_id: Uuid,
    ) -> Result<u64, ApplicationError> {
        let result = sqlx::query(
            "UPDATE tasks SET category_id = NULL WHERE user_id = ? AND category_id = ?",
        )
        .bind(user_id.to_string())
        .bind(category_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(result.rows_affected())
    }
}

fn map_row_to_task(row: sqlx::sqlite::SqliteRow) -> Result<Task, ApplicationError> {
    let id_str: String = row
        .try_get("id")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
    let user_id_str: Option<String> = row.try_get("user_id").unwrap_or(None);
    let title: String = row
        .try_get("title")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
    let description: Option<String> = row
        .try_get("description")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
    let status_str: String = row
        .try_get("status")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
    let priority_str: String = row
        .try_get("priority")
        .unwrap_or_else(|_| "medium".to_string());
    let due_date_str: Option<String> = row.try_get("due_date").unwrap_or(None);
    let category_id_str: Option<String> = row.try_get("category_id").unwrap_or(None);
    let created_at_str: String = row
        .try_get("created_at")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
    let updated_at_str: String = row
        .try_get("updated_at")
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

    let id = Uuid::parse_str(&id_str)
        .map_err(|e| ApplicationError::Repository(format!("Invalid UUID in SQLite: {}", e)))?;
    let user_id = user_id_str
        .and_then(|u| Uuid::parse_str(&u).ok())
        .unwrap_or_default();
    let status: TaskStatus = status_str.parse().map_err(ApplicationError::Domain)?;
    let priority: TaskPriority = priority_str.parse().map_err(ApplicationError::Domain)?;
    let due_date = if let Some(ref d_str) = due_date_str {
        Some(
            DateTime::parse_from_rfc3339(d_str)
                .map_err(|e| {
                    ApplicationError::Repository(format!(
                        "Invalid due_date timestamp in SQLite: {}",
                        e
                    ))
                })?
                .with_timezone(&Utc),
        )
    } else {
        None
    };
    let category_id = if let Some(ref c_str) = category_id_str {
        Uuid::parse_str(c_str).ok()
    } else {
        None
    };
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
        user_id,
        title,
        description,
        status,
        priority,
        due_date,
        category_id,
        created_at,
        updated_at,
    ))
}
