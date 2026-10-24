use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Task, TaskPriority, TaskStatus};

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,
    pub due_date: Option<DateTime<Utc>>,
    pub category_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,
    pub due_date: Option<DateTime<Utc>>,
    pub clear_due_date: Option<bool>,
    pub category_id: Option<Uuid>,
    pub clear_category_id: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TaskQuery {
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub category_id: Option<Uuid>,
    pub search: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct PaginationMeta {
    pub page: u64,
    pub per_page: u64,
    pub total_items: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: u64, per_page: u64, total_items: u64) -> Self {
        let per_page = if per_page == 0 { 10 } else { per_page };
        let total_pages = if total_items == 0 {
            1
        } else {
            total_items.div_ceil(per_page)
        };

        Self {
            success: true,
            data,
            pagination: PaginationMeta {
                page: if page == 0 { 1 } else { page },
                per_page,
                total_items,
                total_pages,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub due_date: Option<DateTime<Utc>>,
    pub category_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Task> for TaskResponse {
    fn from(task: Task) -> Self {
        Self {
            id: task.id(),
            title: task.title().to_string(),
            description: task.description().map(str::to_string),
            status: task.status(),
            priority: task.priority(),
            due_date: task.due_date(),
            category_id: task.category_id(),
            created_at: task.created_at(),
            updated_at: task.updated_at(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
        }
    }
}
