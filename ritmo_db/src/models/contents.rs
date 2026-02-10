use crate::crud_trait::CrudModel;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Default)]
pub struct Content {
    /// Vale lo stesso che per Book, quando si immette un nuovo Content il suo Id è None, memorizzandolo viene assegnato.
    pub id: Option<i64>,
    pub name: String,
    pub original_title: Option<String>,
    pub type_id: Option<i64>,
    pub publication_date: Option<i64>,
    pub pages: Option<i64>,
    pub notes: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

// ✅ Implementa CrudModel trait - elimina necessità di get/list_all/delete custom
impl CrudModel for Content {
    const TABLE_NAME: &'static str = "contents";
    const ORDER_BY: &'static str = "name";
}

impl Content {
    /// Create a new content and save it to the database
    /// Returns the newly created content ID
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    ///
    /// # Returns
    /// * `Ok(i64)` - The ID of the newly inserted content
    /// * `Err(sqlx::Error)` - Database error if insertion fails
    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "INSERT INTO contents (
                name, original_title, type_id, publication_date, notes, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
            self.name,
            self.original_title,
            self.type_id,
            self.publication_date,
            self.notes,
            now,
            now
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// ❌ REMOVED: use `crud_get::<Content>(pool, id).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_get::<Content>(pool, id).await` instead"
    )]
    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Content>, sqlx::Error> {
        use crate::crud_get;
        crud_get::<Content>(pool, id).await
    }

    /// Update an existing content in the database
    /// Only updates fields that are part of the struct
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    ///
    /// # Returns
    /// * `Ok(u64)` - Number of rows affected
    /// * `Err(sqlx::Error)` - Database error if update fails
    pub async fn update(&self, pool: &sqlx::SqlitePool) -> Result<u64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "UPDATE contents SET
                name = ?, original_title = ?, type_id = ?, publication_date = ?, pages = ?, notes = ?, updated_at = ?
            WHERE id = ?",
            self.name,
            self.original_title,
            self.type_id,
            self.publication_date,
            self.pages,
            self.notes,
            now,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// ❌ REMOVED: use `crud_delete::<Content>(pool, id).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_delete::<Content>(pool, id).await` instead"
    )]
    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
        use crate::crud_delete;
        crud_delete::<Content>(pool, id).await
    }

    /// List all contents from the database ordered by name
    ///
    /// # Returns
    /// * `Ok(Vec<Content>)` - Vector of all contents
    /// * `Err(sqlx::Error)` - Database error if query fails
    ///
    /// ❌ REMOVED: use `crud_list_all::<Content>(pool).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_list_all::<Content>(pool).await` instead"
    )]
    pub async fn list_all(pool: &sqlx::SqlitePool) -> Result<Vec<Content>, sqlx::Error> {
        use crate::crud_list_all;
        crud_list_all::<Content>(pool).await
    }

    /// Search for contents by pattern matching multiple fields
    /// Searches in: name, original_title, notes
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    /// * `pattern` - Search pattern (will be wrapped with %)
    ///
    /// # Returns
    /// * `Ok(Vec<Content>)` - Matching contents ordered by name
    /// * `Err(sqlx::Error)` - Database error if query fails
    pub async fn search(
        pool: &sqlx::SqlitePool,
        pattern: &str,
    ) -> Result<Vec<Content>, sqlx::Error> {
        use crate::crud_search;
        crud_search::<Content>(pool, pattern, &["name", "original_title", "notes"]).await
    }

    /// Get or create a content by name
    /// If a content with the given name exists, return its ID
    /// Otherwise create a new content and return its ID
    pub async fn get_or_create_by_name(
        pool: &sqlx::SqlitePool,
        name: &str,
    ) -> Result<i64, sqlx::Error> {
        // Try to find existing content
        let search_result = Self::search(pool, name).await?;
        if let Some(existing) = search_result.first() {
            if existing.name.eq_ignore_ascii_case(name) {
                return Ok(existing.id.unwrap_or(0));
            }
        }

        // Create new content if not found
        let content = Content {
            id: None,
            name: name.to_string(),
            original_title: None,
            type_id: None,
            publication_date: None,
            pages: None,
            notes: None,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };
        content.save(pool).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_default() {
        let content = Content::default();
        assert_eq!(content.name, "");
        assert!(content.type_id.is_none());
    }

    #[test]
    fn test_crud_model_impl() {
        // Verify CrudModel trait is properly implemented
        assert_eq!(Content::TABLE_NAME, "contents");
        assert_eq!(Content::ORDER_BY, "name");
    }
}
