use crate::i18n_trait::I18nDisplayable;
use sqlx::FromRow;

/// Format with i18n support
/// Uses canonical keys (e.g., "format.epub") instead of translated strings
#[derive(Debug, Clone, FromRow)]
pub struct Format {
    pub id: Option<i64>,
    pub key: String,
    pub description: Option<String>,
    pub created_at: i64,
}

impl I18nDisplayable for Format {
    fn i18n_key(&self) -> &str {
        &self.key
    }
}

impl Format {
    /// Get the display name for this format in the current UI language
    /// Uses the i18n system to translate format keys (e.g., "format.epub" -> "EPUB (ebook)")
    pub fn display_name(&self) -> String {
        // Delegate to I18nDisplayable trait
        self.translate()
    }

    pub async fn create(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO formats (key, description) VALUES (?, ?)",
            self.key,
            self.description
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Format>, sqlx::Error> {
        let result = sqlx::query_as!(
            Format,
            "SELECT id, key, description, created_at FROM formats WHERE id = ?",
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    pub async fn update(
        pool: &sqlx::SqlitePool,
        id: i64,
        key: &str,
        description: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE formats SET key = ?, description = ? WHERE id = ?",
            key,
            description,
            id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM formats WHERE id = ?", id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Get format by key (e.g., "format.epub")
    pub async fn get_by_key(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<Option<Format>, sqlx::Error> {
        let result = sqlx::query_as!(
            Format,
            "SELECT id, key, description, created_at FROM formats WHERE key = ?",
            key
        )
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    /// Get or create format by key (e.g., "format.epub")
    /// Creates new format if it doesn't exist
    pub async fn get_or_create_by_key(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<i64, sqlx::Error> {
        if let Some(format) = Self::get_by_key(pool, key).await? {
            return Ok(format.id.unwrap_or(0));
        }
        let format = Format {
            id: None,
            key: key.to_string(),
            description: None,
            created_at: chrono::Utc::now().timestamp(),
        };
        format.create(pool).await
    }

    /// Legacy method for backward compatibility
    /// Use get_by_key instead for new code
    #[deprecated(since = "0.1.0", note = "Use get_by_key instead")]
    pub async fn get_by_name(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<Option<Format>, sqlx::Error> {
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
