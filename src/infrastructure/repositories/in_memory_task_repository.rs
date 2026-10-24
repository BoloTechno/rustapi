use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::ports::{TaskFilter, TaskRepository};
use crate::domain::Task;

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
        let title_lower = task.title().trim().to_lowercase();
        if map.values().any(|t| {
            t.user_id() == task.user_id()
                && t.id() != task.id()
                && t.title().trim().to_lowercase() == title_lower
        }) {
            return Err(ApplicationError::DuplicateTaskTitle(
                task.title().to_string(),
            ));
        }
        map.insert(task.id(), task.clone());
        Ok(())
    }

    async fn find_by_id(&self, user_id: Uuid, id: Uuid) -> Result<Option<Task>, ApplicationError> {
        let map = self.storage.read().await;
        Ok(map.get(&id).filter(|t| t.user_id() == user_id).cloned())
    }

    async fn find_by_title(
        &self,
        user_id: Uuid,
        title: &str,
    ) -> Result<Option<Task>, ApplicationError> {
        let map = self.storage.read().await;
        let title_lower = title.trim().to_lowercase();
        Ok(map
            .values()
            .find(|t| t.user_id() == user_id && t.title().to_lowercase() == title_lower)
            .cloned())
    }

    async fn find_with_filter(
        &self,
        user_id: Uuid,
        filter: &TaskFilter,
    ) -> Result<(Vec<Task>, u64), ApplicationError> {
        let map = self.storage.read().await;
        let mut list: Vec<Task> = map
            .values()
            .filter(|t| {
                if t.user_id() != user_id {
                    return false;
                }
                if let Some(s) = filter.status
                    && t.status() != s
                {
                    return false;
                }
                if let Some(p) = filter.priority
                    && t.priority() != p
                {
                    return false;
                }
                if let Some(c) = filter.category_id
                    && t.category_id() != Some(c)
                {
                    return false;
                }
                if let Some(ref q) = filter.search {
                    let q_lower = q.to_lowercase();
                    let title_match = t.title().to_lowercase().contains(&q_lower);
                    let desc_match = t
                        .description()
                        .map(|d| d.to_lowercase().contains(&q_lower))
                        .unwrap_or(false);
                    if !title_match && !desc_match {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        drop(map);

        list.sort_by_key(|a| std::cmp::Reverse(a.created_at()));
        let total = list.len() as u64;

        let page = if filter.page == 0 { 1 } else { filter.page };
        let per_page = if filter.per_page == 0 {
            10
        } else {
            filter.per_page
        };
        let offset = ((page - 1).saturating_mul(per_page)) as usize;

        let paged_list: Vec<Task> = list
            .into_iter()
            .skip(offset)
            .take(per_page as usize)
            .collect();
        Ok((paged_list, total))
    }

    async fn update(&self, task: &Task) -> Result<(), ApplicationError> {
        let mut map = self.storage.write().await;
        let title_lower = task.title().trim().to_lowercase();
        if map.values().any(|t| {
            t.user_id() == task.user_id()
                && t.id() != task.id()
                && t.title().trim().to_lowercase() == title_lower
        }) {
            return Err(ApplicationError::DuplicateTaskTitle(
                task.title().to_string(),
            ));
        }
        if let std::collections::hash_map::Entry::Occupied(mut e) = map.entry(task.id()) {
            if e.get().user_id() != task.user_id() {
                return Err(ApplicationError::TaskNotFound(task.id()));
            }
            e.insert(task.clone());
            Ok(())
        } else {
            Err(ApplicationError::TaskNotFound(task.id()))
        }
    }

    async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<bool, ApplicationError> {
        let mut map = self.storage.write().await;
        if let Some(t) = map.get(&id) {
            if t.user_id() != user_id {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
        Ok(map.remove(&id).is_some())
    }

    async fn clear_category_id(
        &self,
        user_id: Uuid,
        category_id: Uuid,
    ) -> Result<u64, ApplicationError> {
        let mut map = self.storage.write().await;
        let mut count = 0;
        for task in map.values_mut() {
            if task.user_id() == user_id && task.category_id() == Some(category_id) {
                task.update_category_id(None);
                count += 1;
            }
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_repository_crud() {
        let repo = InMemoryTaskRepository::new();
        let user_id = Uuid::new_v4();
        let task = Task::new(user_id, "Uji Repository".into(), None, None, None, None).unwrap();

        // 1. Save
        repo.save(&task).await.unwrap();

        // 2. Find by id
        let found = repo.find_by_id(user_id, task.id()).await.unwrap();
        assert_eq!(found.unwrap().title(), "Uji Repository");

        // 3. Find with filter
        let filter = TaskFilter {
            page: 1,
            per_page: 10,
            ..Default::default()
        };
        let (all, total) = repo.find_with_filter(user_id, &filter).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(total, 1);

        // 4. Delete
        let deleted = repo.delete(user_id, task.id()).await.unwrap();
        assert!(deleted);

        // 5. Verify deleted
        let found_after = repo.find_by_id(user_id, task.id()).await.unwrap();
        assert!(found_after.is_none());
    }
}
