use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::ports::{CategoryRepository, TaskFilter, TaskRepository};
use crate::domain::{Task, TaskPriority};

pub struct TaskService {
    repository: Arc<dyn TaskRepository>,
    category_repository: Arc<dyn CategoryRepository>,
}

impl TaskService {
    pub fn new(
        repository: Arc<dyn TaskRepository>,
        category_repository: Arc<dyn CategoryRepository>,
    ) -> Self {
        Self {
            repository,
            category_repository,
        }
    }

    /// Use case: Membuat task baru (dengan validasi keunikan judul)
    pub async fn create_task(
        &self,
        title: String,
        description: Option<String>,
        priority: Option<TaskPriority>,
        due_date: Option<DateTime<Utc>>,
        category_id: Option<Uuid>,
    ) -> Result<Task, ApplicationError> {
        // 1. Validasi aturan domain terlebih dahulu (empty title, max title length, dll)
        let task = Task::new(title, description, priority, due_date, category_id)?;

        // 2. Validasi keunikan judul pada level repository
        if self.repository.find_by_title(task.title()).await?.is_some() {
            return Err(ApplicationError::DuplicateTaskTitle(
                task.title().to_string(),
            ));
        }

        // 3. Validasi keberadaan kategori jika disertakan
        if let Some(cat_id) = task.category_id()
            && self.category_repository.find_by_id(cat_id).await?.is_none()
        {
            return Err(ApplicationError::CategoryNotFound(cat_id));
        }

        self.repository.save(&task).await?;
        Ok(task)
    }

