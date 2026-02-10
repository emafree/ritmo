use crate::crud_trait::CrudModel;
use crate::i18n_trait::I18nDisplayable;
use ritmo_errors::RitmoResult;
use sqlx::FromRow;
use sqlx::SqlitePool;

/// Type with i18n support
/// Uses canonical keys (e.g., "type.novel") instead of translated strings
#[derive(Debug, Clone, FromRow)]
pub struct Type {
    pub id: Option<i64>,
    pub key: String,
    pub description: Option<String>,
    pub created_at: i64,
}

impl I18nDisplayable for Type {
    fn i18n_key(&self) -> &str {
        &self.key
    }
}

// ✅ Implementa CrudModel trait - elimina necessità di get/list_all/delete custom
impl CrudModel for Type {
    const TABLE_NAME: &'static str = "types";
    const ORDER_BY: &'static str = "key";
}

impl Type {
    /// Get the display name for this type in the current UI language
    /// Uses the i18n system to translate type keys (e.g., "type.novel" -> "Novel"/"Romanzo")
    pub fn display_name(&self) -> String {
        // Delegate to I18nDisplayable trait
        self.translate()
    }

    /// Create a new type and save it to the database
    /// Returns the newly created type ID
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    ///
    /// # Returns
    /// * `Ok(i64)` - The ID of the newly inserted type
    /// * `Err(sqlx::Error)` - Database error if insertion fails
    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let rec = sqlx::query!(
            "INSERT INTO types (key, description) VALUES (?, ?)",
            self.key,
            self.description
        )
        .execute(pool)
        .await?;
        // Recupera l'ID appena inserito
        let id = rec.last_insert_rowid();
        Ok(id)
    }

    /// ❌ REMOVED: use `crud_get::<Type>(pool, id).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_get::<Type>(pool, id).await` instead"
    )]
    pub async fn get(id: i64, pool: &SqlitePool) -> RitmoResult<Option<Self>> {
        use crate::crud_get;
        crud_get::<Self>(pool, id).await.map_err(Into::into)
    }

    /// Update an existing type in the database
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(sqlx::Error)` - Database error if update fails
    pub async fn update(&self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE types SET key = ?, description = ? WHERE id = ?",
            self.key,
            self.description,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Delete a type by ID
    pub async fn delete(&self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM types WHERE id = ?", self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Get type by key (e.g., "type.novel")
    pub async fn get_by_key(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let result = sqlx::query_as!(
            Self,
            "SELECT id, key, description, created_at FROM types WHERE key = ?",
            key
        )
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    /// Get or create type by key (e.g., "type.novel")
    /// Creates new type if it doesn't exist
    pub async fn get_or_create_by_key(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<i64, sqlx::Error> {
        if let Some(type_record) = Self::get_by_key(pool, key).await? {
            return Ok(type_record.id.unwrap_or(0));
        }
        let type_record = Type {
            id: None,
            key: key.to_string(),
            description: None,
            created_at: chrono::Utc::now().timestamp(),
        };
        type_record.save(pool).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crud_model_impl() {
        assert_eq!(Type::TABLE_NAME, "types");
        assert_eq!(Type::ORDER_BY, "key");
    }
}
