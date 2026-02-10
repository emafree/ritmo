use crate::crud_trait::CrudModel;
use crate::i18n_trait::I18nDisplayable;
use crate::GetOrCreateModel;
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

impl I18nDisplayable for Role {
    fn i18n_key(&self) -> &str {
        &self.key
    }
}

// ✅ Implementa CrudModel trait - elimina necessità di get/list_all/delete custom
impl CrudModel for Role {
    const TABLE_NAME: &'static str = "roles";
    const ORDER_BY: &'static str = "key";
}

// ✅ Implement GetOrCreateModel trait
impl GetOrCreateModel for Role {
    type LookupKey = str;

    fn id(&self) -> Option<i64> {
        self.id
    }

    fn new_from_key(key: &str) -> Self {
        Role {
            id: None,
            key: key.to_string(),
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    async fn find_by_key(pool: &sqlx::SqlitePool, key: &str) -> Result<Option<Self>, sqlx::Error> {
        Self::get_by_key(pool, key).await
    }

    async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        self.save(pool).await
    }
}

impl Role {
    /// Get the display name for this role in the current UI language
    /// Uses the i18n system to translate role keys (e.g., "role.author" -> "Author"/"Autore")
    pub fn display_name(&self) -> String {
        // Delegate to I18nDisplayable trait
        self.translate()
    }

    /// Create a new role and save it to the database
    /// Returns the newly created role ID
    ///
    /// # Arguments
    /// * `pool` - SQLite connection pool
    ///
    /// # Returns
    /// * `Ok(i64)` - The ID of the newly inserted role
    /// * `Err(sqlx::Error)` - Database error if insertion fails
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

    /// ❌ REMOVED: use `crud_get::<Role>(pool, id).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_get::<Role>(pool, id).await` instead"
    )]
    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Role>, sqlx::Error> {
        use crate::crud_get;
        crud_get::<Role>(pool, id).await
    }

    /// Get all roles ordered by key
    pub async fn get_all(pool: &sqlx::SqlitePool) -> Result<Vec<Role>, sqlx::Error> {
        use crate::crud_list_all;
        crud_list_all::<Role>(pool).await
    }

    /// Get role by key (e.g., "role.author")
    pub async fn get_by_key(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<Option<Role>, sqlx::Error> {
        let result = sqlx::query_as!(
            Role,
            "SELECT id, key, created_at FROM roles WHERE key = ?",
            key
        )
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    /// Update a role's key
    pub async fn update(pool: &sqlx::SqlitePool, id: i64, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE roles SET key = ? WHERE id = ?", key, id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// ❌ REMOVED: use `crud_delete::<Role>(pool, id).await` instead
    /// This is now provided by the CrudModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `crud_delete::<Role>(pool, id).await` instead"
    )]
    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        use crate::crud_delete;
        crud_delete::<Role>(pool, id).await?;
        Ok(())
    }

    /// ❌ DEPRECATED: use `get_or_create::<Role>(pool, key).await` instead
    #[deprecated(
        since = "0.1.0",
        note = "Use `get_or_create::<Role>(pool, key).await` instead"
    )]
    pub async fn get_or_create_by_key(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<i64, sqlx::Error> {
        use crate::get_or_create;
        get_or_create::<Role>(pool, key).await
    }

    /// Legacy method for backward compatibility
    /// Use get_by_key instead for new code
    #[deprecated(since = "0.1.0", note = "Use get_by_key instead")]
    pub async fn get_by_name(
        pool: &sqlx::SqlitePool,
        key: &str,
    ) -> Result<Option<Role>, sqlx::Error> {
        Self::get_by_key(pool, key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crud_model_impl() {
        assert_eq!(Role::TABLE_NAME, "roles");
        assert_eq!(Role::ORDER_BY, "key");
    }
}
