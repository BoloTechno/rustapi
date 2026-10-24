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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    #[default]
    Medium,
    High,
    Urgent,
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "low"),
            TaskPriority::Medium => write!(f, "medium"),
            TaskPriority::High => write!(f, "high"),
            TaskPriority::Urgent => write!(f, "urgent"),
        }
    }
}

impl std::str::FromStr for TaskPriority {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(TaskPriority::Low),
            "medium" => Ok(TaskPriority::Medium),
            "high" => Ok(TaskPriority::High),
            "urgent" => Ok(TaskPriority::Urgent),
            other => Err(DomainError::InvalidPriority(format!(
                "Prioritas '{}' tidak valid",
                other
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    id: Uuid,
    user_id: Uuid,
    title: String,
    description: Option<String>,
    status: TaskStatus,
    priority: TaskPriority,
    due_date: Option<DateTime<Utc>>,
    category_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Task {
    pub const MAX_TITLE_LENGTH: usize = 120;

    /// Buat Task baru dengan validasi domain.
    pub fn new(
        user_id: Uuid,
        title: String,
        description: Option<String>,
        priority: Option<TaskPriority>,
        due_date: Option<DateTime<Utc>>,
        category_id: Option<Uuid>,
    ) -> Result<Self, DomainError> {
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
            user_id,
            title: trimmed_title.to_string(),
            description: description
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty()),
            status: TaskStatus::Pending,
            priority: priority.unwrap_or_default(),
            due_date,
            category_id,
            created_at: now,
            updated_at: now,
        })
    }

    /// Rekonstitusi entity Task dari persistence layer (Database).
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: Uuid,
        user_id: Uuid,
        title: String,
        description: Option<String>,
        status: TaskStatus,
        priority: TaskPriority,
        due_date: Option<DateTime<Utc>>,
        category_id: Option<Uuid>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
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
        }
    }

    // Getters untuk enkapsulasi domain
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
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

    pub fn priority(&self) -> TaskPriority {
        self.priority
    }

    pub fn due_date(&self) -> Option<DateTime<Utc>> {
        self.due_date
    }

    pub fn category_id(&self) -> Option<Uuid> {
        self.category_id
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

    /// Update prioritas task.
    pub fn update_priority(&mut self, priority: TaskPriority) {
        if self.priority != priority {
            self.priority = priority;
            self.updated_at = Utc::now();
        }
    }

    /// Update tenggat waktu task.
    pub fn update_due_date(&mut self, due_date: Option<DateTime<Utc>>) {
        if self.due_date != due_date {
            self.due_date = due_date;
            self.updated_at = Utc::now();
        }
    }

    /// Update kategori task.
    pub fn update_category_id(&mut self, category_id: Option<Uuid>) {
        if self.category_id != category_id {
            self.category_id = category_id;
            self.updated_at = Utc::now();
        }
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
        let user_id = Uuid::new_v4();
        let task = Task::new(
            user_id,
            "Belajar Rust".into(),
            Some("Clean Architecture".into()),
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(task.user_id(), user_id);
        assert_eq!(task.title(), "Belajar Rust");
        assert_eq!(task.description(), Some("Clean Architecture"));
        assert_eq!(task.status(), TaskStatus::Pending);
        assert_eq!(task.priority(), TaskPriority::Medium); // Default
        assert_eq!(task.due_date(), None);
        assert_eq!(task.category_id(), None);
    }

    #[test]
    fn test_create_task_with_custom_priority_and_due_date_and_category() {
        let user_id = Uuid::new_v4();
        let due = Utc::now();
        let cat_id = Uuid::new_v4();
        let task = Task::new(
            user_id,
            "Tugas Urgent".into(),
            None,
            Some(TaskPriority::Urgent),
            Some(due),
            Some(cat_id),
        )
        .unwrap();
        assert_eq!(task.user_id(), user_id);
        assert_eq!(task.priority(), TaskPriority::Urgent);
        assert_eq!(task.due_date(), Some(due));
        assert_eq!(task.category_id(), Some(cat_id));
    }

    #[test]
    fn test_create_task_with_empty_title() {
        let user_id = Uuid::new_v4();
        let res = Task::new(user_id, "   ".into(), None, None, None, None);
        assert_eq!(res, Err(DomainError::EmptyTitle));
    }

    #[test]
    fn test_create_task_with_unicode_emojis() {
        let user_id = Uuid::new_v4();
        let emoji_title = "🦀".repeat(50);
        let task = Task::new(user_id, emoji_title.clone(), None, None, None, None).unwrap();
        assert_eq!(task.title(), emoji_title);

        let long_emoji_title = "🦀".repeat(121);
        let res = Task::new(user_id, long_emoji_title, None, None, None, None);
        assert_eq!(
            res,
            Err(DomainError::TitleTooLong {
                max: 120,
                actual: 121
            })
        );
    }

    #[test]
    fn test_priority_parsing_and_mutation() {
        let user_id = Uuid::new_v4();
        assert_eq!("low".parse::<TaskPriority>().unwrap(), TaskPriority::Low);
        assert_eq!(
            "urgent".parse::<TaskPriority>().unwrap(),
            TaskPriority::Urgent
        );
        assert!("invalid".parse::<TaskPriority>().is_err());

        let mut task = Task::new(user_id, "Uji Mutasi".into(), None, None, None, None).unwrap();
        assert_eq!(task.priority(), TaskPriority::Medium);
        task.update_priority(TaskPriority::High);
        assert_eq!(task.priority(), TaskPriority::High);
    }

    #[test]
    fn test_state_transitions_idempotent() {
        let user_id = Uuid::new_v4();
        let mut task = Task::new(user_id, "Deploy App".into(), None, None, None, None).unwrap();
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
