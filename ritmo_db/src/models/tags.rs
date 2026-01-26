use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Default)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
    pub created_at: Option<i64>,
}

impl Tag {
    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "INSERT INTO tags (name, created_at) VALUES (?, ?)",
            self.name,
            now
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Tag>, sqlx::Error> {
        let result = sqlx::query_as!(
            Tag,
            "SELECT id, name, created_at FROM tags WHERE id = ?",
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    pub async fn get_by_name(pool: &sqlx::SqlitePool, name: &str) -> Result<Option<Tag>, sqlx::Error> {
        let result = sqlx::query_as!(
            Tag,
            "SELECT id, name, created_at FROM tags WHERE name = ? LIMIT 1",
            name
        )
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    /// Get or create a tag by name, returning the tag ID
    pub async fn get_or_create_by_name(pool: &sqlx::SqlitePool, name: &str) -> Result<i64, sqlx::Error> {
        if let Some(tag) = Self::get_by_name(pool, name).await? {
            return Ok(tag.id.unwrap_or(0));
        }
        let tag = Tag {
            id: None,
            name: name.to_string(),
            created_at: None,
        };
        tag.save(pool).await
    }

    pub async fn update(pool: &sqlx::SqlitePool, id: i64, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE tags SET name = ? WHERE id = ?", name, id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM tags WHERE id = ?", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
