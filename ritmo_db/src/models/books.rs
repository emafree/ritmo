#[derive(Debug, Clone, Default)]
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
    pub pages: Option<i64>,
    pub notes: Option<String>,
    pub has_cover: i64,
    pub has_paper: i64,
    pub file_link: Option<String>,
    pub file_size: Option<i64>,
    pub file_hash: Option<String>,
    pub created_at: i64,
}

impl Book {
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

    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Book>, sqlx::Error> {
        let book = sqlx::query_as!(Book, "SELECT * FROM books WHERE id = ?", id)
            .fetch_optional(pool)
            .await?;
        Ok(book)
    }

    pub async fn update(&self, pool: &sqlx::SqlitePool) -> Result<u64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "UPDATE books SET
                name = ?, original_title = ?, publisher_id = ?, format_id = ?, series_id = ?,
                series_index = ?, publication_date = ?, last_modified_date = ?, isbn = ?,
                pages = ?, notes = ?, has_cover = ?, has_paper = ?, file_link = ?,
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
            self.pages,
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

    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM books WHERE id = ?", id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_all(pool: &sqlx::SqlitePool) -> Result<Vec<Book>, sqlx::Error> {
        let all = sqlx::query_as!(Book, "SELECT * FROM books ORDER BY name")
            .fetch_all(pool)
            .await?;
        Ok(all)
    }

    pub async fn search(pool: &sqlx::SqlitePool, pattern: &str) -> Result<Vec<Book>, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let found = sqlx::query_as!(
            Book,
            "SELECT * FROM books WHERE name LIKE ? OR original_title LIKE ? OR notes LIKE ? OR isbn LIKE ? ORDER BY name",
            search_pattern,
            search_pattern,
            search_pattern,
            search_pattern
            )
        .fetch_all(pool)
        .await?;
        Ok(found)
    }

}
