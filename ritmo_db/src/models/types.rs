use ritmo_errors::RitmoResult;
use sqlx::FromRow;
use sqlx::SqlitePool;

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
        sqlx::query!("DELETE FROM types WHERE id = ?", self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn get_by_name(
        pool: &sqlx::SqlitePool,
        name: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let result = sqlx::query_as!(
            Self,
            "SELECT id, name, description, created_at FROM types WHERE name = ?",
            name
        )
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    pub async fn get_or_create_by_name(
        pool: &sqlx::SqlitePool,
        name: &str,
    ) -> Result<i64, sqlx::Error> {
        if let Some(type_record) = Self::get_by_name(pool, name).await? {
            return Ok(type_record.id.unwrap_or(0));
        }
        let type_record = Type {
            id: None,
            name: name.to_string(),
            description: None,
            created_at: chrono::Utc::now().timestamp(),
        };
        type_record.save(pool).await
    }
}
