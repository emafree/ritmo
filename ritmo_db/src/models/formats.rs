use crate::crud_trait::CrudModel;
use crate::i18n_trait::I18nDisplayable;
use crate::GetOrCreateModel;
use sqlx::FromRow;

/// Format with i18n support
/// Uses canonical keys (e.g., "format.epub") instead of translated strings
#[derive(Debug, Clone, FromRow)]
pub struct Format {
    pub id: Option<i64>,
    pub key: String,
    pub created_at: i64,
}

impl I18nDisplayable for Format {
    fn i18n_key(&self) -> &str {
        &self.key
    }
}

// ✅ Implementa CrudModel trait - elimina necessità di get/list_all/delete custom
impl CrudModel for Format {
    const TABLE_NAME: &'static str = "formats";
    const ORDER_BY: &'static str = "key";
}

// ✅ Implement GetOrCreateModel trait
impl GetOrCreateModel for Format {
    type LookupKey = str;

    fn id(&self) -> Option<i64> {
        self.id
    }

    fn new_from_key(key: &str) -> Self {
        Format {
            id: None,
            key: key.to_string(),
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    async fn find_by_key(pool: &sqlx::SqlitePool, key: &str) -> Result<Option<Self>, sqlx::Error> {
        Self::get_by_key(pool, key).await
    }

    async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        self.save(pool).await
    }
}

impl Format {
    /// Get the display name for this format in the current UI language
    /// Uses the i18n system to translate format keys (e.g., "format.epub" -> "EPUB"/"Epub")
    pub fn display_name(&self) -> String {
        // Delegate to I18nDisplayable trait
        self.translate()
    }

    /// Create a new format and save it to the database
    /// Returns the newly created format ID
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    ///
    /// # Returns
    /// * `Ok(i64)` - The ID of the newly inserted format
    /// * `Err(sqlx::Error)` - Database error if insertion fails
    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let rec = sqlx::query!("INSERT INTO formats (key) VALUES (?)", self.key)
            .execute(pool)
            .await?;
        let id = rec.last_insert_rowid();
        Ok(id)
    }

    /// ❌ REMOVED: use `crud_get::<Format>(pool, id).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_get::<Format>(pool, id).await` instead"
    )]
    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        use crate::crud_get;
        crud_get::<Self>(pool, id).await
    }

    /// Update an existing format in the database
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(sqlx::Error)` - Database error if update fails
    pub async fn update(&self, pool: &sqlx::SqlitePool, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE formats SET key = ? WHERE id = ?", key, self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Delete a format by ID
    pub async fn delete(&self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM formats WHERE id = ?", self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Get format by key (e.g., "format.epub")
    pub async fn get_by_key(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let result = sqlx::query_as!(
            Self,
            "SELECT id, key, created_at FROM formats WHERE key = ?",
            key
        )
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    /// ❌ DEPRECATED: use `get_or_create::<Format>(pool, key).await` instead
    #[deprecated(
        since = "0.1.0",
        note = "Use `get_or_create::<Format>(pool, key).await` instead"
    )]
    pub async fn get_or_create_by_key(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<i64, sqlx::Error> {
        use crate::get_or_create;
        get_or_create::<Format>(pool, key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crud_model_impl() {
        assert_eq!(Format::TABLE_NAME, "formats");
        assert_eq!(Format::ORDER_BY, "key");
    }
}
