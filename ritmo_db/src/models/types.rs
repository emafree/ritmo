use sqlx::SqlitePool;
use ritmo_errors::RitmoResult;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Type {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
}

impl Type {
    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let rec = sqlx::query!(
            "INSERT INTO types (name, description) VALUES (?, ?)",
            self.name,
            self.description
            )
            .execute(pool)
            .await?;
        // Recupera l'ID appena inserito
        let id = rec.last_insert_rowid();
        Ok(id)
    }

    pub async fn get(id: i64, pool: &SqlitePool) -> RitmoResult<Option<Self>> {
        let result = sqlx::query_as!(
            Self, // Qui usiamo Self invece di Type
            "SELECT id, name, description, created_at FROM types WHERE id = ?",
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    pub async fn update(&self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE types SET name = ?, description = ? WHERE id = ?",
            self.name,
            self.description,
            self.id
            )
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM types WHERE id = ?",
            self.id
            )
            .execute(pool)
            .await?;
        Ok(())
    }
}
