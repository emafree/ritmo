use ritmo_errors::RitmoResult;
use sqlx::FromRow;

/// Role with i18n support
/// Uses canonical keys (e.g., "role.author") instead of translated strings
#[derive(Debug, Clone, FromRow)]
pub struct Role {
    pub id: Option<i64>,
    pub key: String,
    pub created_at: i64,
}

impl Role {
    /// Get the display name for this role in the current UI language
    /// Uses the i18n system to translate role keys (e.g., "role.author" -> "Author"/"Autore")
    pub fn display_name(&self) -> String {
        use rust_i18n::t;

        // Map role key to translation key
        // "role.author" -> "db.role.author"
        let translation_key = format!("db.{}", self.key);
        t!(&translation_key).to_string()
    }

    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let rec = sqlx::query!(
            "INSERT INTO roles (key, created_at) VALUES (?, ?)",
            self.key,
            self.created_at
        )
        .execute(pool)
        .await?;
        let id = rec.last_insert_rowid();
        Ok(id)
    }

    /// Get role by ID
    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> RitmoResult<Option<Role>> {
        let result = sqlx::query_as!(
            Role,
            "SELECT id, key, created_at FROM roles WHERE id = ?",
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    /// Get all roles ordered by key
    pub async fn get_all(pool: &sqlx::SqlitePool) -> RitmoResult<Vec<Role>> {
        let roles = sqlx::query_as!(
            Role,
            "SELECT id, key, created_at FROM roles ORDER BY key"
        )
        .fetch_all(pool)
        .await?;
        Ok(roles)
    }

    /// Get role by key (e.g., "role.author")
    pub async fn get_by_key(pool: &sqlx::SqlitePool, key: &str) -> RitmoResult<Option<Role>> {
        let result = sqlx::query_as!(
            Role,
            "SELECT id, key, created_at FROM roles WHERE key = ?",
            key
        )
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    pub async fn update(
        pool: &sqlx::SqlitePool,
        id: i64,
        key: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE roles SET key = ? WHERE id = ?", key, id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM roles WHERE id = ?", id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Legacy method for backward compatibility
    /// Use get_by_key instead for new code
    #[deprecated(since = "0.1.0", note = "Use get_by_key instead")]
    pub async fn get_by_name(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<Option<Role>, sqlx::Error> {
        Self::get_by_key(pool, key).await.map_err(|e| {
            sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))
        })
    }

    /// Get or create role by key (e.g., "role.author")
    /// Creates new role if it doesn't exist
    pub async fn get_or_create_by_key(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<i64, sqlx::Error> {
        if let Ok(Some(role)) = Self::get_by_key(pool, key).await {
            return Ok(role.id.unwrap_or(0));
        }
        let role = Role {
            id: None,
            key: key.to_string(),
            created_at: chrono::Utc::now().timestamp(),
        };
        role.save(pool).await
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
