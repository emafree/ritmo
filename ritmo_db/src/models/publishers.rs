use ritmo_core::PublisherDto;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Default)]
pub struct Publisher {
    pub id: Option<i64>,
    pub name: String,
    pub country: Option<String>,
    pub website: Option<String>,
    pub notes: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Publisher {
    pub fn from_dto(_dto: &PublisherDto) -> Self {
        Publisher::default()
    }

    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "INSERT INTO publishers (name, country, website, notes, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        self.name,
        self.country,
        self.website,
        self.notes,
        now,
        now
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Publisher>, sqlx::Error> {
        let publisher = sqlx::query_as!(
            Publisher,
            "SELECT * FROM publishers WHERE id = ?",
            id
            )
            .fetch_optional(pool)
            .await?;
        Ok(publisher)
    }

    pub async fn get_by_name(
        pool: &sqlx::SqlitePool,
        name: &str,
    ) -> Result<Option<Publisher>, sqlx::Error> {
        let publisher = sqlx::query_as!(
            Publisher,
            "SELECT * FROM publishers WHERE name = ?",
            name
            )
            .fetch_optional(pool)
            .await?;
        Ok(publisher)
    }

    pub async fn update(&self, pool: &sqlx::SqlitePool) -> Result<u64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "UPDATE publishers SET name = ?, country = ?, website = ?, notes = ?, updated_at = ? WHERE id = ?",
        self.name,
        self.country,
        self.website,
        self.notes,
        now,
        self.id
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM publishers WHERE id = ?",
            id
            )
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_all(pool: &sqlx::SqlitePool) -> Result<Vec<Publisher>, sqlx::Error> {
        let publishers = sqlx::query_as!(
            Publisher,
            "SELECT * FROM publishers ORDER BY name"
            )
            .fetch_all(pool)
            .await?;
        Ok(publishers)
    }

    pub async fn search(
        pool: &sqlx::SqlitePool,
        pattern: &str,
    ) -> Result<Vec<Publisher>, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let publishers = sqlx::query_as!(
            Publisher,
            "SELECT * FROM publishers WHERE name LIKE ? OR country LIKE ? OR website LIKE ? OR notes LIKE ? ORDER BY name",
        search_pattern,
        search_pattern,
        search_pattern,
        search_pattern
        )
        .fetch_all(pool)
        .await?;
        Ok(publishers)
    }
}
