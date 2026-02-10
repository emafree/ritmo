use crate::GetOrCreateModel;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Default)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
    pub created_at: Option<i64>,
}

// ✅ Implement GetOrCreateModel trait - eliminates get_or_create_by_name duplication
impl GetOrCreateModel for Tag {
    type LookupKey = str;

    fn id(&self) -> Option<i64> {
        self.id
    }

    fn new_from_key(name: &str) -> Self {
        Tag {
            id: None,
            name: name.to_string(),
            created_at: None,
        }
    }

    async fn find_by_key(
        pool: &sqlx::SqlitePool,
        name: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        Self::get_by_name(pool, name).await
    }

    async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        self.save(pool).await
    }
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
    ///
    /// ❌ DEPRECATED: use `get_or_create::<Tag>(pool, name).await` instead
    /// This is now provided by the GetOrCreateModel trait through generic implementation
    #[deprecated(
        since = "0.1.0",
        note = "Use `get_or_create::<Tag>(pool, name).await` instead"
    )]
    pub async fn get_or_create_by_name(pool: &sqlx::SqlitePool, name: &str) -> Result<i64, sqlx::Error> {
        use crate::get_or_create;
        get_or_create::<Tag>(pool, name).await
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
