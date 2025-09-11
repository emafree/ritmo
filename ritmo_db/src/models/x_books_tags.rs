use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct BookTag {
    pub book_id: i64,
    pub tag_id: i64,
}

impl BookTag {
    pub async fn create(pool: &sqlx::SqlitePool, new_link: &BookTag) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO x_books_tags (book_id, tag_id) VALUES (?, ?)",
            new_link.book_id,
            new_link.tag_id
            )
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete(
        pool: &sqlx::SqlitePool,
        book_id: i64,
        tag_id: i64,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM x_books_tags WHERE book_id = ? AND tag_id = ?",
            book_id,
            tag_id
            )
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_by_book(
        pool: &sqlx::SqlitePool,
        book_id: i64,
    ) -> Result<Vec<BookTag>, sqlx::Error> {
        let links = sqlx::query_as!(
            BookTag,
            "SELECT * FROM x_books_tags WHERE book_id = ?",
            book_id
            )
            .fetch_all(pool)
            .await?;
        Ok(links)
    }

    pub async fn list_by_tag(
        pool: &sqlx::SqlitePool,
        tag_id: i64,
    ) -> Result<Vec<BookTag>, sqlx::Error> {
        let links = sqlx::query_as!(
            BookTag,
            "SELECT * FROM x_books_tags WHERE tag_id = ?",
            tag_id
            )
            .fetch_all(pool)
            .await?;
        Ok(links)
    }
}
