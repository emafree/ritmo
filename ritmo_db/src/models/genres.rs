use crate::crud_trait::CrudModel;
use crate::GetOrCreateModel;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Genre {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

// ✅ Implementa CrudModel trait - elimina necessità di get/list_all/delete custom
impl CrudModel for Genre {
    const TABLE_NAME: &'static str = "genres";
    const ORDER_BY: &'static str = "name";
}

// ✅ Implement GetOrCreateModel trait
impl GetOrCreateModel for Genre {
    type LookupKey = str;

    fn id(&self) -> Option<i64> {
        self.id
    }

    fn new_from_key(key: &str) -> Self {
        Genre {
            id: None,
            name: key.to_string(),
            description: None,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        }
    }

    async fn find_by_key(pool: &sqlx::SqlitePool, key: &str) -> Result<Option<Self>, sqlx::Error> {
        Self::get_by_name(pool, key).await
    }

    async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        self.save(pool).await
    }
}

impl Genre {
    /// Create a new genre and save it to the database
    /// Returns the newly created genre ID
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    ///
    /// # Returns
    /// * `Ok(i64)` - The ID of the newly inserted genre
    /// * `Err(sqlx::Error)` - Database error if insertion fails
    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "INSERT INTO genres (name, description, created_at, updated_at)
             VALUES (?, ?, ?, ?)",
            self.name,
            self.description,
            now,
            now
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// ❌ REMOVED: use `crud_get::<Genre>(pool, id).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_get::<Genre>(pool, id).await` instead"
    )]
    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Genre>, sqlx::Error> {
        use crate::crud_get;
        crud_get::<Genre>(pool, id).await
    }

    /// Get a genre by name
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    /// * `name` - Name to search for (exact match)
    ///
    /// # Returns
    /// * `Ok(Option<Genre>)` - The genre if found
    /// * `Err(sqlx::Error)` - Database error if query fails
    pub async fn get_by_name(
        pool: &sqlx::SqlitePool,
        name: &str,
    ) -> Result<Option<Genre>, sqlx::Error> {
        let genre = sqlx::query_as!(Genre, "SELECT * FROM genres WHERE name = ?", name)
            .fetch_optional(pool)
            .await?;
        Ok(genre)
    }

    /// Update an existing genre in the database
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
            "UPDATE genres SET name = ?, description = ?, updated_at = ? WHERE id = ?",
            self.name,
            self.description,
            now,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// ❌ REMOVED: use `crud_delete::<Genre>(pool, id).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_delete::<Genre>(pool, id).await` instead"
    )]
    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
        use crate::crud_delete;
        crud_delete::<Genre>(pool, id).await
    }

    /// List all genres from the database ordered by name
    ///
    /// # Returns
    /// * `Ok(Vec<Genre>)` - Vector of all genres
    /// * `Err(sqlx::Error)` - Database error if query fails
    ///
    /// ❌ REMOVED: use `crud_list_all::<Genre>(pool).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_list_all::<Genre>(pool).await` instead"
    )]
    pub async fn list_all(pool: &sqlx::SqlitePool) -> Result<Vec<Genre>, sqlx::Error> {
        use crate::crud_list_all;
        crud_list_all::<Genre>(pool).await
    }

    /// Search for genres by pattern matching multiple fields
    /// Searches in: name, description
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    /// * `pattern` - Search pattern (will be wrapped with %)
    ///
    /// # Returns
    /// * `Ok(Vec<Genre>)` - Matching genres ordered by name
    /// * `Err(sqlx::Error)` - Database error if query fails
    pub async fn search(
        pool: &sqlx::SqlitePool,
        pattern: &str,
    ) -> Result<Vec<Genre>, sqlx::Error> {
        use crate::crud_search;
        crud_search::<Genre>(pool, pattern, &["name", "description"]).await
    }

    /// ❌ DEPRECATED: use `get_or_create::<Genre>(pool, name).await` instead
    #[deprecated(
        since = "0.1.0",
        note = "Use `get_or_create::<Genre>(pool, name).await` instead"
    )]
    pub async fn get_or_create_by_name(
        pool: &sqlx::SqlitePool,
        name: &str,
    ) -> Result<i64, sqlx::Error> {
        use crate::get_or_create;
        get_or_create::<Genre>(pool, name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genre_default() {
        let genre = Genre {
            id: None,
            name: "Science Fiction".to_string(),
            description: Some("Science fiction and speculative fiction".to_string()),
            created_at: 0,
            updated_at: 0,
        };
        assert_eq!(genre.name, "Science Fiction");
    }

    #[test]
    fn test_crud_model_impl() {
        // Verify CrudModel trait is properly implemented
        assert_eq!(Genre::TABLE_NAME, "genres");
        assert_eq!(Genre::ORDER_BY, "name");
    }
}
