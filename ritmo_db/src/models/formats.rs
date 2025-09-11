use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Format {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
}

impl Format {
    pub async fn create(&self, pool: &sqlx::SqlitePool ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
                "INSERT INTO formats (name, description) VALUES (?, ?)",
                self.name,
                self.description
            )
            .execute(pool)
            .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Format>, sqlx::Error> {
        let result = sqlx::query_as!(
            Format,
            "SELECT id, name, description, created_at FROM formats WHERE id = ?",
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    pub async fn update(
        pool: &sqlx::SqlitePool,
        id: i64,
        name: &str,
        description: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE formats SET name = ?, description = ? WHERE id = ?",
            name,
            description,
            id
            )
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM formats WHERE id = ?",
            id
            )
            .execute(pool)
            .await?;
        Ok(())
    }
}
