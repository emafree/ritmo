use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct ContentLanguage {
    pub content_id: i64,
    pub language_id: i64,
}

#[derive(Debug)]
pub struct NewContentLanguage {
    pub content_id: i64,
    pub language_id: i64,
}

impl ContentLanguage {
    pub async fn create(pool: &sqlx::SqlitePool, new_link: &NewContentLanguage) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO x_contents_languages (content_id, language_id) VALUES (?, ?)",
            new_link.content_id,
            new_link.language_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete(pool: &sqlx::SqlitePool, content_id: i64, language_id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM x_contents_languages WHERE content_id = ? AND language_id = ?",
            content_id,
            language_id
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_by_content(pool: &sqlx::SqlitePool, content_id: i64) -> Result<Vec<ContentLanguage>, sqlx::Error> {
        let links = sqlx::query_as!(
            ContentLanguage,
            "SELECT * FROM x_contents_languages WHERE content_id = ?",
            content_id
        )
        .fetch_all(pool)
        .await?;
        Ok(links)
    }

    pub async fn list_by_language(pool: &sqlx::SqlitePool, language_id: i64) -> Result<Vec<ContentLanguage>, sqlx::Error> {
        let links = sqlx::query_as!(
            ContentLanguage,
            "SELECT * FROM x_contents_languages WHERE language_id = ?",
            language_id
        )
        .fetch_all(pool)
        .await?;
        Ok(links)
    }
}