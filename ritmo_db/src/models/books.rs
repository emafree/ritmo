use crate::crud_trait::CrudModel;
use sqlx::FromRow;

#[derive(Debug, Clone, Default, FromRow)]
pub struct Book {
    /// Il campo 'id' è Option perchè quando il libro viene creato il suo valore è None, e viene creato alla memorizzazione.
    pub id: Option<i64>,
    pub name: String,
    pub original_title: Option<String>,
    pub publisher_id: Option<i64>,
    pub format_id: Option<i64>,
    pub series_id: Option<i64>,
    pub series_index: Option<i64>,
    pub publication_date: Option<i64>,
    pub last_modified_date: i64,
    pub isbn: Option<String>,
    pub notes: Option<String>,
    pub has_cover: i64,
    pub has_paper: i64,
    pub file_link: Option<String>,
    pub file_size: Option<i64>,
    pub file_hash: Option<String>,
    pub created_at: i64,
}

// ✅ Implementa CrudModel trait - elimina necessità di get/list_all/delete custom
impl CrudModel for Book {
    const TABLE_NAME: &'static str = "books";
    const ORDER_BY: &'static str = "name";
}

impl Book {
    /// Create a new book and save it to the database
    /// Returns the newly created book ID
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    ///
    /// # Returns
    /// * `Ok(i64)` - The ID of the newly inserted book
    /// * `Err(sqlx::Error)` - Database error if insertion fails
    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "INSERT INTO books (
                name, original_title, publisher_id, format_id, series_id, series_index,
                publication_date, last_modified_date, isbn, notes,
                has_cover, has_paper, file_link, file_size, file_hash, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            self.name,
            self.original_title,
            self.publisher_id,
            self.format_id,
            self.series_id,
            self.series_index,
            self.publication_date,
            now,
            self.isbn,
            self.notes,
            self.has_cover,
            self.has_paper,
            self.file_link,
            self.file_size,
            self.file_hash,
            now
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// ❌ REMOVED: use `crud_get::<Book>(pool, id).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_get::<Book>(pool, id).await` instead"
    )]
    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Book>, sqlx::Error> {
        use crate::crud_get;
        crud_get::<Book>(pool, id).await
    }

    /// Update an existing book in the database
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
            "UPDATE books SET
                name = ?, original_title = ?, publisher_id = ?, format_id = ?, series_id = ?,
                series_index = ?, publication_date = ?, last_modified_date = ?, isbn = ?,
                notes = ?, has_cover = ?, has_paper = ?, file_link = ?,
                file_size = ?, file_hash = ?
            WHERE id = ?",
            self.name,
            self.original_title,
            self.publisher_id,
            self.format_id,
            self.series_id,
            self.series_index,
            self.publication_date,
            now,
            self.isbn,
            self.notes,
            self.has_cover,
            self.has_paper,
            self.file_link,
            self.file_size,
            self.file_hash,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// ❌ REMOVED: use `crud_delete::<Book>(pool, id).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_delete::<Book>(pool, id).await` instead"
    )]
    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
        use crate::crud_delete;
        crud_delete::<Book>(pool, id).await
    }

    /// List all books from the database ordered by name
    ///
    /// # Returns
    /// * `Ok(Vec<Book>)` - Vector of all books
    /// * `Err(sqlx::Error)` - Database error if query fails
    ///
    /// ❌ REMOVED: use `crud_list_all::<Book>(pool).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_list_all::<Book>(pool).await` instead"
    )]
    pub async fn list_all(pool: &sqlx::SqlitePool) -> Result<Vec<Book>, sqlx::Error> {
        use crate::crud_list_all;
        crud_list_all::<Book>(pool).await
    }

    /// Search for books by pattern matching multiple fields
    /// Searches in: name, original_title, notes, isbn
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    /// * `pattern` - Search pattern (will be wrapped with %)
    ///
    /// # Returns
    /// * `Ok(Vec<Book>)` - Matching books ordered by name
    /// * `Err(sqlx::Error)` - Database error if query fails
    pub async fn search(pool: &sqlx::SqlitePool, pattern: &str) -> Result<Vec<Book>, sqlx::Error> {
        use crate::crud_search;
        crud_search::<Book>(pool, pattern, &["name", "original_title", "notes", "isbn"]).await
    }

    /// Get or create a book by name
    /// If a book with the given name exists, return its ID
    /// Otherwise create a new book and return its ID
    pub async fn get_or_create_by_name(
        pool: &sqlx::SqlitePool,
        name: &str,
    ) -> Result<i64, sqlx::Error> {
        // Try to find existing book
        let search_result = Self::search(pool, name).await?;
        if let Some(existing) = search_result.first() {
            if existing.name.eq_ignore_ascii_case(name) {
                return Ok(existing.id.unwrap_or(0));
            }
        }

        // Create new book if not found
        let book = Book {
            id: None,
            name: name.to_string(),
            original_title: None,
            publisher_id: None,
            format_id: None,
            series_id: None,
            series_index: None,
            publication_date: None,
            last_modified_date: chrono::Utc::now().timestamp(),
            isbn: None,
            notes: None,
            has_cover: 0,
            has_paper: 0,
            file_link: None,
            file_size: None,
            file_hash: None,
            created_at: chrono::Utc::now().timestamp(),
        };
        book.save(pool).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_default() {
        let book = Book::default();
        assert_eq!(book.name, "");
        assert_eq!(book.has_cover, 0);
        assert_eq!(book.has_paper, 0);
    }

    #[test]
    fn test_crud_model_impl() {
        // Verify CrudModel trait is properly implemented
        assert_eq!(Book::TABLE_NAME, "books");
        assert_eq!(Book::ORDER_BY, "name");
    }
}
