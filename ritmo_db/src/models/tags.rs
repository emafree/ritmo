use chrono::Utc;
use ritmo_core::ContentDto;
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
        let result =
            sqlx::query!(
                "INSERT INTO tags (name, created_at) VALUES (?, ?)",
                self.name,
                now
                )
                .execute(pool)
                .await?;
        Ok(result.last_insert_rowid())
    }

    pub fn from_dto(content_dto: &ContentDto) -> Self {
        let now = Utc::now().timestamp();

        Self {
            id: None,
            name: content_dto.name.clone(),
            created_at: Some(now),
        }
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

    pub async fn update(pool: &sqlx::SqlitePool, id: i64, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE tags SET name = ? WHERE id = ?",
            name,
            id
            )
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM tags WHERE id = ?",
            id
            )
            .execute(pool)
            .await?;
        Ok(())
    }
}
