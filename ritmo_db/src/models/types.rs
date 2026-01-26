use crate::i18n_trait::I18nDisplayable;
use ritmo_errors::RitmoResult;
use sqlx::FromRow;
use sqlx::SqlitePool;

/// Type with i18n support
/// Uses canonical keys (e.g., "type.novel") instead of translated strings
#[derive(Debug, Clone, FromRow)]
pub struct Type {
    pub id: Option<i64>,
    pub key: String,
    pub description: Option<String>,
    pub created_at: i64,
}

impl I18nDisplayable for Type {
    fn i18n_key(&self) -> &str {
        &self.key
    }
}

impl Type {
    /// Get the display name for this type in the current UI language
    /// Uses the i18n system to translate type keys (e.g., "type.novel" -> "Novel"/"Romanzo")
    pub fn display_name(&self) -> String {
        // Delegate to I18nDisplayable trait
        self.translate()
    }

    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let rec = sqlx::query!(
            "INSERT INTO types (key, description) VALUES (?, ?)",
            self.key,
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
            Self,
            "SELECT id, key, description, created_at FROM types WHERE id = ?",
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    pub async fn update(&self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE types SET key = ?, description = ? WHERE id = ?",
            self.key,
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

    /// Get type by key (e.g., "type.novel")
    pub async fn get_by_key(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let result = sqlx::query_as!(
            Self,
            "SELECT id, key, description, created_at FROM types WHERE key = ?",
            key
        )
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    /// Get or create type by key (e.g., "type.novel")
    /// Creates new type if it doesn't exist
    pub async fn get_or_create_by_key(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<i64, sqlx::Error> {
        if let Some(type_record) = Self::get_by_key(pool, key).await? {
            return Ok(type_record.id.unwrap_or(0));
        }
        let type_record = Type {
            id: None,
            key: key.to_string(),
            description: None,
            created_at: chrono::Utc::now().timestamp(),
        };
        type_record.save(pool).await
    }

    /// Legacy method for backward compatibility
    /// Use get_by_key instead for new code
    #[deprecated(since = "0.1.0", note = "Use get_by_key instead")]
    pub async fn get_by_name(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        Self::get_by_key(pool, key).await
    }

    /// Legacy method for backward compatibility
    #[deprecated(since = "0.1.0", note = "Use get_or_create_by_key instead")]
    pub async fn get_or_create_by_name(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<i64, sqlx::Error> {
        Self::get_or_create_by_key(pool, key).await
    }
}
