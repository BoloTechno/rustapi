use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::ports::CategoryRepository;
use crate::domain::Category;

#[derive(Default)]
pub struct InMemoryCategoryRepository {
    storage: RwLock<HashMap<Uuid, Category>>,
}

impl InMemoryCategoryRepository {
    pub fn new() -> Self {
        Self {
            storage: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl CategoryRepository for InMemoryCategoryRepository {
    async fn save(&self, category: &Category) -> Result<(), ApplicationError> {
        let mut map = self.storage.write().await;
        let name_lower = category.name().trim().to_lowercase();
        if map.values().any(|c| {
            c.user_id() == category.user_id()
                && c.id() != category.id()
                && c.name().trim().to_lowercase() == name_lower
        }) {
            return Err(ApplicationError::CategoryNameAlreadyExists(
                category.name().to_string(),
            ));
        }
        map.insert(category.id(), category.clone());
        Ok(())
    }

    async fn find_by_id(
        &self,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Category>, ApplicationError> {
        let map = self.storage.read().await;
        Ok(map.get(&id).filter(|c| c.user_id() == user_id).cloned())
    }

    async fn find_by_name(
        &self,
        user_id: Uuid,
        name: &str,
    ) -> Result<Option<Category>, ApplicationError> {
        let map = self.storage.read().await;
        let name_lower = name.trim().to_lowercase();
        Ok(map
            .values()
            .find(|c| c.user_id() == user_id && c.name().to_lowercase() == name_lower)
            .cloned())
    }

    async fn find_all(&self, user_id: Uuid) -> Result<Vec<Category>, ApplicationError> {
        let map = self.storage.read().await;
        let mut list: Vec<Category> = map
            .values()
            .filter(|c| c.user_id() == user_id)
            .cloned()
            .collect();
        drop(map);
        list.sort_by_key(|c| c.name().to_string());
        Ok(list)
    }

    async fn update(&self, category: &Category) -> Result<(), ApplicationError> {
        let mut map = self.storage.write().await;
        let name_lower = category.name().trim().to_lowercase();
        if map.values().any(|c| {
            c.user_id() == category.user_id()
                && c.id() != category.id()
                && c.name().trim().to_lowercase() == name_lower
        }) {
            return Err(ApplicationError::CategoryNameAlreadyExists(
                category.name().to_string(),
            ));
        }
        if let std::collections::hash_map::Entry::Occupied(mut e) = map.entry(category.id()) {
            if e.get().user_id() != category.user_id() {
                return Err(ApplicationError::CategoryNotFound(category.id()));
            }
            e.insert(category.clone());
            Ok(())
        } else {
            Err(ApplicationError::CategoryNotFound(category.id()))
        }
    }

    async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<bool, ApplicationError> {
        let mut map = self.storage.write().await;
        if let Some(c) = map.get(&id) {
            if c.user_id() != user_id {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
        Ok(map.remove(&id).is_some())
    }
}
