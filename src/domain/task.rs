use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::errors::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Completed,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Completed => write!(f, "completed"),
        }
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TaskStatus::Pending),
            "completed" => Ok(TaskStatus::Completed),
            other => Err(DomainError::InvalidStateTransition(format!(
                "Status '{}' tidak valid",
                other
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    id: Uuid,
    title: String,
    description: Option<String>,
    status: TaskStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Task {
    pub const MAX_TITLE_LENGTH: usize = 120;

    /// Buat Task baru dengan validasi domain.
    pub fn new(title: String, description: Option<String>) -> Result<Self, DomainError> {
        let trimmed_title = title.trim();
        if trimmed_title.is_empty() {
            return Err(DomainError::EmptyTitle);
        }
        let char_count = trimmed_title.chars().count();
        if char_count > Self::MAX_TITLE_LENGTH {
            return Err(DomainError::TitleTooLong {
                max: Self::MAX_TITLE_LENGTH,
                actual: char_count,
            });
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            title: trimmed_title.to_string(),
            description: description
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty()),
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
        })
    }

    /// Rekonstitusi entity Task dari persistence layer (Database).
    pub fn reconstitute(
        id: Uuid,
        title: String,
        description: Option<String>,
        status: TaskStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            title,
            description,
            status,
            created_at,
            updated_at,
        }
    }

    // Getters untuk enkapsulasi domain
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn status(&self) -> TaskStatus {
        self.status
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Update judul task dengan validasi.
    pub fn update_title(&mut self, new_title: String) -> Result<(), DomainError> {
        let trimmed = new_title.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyTitle);
        }
        let char_count = trimmed.chars().count();
        if char_count > Self::MAX_TITLE_LENGTH {
            return Err(DomainError::TitleTooLong {
                max: Self::MAX_TITLE_LENGTH,
                actual: char_count,
            });
        }
        self.title = trimmed.to_string();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update deskripsi task.
    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty());
        self.updated_at = Utc::now();
    }

    /// Transisi status ke Completed (Idempotent).
    pub fn mark_completed(&mut self) -> Result<(), DomainError> {
        if self.status != TaskStatus::Completed {
            self.status = TaskStatus::Completed;
            self.updated_at = Utc::now();
        }
        Ok(())
    }

    /// Membuka kembali task ke status Pending (Idempotent).
    pub fn reopen(&mut self) -> Result<(), DomainError> {
        if self.status != TaskStatus::Pending {
            self.status = TaskStatus::Pending;
            self.updated_at = Utc::now();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_valid_task() {
        let task = Task::new("Belajar Rust".into(), Some("Clean Architecture".into())).unwrap();
        assert_eq!(task.title(), "Belajar Rust");
        assert_eq!(task.description(), Some("Clean Architecture"));
        assert_eq!(task.status(), TaskStatus::Pending);
    }

    #[test]
    fn test_create_task_with_empty_title() {
        let res = Task::new("   ".into(), None);
        assert_eq!(res, Err(DomainError::EmptyTitle));
    }

    #[test]
    fn test_create_task_with_unicode_emojis() {
        // Emojis take 4 bytes each in UTF-8. 50 emojis = 200 bytes, but only 50 characters!
        let emoji_title = "🦀".repeat(50);
        let task = Task::new(emoji_title.clone(), None).unwrap();
        assert_eq!(task.title(), emoji_title);

        // 121 emojis should fail based on character count
        let long_emoji_title = "🦀".repeat(121);
        let res = Task::new(long_emoji_title, None);
        assert_eq!(
            res,
            Err(DomainError::TitleTooLong {
                max: 120,
                actual: 121
            })
        );
    }

    #[test]
    fn test_state_transitions_idempotent() {
        let mut task = Task::new("Deploy App".into(), None).unwrap();
        assert_eq!(task.status(), TaskStatus::Pending);

        // 1. Mark completed
        task.mark_completed().unwrap();
        assert_eq!(task.status(), TaskStatus::Completed);

        // 2. Mark completed lagi (Idempotent - tidak error)
        task.mark_completed().unwrap();
        assert_eq!(task.status(), TaskStatus::Completed);

        // 3. Reopen task
        task.reopen().unwrap();
        assert_eq!(task.status(), TaskStatus::Pending);

        // 4. Reopen lagi (Idempotent - tidak error)
        task.reopen().unwrap();
        assert_eq!(task.status(), TaskStatus::Pending);
    }
}