    /// Use case: Mengambil task berdasarkan ID
    pub async fn get_task_by_id(&self, id: Uuid) -> Result<Task, ApplicationError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::TaskNotFound(id))
    }

    /// Use case: Mendapatkan daftar task dengan filter, pencarian, dan pagination
    pub async fn list_tasks(
        &self,
        filter: TaskFilter,
    ) -> Result<(Vec<Task>, u64), ApplicationError> {
        self.repository.find_with_filter(&filter).await
    }

    /// Use case: Update data task (judul, deskripsi, prioritas, due date, atau kategori)
    pub async fn update_task(
        &self,
        id: Uuid,
        title: Option<String>,
        description: Option<String>,
        priority: Option<TaskPriority>,
        due_date: Option<Option<DateTime<Utc>>>,
        category_id: Option<Option<Uuid>>,
    ) -> Result<Task, ApplicationError> {
        let mut task = self.get_task_by_id(id).await?;

        if let Some(new_title) = title {
            let trimmed = new_title.trim();
            if trimmed != task.title()
                && let Some(existing) = self.repository.find_by_title(trimmed).await?
                && existing.id() != id
            {
                return Err(ApplicationError::DuplicateTaskTitle(trimmed.to_string()));
            }
            task.update_title(new_title)?;
        }

        if description.is_some() {
            task.update_description(description);
        }

        if let Some(p) = priority {
            task.update_priority(p);
        }

        if let Some(d) = due_date {
            task.update_due_date(d);
        }

        if let Some(c) = category_id {
            if let Some(cat_id) = c
                && self.category_repository.find_by_id(cat_id).await?.is_none()
            {
                return Err(ApplicationError::CategoryNotFound(cat_id));
            }
            task.update_category_id(c);
        }

        self.repository.update(&task).await?;
        Ok(task)
    }

    /// Use case: Menyelesaikan task (status -> Completed)
    pub async fn complete_task(&self, id: Uuid) -> Result<Task, ApplicationError> {
        let mut task = self.get_task_by_id(id).await?;
        task.mark_completed()?;
        self.repository.update(&task).await?;
        Ok(task)
    }

    /// Use case: Membuka kembali task (status -> Pending)
    pub async fn reopen_task(&self, id: Uuid) -> Result<Task, ApplicationError> {
        let mut task = self.get_task_by_id(id).await?;
        task.reopen()?;
        self.repository.update(&task).await?;
        Ok(task)
    }

    /// Use case: Menghapus task
    pub async fn delete_task(&self, id: Uuid) -> Result<(), ApplicationError> {
        let deleted = self.repository.delete(id).await?;
        if !deleted {
            return Err(ApplicationError::TaskNotFound(id));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Category, TaskStatus};
    use crate::infrastructure::{InMemoryCategoryRepository, InMemoryTaskRepository};

    #[tokio::test]
    async fn test_task_service_crud_flow() {
        let repo = Arc::new(InMemoryTaskRepository::new());
        let cat_repo = Arc::new(InMemoryCategoryRepository::new());
        let service = TaskService::new(repo, cat_repo);

        // 1. Create task
        let created = service
            .create_task(
                "Tulis Unit Test".into(),
                Some("Clean Architecture".into()),
                Some(TaskPriority::High),
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(created.title(), "Tulis Unit Test");
        assert_eq!(created.status(), TaskStatus::Pending);
        assert_eq!(created.priority(), TaskPriority::High);

        // 1b. Test duplicate title rejection
        let dup = service
            .create_task(
                "Tulis Unit Test".into(),
                None,
                Some(TaskPriority::Low),
                None,
                None,
            )
            .await;
        assert!(matches!(dup, Err(ApplicationError::DuplicateTaskTitle(_))));

        // 2. Get task
        let fetched = service.get_task_by_id(created.id()).await.unwrap();
        assert_eq!(fetched.id(), created.id());

        // 3. Complete task
        let completed = service.complete_task(created.id()).await.unwrap();
        assert_eq!(completed.status(), TaskStatus::Completed);

        // Reopen task
        let reopened = service.reopen_task(created.id()).await.unwrap();
        assert_eq!(reopened.status(), TaskStatus::Pending);

        // 4. Update title and priority
        let updated = service
            .update_task(
                created.id(),
                Some("Tulis Integrasi Test".into()),
                None,
                Some(TaskPriority::Urgent),
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(updated.title(), "Tulis Integrasi Test");
        assert_eq!(updated.priority(), TaskPriority::Urgent);

        // 5. Delete task
        service.delete_task(created.id()).await.unwrap();

        // 6. Verify not found
        let err = service.get_task_by_id(created.id()).await;
        assert!(matches!(err, Err(ApplicationError::TaskNotFound(_))));
    }

    #[tokio::test]
    async fn test_task_service_category_validation() {
        let repo = Arc::new(InMemoryTaskRepository::new());
        let cat_repo = Arc::new(InMemoryCategoryRepository::new());
        let service = TaskService::new(repo, cat_repo.clone());

        let fake_cat_id = Uuid::new_v4();

        // 1. Rejection saat create task dengan category_id yang tidak ada
        let err_create = service
            .create_task(
                "Task Validasi Kategori".into(),
                None,
                None,
                None,
                Some(fake_cat_id),
            )
            .await;
        assert!(matches!(
            err_create,
            Err(ApplicationError::CategoryNotFound(id)) if id == fake_cat_id
        ));

        // 2. Berhasil create jika kategori ada
        let category = Category::new("Testing".into(), None).unwrap();
        cat_repo.save(&category).await.unwrap();

        let task = service
            .create_task(
                "Task Validasi Kategori".into(),
                None,
                None,
                None,
                Some(category.id()),
            )
            .await
            .unwrap();
        assert_eq!(task.category_id(), Some(category.id()));

        // 3. Rejection saat update task dengan category_id yang tidak ada
        let err_update = service
            .update_task(task.id(), None, None, None, None, Some(Some(fake_cat_id)))
            .await;
        assert!(matches!(
            err_update,
            Err(ApplicationError::CategoryNotFound(id)) if id == fake_cat_id
        ));

        // 4. Berhasil update untuk menghapus kategori (Some(None))
        let updated = service
            .update_task(task.id(), None, None, None, None, Some(None))
            .await
            .unwrap();
        assert_eq!(updated.category_id(), None);
    }
}
