use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    id: Uuid,
    name: String,
    color: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Category {
    pub const MAX_NAME_LENGTH: usize = 50;
    pub const MAX_COLOR_LENGTH: usize = 16;

    /// Membuat entity Category baru dengan validasi domain.
    pub fn new(name: String, color: Option<String>) -> Result<Self, DomainError> {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(DomainError::EmptyCategoryName);
        }

        let char_count = trimmed_name.chars().count();
        if char_count > Self::MAX_NAME_LENGTH {
            return Err(DomainError::CategoryNameTooLong {
                max: Self::MAX_NAME_LENGTH,
                actual: char_count,
            });
        }

        let trimmed_color = color
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty());

        if let Some(ref c) = trimmed_color {
            let color_count = c.chars().count();
            if color_count > Self::MAX_COLOR_LENGTH {
                return Err(DomainError::CategoryColorTooLong {
                    max: Self::MAX_COLOR_LENGTH,
                    actual: color_count,
                });
            }
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            name: trimmed_name.to_string(),
            color: trimmed_color,
            created_at: now,
            updated_at: now,
        })
    }

    /// Rekonstitusi entity Category dari persistence layer (Database).
    pub fn reconstitute(
        id: Uuid,
        name: String,
        color: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            color,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn update_name(&mut self, new_name: String) -> Result<(), DomainError> {
        let trimmed = new_name.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyCategoryName);
        }

        let char_count = trimmed.chars().count();
        if char_count > Self::MAX_NAME_LENGTH {
            return Err(DomainError::CategoryNameTooLong {
                max: Self::MAX_NAME_LENGTH,
                actual: char_count,
            });
        }

        self.name = trimmed.to_string();
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_color(&mut self, color: Option<String>) -> Result<(), DomainError> {
        let trimmed_color = color
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty());

        if let Some(ref c) = trimmed_color {
            let color_count = c.chars().count();
            if color_count > Self::MAX_COLOR_LENGTH {
                return Err(DomainError::CategoryColorTooLong {
                    max: Self::MAX_COLOR_LENGTH,
                    actual: color_count,
                });
            }
        }

        self.color = trimmed_color;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_valid_category() {
        let cat = Category::new("Pekerjaan".into(), Some("#007ACC".into())).unwrap();
        assert_eq!(cat.name(), "Pekerjaan");
        assert_eq!(cat.color(), Some("#007ACC"));
    }

    #[test]
    fn test_create_category_empty_name() {
        let res = Category::new("   ".into(), None);
        assert_eq!(res, Err(DomainError::EmptyCategoryName));
    }

    #[test]
    fn test_create_category_name_too_long() {
        let long_name = "A".repeat(51);
        let res = Category::new(long_name, None);
        assert_eq!(
            res,
            Err(DomainError::CategoryNameTooLong {
                max: 50,
                actual: 51
            })
        );
    }

    #[test]
    fn test_create_category_color_too_long() {
        let long_color = "#1234567890ABCDEF1".to_string(); // 18 chars
        let res = Category::new("Testing".into(), Some(long_color));
        assert!(matches!(
            res,
            Err(DomainError::CategoryColorTooLong {
                max: 16,
                actual: 18
            })
        ));
    }

    #[test]
    fn test_update_category_fields() {
        let mut cat = Category::new("Awal".into(), None).unwrap();
        cat.update_name("Baru".into()).unwrap();
        assert_eq!(cat.name(), "Baru");

        cat.update_color(Some("#FFFFFF".into())).unwrap();
        assert_eq!(cat.color(), Some("#FFFFFF"));

        // Clear color
        cat.update_color(None).unwrap();
        assert_eq!(cat.color(), None);

        // Update color too long
        let err = cat.update_color(Some("#1234567890ABCDEF1".into()));
        assert!(matches!(
            err,
            Err(DomainError::CategoryColorTooLong {
                max: 16,
                actual: 18
            })
        ));
    }
}
