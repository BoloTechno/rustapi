use std::sync::Arc;
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::ports::TaskRepository;
use crate::domain::{Task, TaskStatus};

pub struct TaskService {
    repository: Arc<dyn TaskRepository>,
}

impl TaskService {
    pub fn new(repository: Arc<dyn TaskRepository>) -> Self {
        Self { repository }
    }

    /// Use case: Membuat task baru
    pub async fn create_task(
        &self,
        title: String,
        description: Option<String>,
    ) -> Result<Task, ApplicationError> {
        let task = Task::new(title, description)?;
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

    /// Use case: Mendapatkan daftar semua task (opsional filter status)
    pub async fn list_tasks(
        &self,
        status: Option<TaskStatus>,
    ) -> Result<Vec<Task>, ApplicationError> {
        self.repository.find_all(status).await
    }

    /// Use case: Update data task (judul atau deskripsi)
    pub async fn update_task(
        &self,
        id: Uuid,
        title: Option<String>,
        description: Option<String>,
    ) -> Result<Task, ApplicationError> {
        let mut task = self.get_task_by_id(id).await?;

        if let Some(new_title) = title {
            task.update_title(new_title)?;
        }

        if description.is_some() {
            task.update_description(description);
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
    use crate::infrastructure::InMemoryTaskRepository;

    #[tokio::test]
    async fn test_task_service_crud_flow() {
        let repo = Arc::new(InMemoryTaskRepository::new());
        let service = TaskService::new(repo);

        // 1. Create task
        let created = service
            .create_task("Tulis Unit Test".into(), Some("Clean Architecture".into()))
            .await
            .unwrap();
        assert_eq!(created.title(), "Tulis Unit Test");
        assert_eq!(created.status(), TaskStatus::Pending);

        // 2. Get task
        let fetched = service.get_task_by_id(created.id()).await.unwrap();
        assert_eq!(fetched.id(), created.id());

        // 3. Complete task
        let completed = service.complete_task(created.id()).await.unwrap();
        assert_eq!(completed.status(), TaskStatus::Completed);

        // Reopen task
        let reopened = service.reopen_task(created.id()).await.unwrap();
        assert_eq!(reopened.status(), TaskStatus::Pending);

        // 4. Update title
        let updated = service
            .update_task(created.id(), Some("Tulis Integrasi Test".into()), None)
            .await
            .unwrap();
        assert_eq!(updated.title(), "Tulis Integrasi Test");

        // 5. Delete task
        service.delete_task(created.id()).await.unwrap();

        // 6. Verify not found
        let err = service.get_task_by_id(created.id()).await;
        assert!(matches!(err, Err(ApplicationError::TaskNotFound(_))));
    }
}
