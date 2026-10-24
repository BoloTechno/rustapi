use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::Category;

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub clear_color: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Category> for CategoryResponse {
    fn from(cat: Category) -> Self {
        Self {
            id: cat.id(),
            name: cat.name().to_string(),
            color: cat.color().map(str::to_string),
            created_at: cat.created_at(),
            updated_at: cat.updated_at(),
        }
    }
}
