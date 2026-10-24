use std::sync::Arc;
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::ports::{CategoryRepository, TaskRepository};
use crate::domain::Category;

pub struct CategoryService {
    repository: Arc<dyn CategoryRepository>,
    task_repository: Arc<dyn TaskRepository>,
}

impl CategoryService {
    pub fn new(
        repository: Arc<dyn CategoryRepository>,
        task_repository: Arc<dyn TaskRepository>,
    ) -> Self {
        Self {
            repository,
            task_repository,
        }
    }

    /// Use case: Membuat kategori baru (dengan validasi keunikan nama per user)
    pub async fn create_category(
        &self,
        user_id: Uuid,
        name: String,
        color: Option<String>,
    ) -> Result<Category, ApplicationError> {
        // 1. Validasi aturan domain terlebih dahulu (nama tidak boleh kosong, panjang max)
        let category = Category::new(user_id, name, color)?;

        // 2. Validasi keunikan nama pada level repository untuk user terkait
        if self
            .repository
            .find_by_name(user_id, category.name())
            .await?
            .is_some()
        {
            return Err(ApplicationError::CategoryNameAlreadyExists(
                category.name().to_string(),
            ));
        }

        self.repository.save(&category).await?;
        Ok(category)
    }

    /// Use case: Mengambil kategori berdasarkan ID milik user terkait
    pub async fn get_category_by_id(
        &self,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<Category, ApplicationError> {
        self.repository
            .find_by_id(user_id, id)
            .await?
            .ok_or(ApplicationError::CategoryNotFound(id))
    }

    /// Use case: Mengambil seluruh daftar kategori milik user terkait
    pub async fn list_categories(&self, user_id: Uuid) -> Result<Vec<Category>, ApplicationError> {
        self.repository.find_all(user_id).await
    }

    /// Use case: Update kategori (nama atau kode warna) milik user terkait
    pub async fn update_category(
        &self,
        user_id: Uuid,
        id: Uuid,
        name: Option<String>,
        color: Option<Option<String>>,
    ) -> Result<Category, ApplicationError> {
        let mut category = self.get_category_by_id(user_id, id).await?;

        if let Some(new_name) = name {
            let trimmed = new_name.trim();
            if trimmed != category.name()
                && let Some(existing) = self.repository.find_by_name(user_id, trimmed).await?
                && existing.id() != id
            {
                return Err(ApplicationError::CategoryNameAlreadyExists(
                    trimmed.to_string(),
                ));
            }
            category.update_name(new_name)?;
        }

        if let Some(c) = color {
            category.update_color(c)?;
        }

        self.repository.update(&category).await?;
        Ok(category)
    }

    /// Use case: Menghapus kategori dan melepaskan referensi kategori pada task milik user terkait
    pub async fn delete_category(&self, user_id: Uuid, id: Uuid) -> Result<(), ApplicationError> {
        let deleted = self.repository.delete(user_id, id).await?;
        if !deleted {
            return Err(ApplicationError::CategoryNotFound(id));
        }
        self.task_repository.clear_category_id(user_id, id).await?;
        Ok(())
    }
}
