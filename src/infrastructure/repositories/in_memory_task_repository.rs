use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::ports::TaskRepository;
use crate::domain::{Task, TaskStatus};

#[derive(Default)]
pub struct InMemoryTaskRepository {
    storage: RwLock<HashMap<Uuid, Task>>,
}

impl InMemoryTaskRepository {
    pub fn new() -> Self {
        Self {
            storage: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl TaskRepository for InMemoryTaskRepository {
    async fn save(&self, task: &Task) -> Result<(), ApplicationError> {
        let mut map = self.storage.write().await;
        map.insert(task.id(), task.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Task>, ApplicationError> {
        let map = self.storage.read().await;
        Ok(map.get(&id).cloned())
    }

    async fn find_all(&self, status: Option<TaskStatus>) -> Result<Vec<Task>, ApplicationError> {
        let map = self.storage.read().await;
        let mut list: Vec<Task> = match status {
            Some(s) => map.values().filter(|t| t.status() == s).cloned().collect(),
            None => map.values().cloned().collect(),
        };
        // Lepas read-lock secepatnya agar concurrent writer tidak terblokir saat sorting
        drop(map);

        list.sort_by_key(|a| std::cmp::Reverse(a.created_at()));
        Ok(list)
    }

    async fn update(&self, task: &Task) -> Result<(), ApplicationError> {
        let mut map = self.storage.write().await;
        if let std::collections::hash_map::Entry::Occupied(mut e) = map.entry(task.id()) {
            e.insert(task.clone());
            Ok(())
        } else {
            Err(ApplicationError::TaskNotFound(task.id()))
        }
    }

    async fn delete(&self, id: Uuid) -> Result<bool, ApplicationError> {
        let mut map = self.storage.write().await;
        Ok(map.remove(&id).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_repository_crud() {
        let repo = InMemoryTaskRepository::new();
        let task = Task::new("Uji Repository".into(), None).unwrap();

        // 1. Save
        repo.save(&task).await.unwrap();

        // 2. Find by id
        let found = repo.find_by_id(task.id()).await.unwrap();
        assert_eq!(found.unwrap().title(), "Uji Repository");

        // 3. Find all
        let all = repo.find_all(None).await.unwrap();
        assert_eq!(all.len(), 1);

        // 4. Delete
        let deleted = repo.delete(task.id()).await.unwrap();
        assert!(deleted);

        // 5. Verify deleted
        let found_after = repo.find_by_id(task.id()).await.unwrap();
        assert!(found_after.is_none());
    }
}
