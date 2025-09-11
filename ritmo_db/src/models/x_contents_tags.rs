use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct ContentTag {
    pub content_id: i64,
    pub tag_id: i64,
}

impl ContentTag {
    pub async fn save(pool: &sqlx::SqlitePool, new_link: &ContentTag) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO x_contents_tags (content_id, tag_id) VALUES (?, ?)",
            new_link.content_id,
            new_link.tag_id
            )
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete(
        pool: &sqlx::SqlitePool,
        content_id: i64,
        tag_id: i64,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM x_contents_tags WHERE content_id = ? AND tag_id = ?",
            content_id,
            tag_id
            )
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_by_content(
        pool: &sqlx::SqlitePool,
        content_id: i64,
    ) -> Result<Vec<ContentTag>, sqlx::Error> {
        let links =
            sqlx::query_as!(
                ContentTag,
                "SELECT * FROM x_contents_tags WHERE content_id = ?",
                content_id
                )
                .fetch_all(pool)
                .await?;
        Ok(links)
    }

    pub async fn list_by_tag(
        pool: &sqlx::SqlitePool,
        tag_id: i64,
    ) -> Result<Vec<ContentTag>, sqlx::Error> {
        let links = sqlx::query_as!(
            ContentTag,
            "SELECT * FROM x_contents_tags WHERE tag_id = ?",
            tag_id
            )
            .fetch_all(pool)
            .await?;
        Ok(links)
    }
}
