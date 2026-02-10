use crate::crud_trait::CrudModel;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Series {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub total_books: Option<i64>,
    pub completed: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

// ✅ Implementa CrudModel trait - elimina necessità di get/list_all/delete custom
impl CrudModel for Series {
    const TABLE_NAME: &'static str = "series";
    const ORDER_BY: &'static str = "name";
}

impl Series {
    /// Create a new series and save it to the database
    /// Returns the newly created series ID
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    ///
    /// # Returns
    /// * `Ok(i64)` - The ID of the newly inserted series
    /// * `Err(sqlx::Error)` - Database error if insertion fails
    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "INSERT INTO series (name, description, total_books, completed, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            self.name,
            self.description,
            self.total_books,
            self.completed,
            now,
            now
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// ❌ REMOVED: use `crud_get::<Series>(pool, id).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_get::<Series>(pool, id).await` instead"
    )]
    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Series>, sqlx::Error> {
        use crate::crud_get;
        crud_get::<Series>(pool, id).await
    }

    /// Get a series by name
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    /// * `name` - Name to search for (exact match)
    ///
    /// # Returns
    /// * `Ok(Option<Series>)` - The series if found
    /// * `Err(sqlx::Error)` - Database error if query fails
    pub async fn get_by_name(
        pool: &sqlx::SqlitePool,
        name: &str,
    ) -> Result<Option<Series>, sqlx::Error> {
        let series = sqlx::query_as!(Series, "SELECT * FROM series WHERE name = ?", name)
            .fetch_optional(pool)
            .await?;
        Ok(series)
    }

    /// Update an existing series in the database
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
            "UPDATE series SET name = ?, description = ?, total_books = ?, completed = ?, updated_at = ? WHERE id = ?",
            self.name,
            self.description,
            self.total_books,
            self.completed,
            now,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// ❌ REMOVED: use `crud_delete::<Series>(pool, id).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_delete::<Series>(pool, id).await` instead"
    )]
    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
        use crate::crud_delete;
        crud_delete::<Series>(pool, id).await
    }

    /// List all series from the database ordered by name
    ///
    /// # Returns
    /// * `Ok(Vec<Series>)` - Vector of all series
    /// * `Err(sqlx::Error)` - Database error if query fails
    ///
    /// ❌ REMOVED: use `crud_list_all::<Series>(pool).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_list_all::<Series>(pool).await` instead"
    )]
    pub async fn list_all(pool: &sqlx::SqlitePool) -> Result<Vec<Series>, sqlx::Error> {
        use crate::crud_list_all;
        crud_list_all::<Series>(pool).await
    }

    /// Search for series by pattern matching multiple fields
    /// Searches in: name, description
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    /// * `pattern` - Search pattern (will be wrapped with %)
    ///
    /// # Returns
    /// * `Ok(Vec<Series>)` - Matching series ordered by name
    /// * `Err(sqlx::Error)` - Database error if query fails
    pub async fn search(
        pool: &sqlx::SqlitePool,
        pattern: &str,
    ) -> Result<Vec<Series>, sqlx::Error> {
        use crate::crud_search;
        crud_search::<Series>(pool, pattern, &["name", "description"]).await
    }

    /// Get or create a series by name
    /// If a series with the given name exists, return its ID
    /// Otherwise create a new series and return its ID
    pub async fn get_or_create_by_name(
        pool: &sqlx::SqlitePool,
        name: &str,
    ) -> Result<i64, sqlx::Error> {
        // Try to find existing series by exact name match first
        if let Some(series) = Self::get_by_name(pool, name).await? {
            return Ok(series.id.unwrap_or(0));
        }

        // Create new series if not found
        let series = Series {
            id: None,
            name: name.to_string(),
            description: None,
            total_books: None,
            completed: 0,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };
        series.save(pool).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_series_default() {
        let series = Series {
            id: None,
            name: "Test Series".to_string(),
            description: None,
            total_books: None,
            completed: 0,
            created_at: 0,
            updated_at: 0,
        };
        assert_eq!(series.name, "Test Series");
        assert_eq!(series.completed, 0);
    }

    #[test]
    fn test_crud_model_impl() {
        // Verify CrudModel trait is properly implemented
        assert_eq!(Series::TABLE_NAME, "series");
        assert_eq!(Series::ORDER_BY, "name");
    }
}
