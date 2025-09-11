use super::FullBook;
use sqlx::FromRow;
#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct BookContent {
    pub book_id: i64,
    pub content_id: i64,
}

impl BookContent {
    pub async fn create(pool: &sqlx::SqlitePool, new_link: &FullBook) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO x_books_contents (book_id, content_id) VALUES (?, ?)",
            new_link.book_content.book_id,
            new_link.book_content.content_id
            )
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete(
        pool: &sqlx::SqlitePool,
        book_id: i64,
        content_id: i64,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM x_books_contents WHERE book_id = ? AND content_id = ?",
            book_id,
            content_id
            )
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_by_book(
        pool: &sqlx::SqlitePool,
        book_id: i64,
    ) -> Result<Vec<BookContent>, sqlx::Error> {
        let links =
            sqlx::query_as!(
                BookContent, 
                "SELECT * FROM x_books_contents WHERE book_id = ?",
                book_id
                )
                .fetch_all(pool)
                .await?;
        Ok(links)
    }

    pub async fn list_by_content(
        pool: &sqlx::SqlitePool,
        content_id: i64,
    ) -> Result<Vec<BookContent>, sqlx::Error> {
        let links =
            sqlx::query_as!(
                BookContent, 
                "SELECT * FROM x_books_contents WHERE content_id = ?",
                content_id
                )
                .fetch_all(pool)
                .await?;
        Ok(links)
    }
}
