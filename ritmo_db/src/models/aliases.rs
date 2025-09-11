use ritmo_core::dto::AliasDto;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Default)]
pub struct Alias {
    pub id: Option<i64>,
    pub name: String,
    pub person_id: Option<i64>,
    pub alias_normalized: Option<String>,
    pub confidence: f64,
    pub created_at: i64,
}

impl Alias {
    /// placeholder
    pub fn from_dto(_dto: &mut AliasDto) -> Self {
        Alias::default()
    }

    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "INSERT INTO aliases (name, person_id, alias_normalized, created_at) VALUES (?, ?, ?, ?)",
            self.name,
            self.person_id, // Assumendo che person_id sia già Option<i64>
            self.alias_normalized,
            now
        )
        .execute(pool)
        .await?;

        Ok(result.last_insert_rowid())
    }
    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Alias>, sqlx::Error> {
        let alias = sqlx::query_as!(
            Alias,
            "SELECT * FROM aliases WHERE id = ?",
            id
        )
        .fetch_optional(pool)
        .await?;        
    Ok(alias)
    }

    pub async fn get_by_person_and_name(
        pool: &sqlx::SqlitePool,
        person_id: i64,
        name: &str,
    ) -> Result<Option<Alias>, sqlx::Error> {
        let alias = sqlx::query_as!(
            Alias,
            "SELECT * FROM aliases WHERE person_id = ? AND name = ?",
            person_id,
            name,
            )
            .fetch_optional(pool)
            .await?;
        Ok(alias)
    }

    pub async fn update(&self, pool: &sqlx::SqlitePool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE aliases SET name = ?, person_id = ?, alias_normalized = ?, confidence = ? WHERE id = ?", 
            self.name,
            self.person_id,
            self.alias_normalized,
            self.confidence,
            self.id
            )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM aliases WHERE id = ?",
            id,
            )
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_by_person(
        pool: &sqlx::SqlitePool,
        person_id: i64,
    ) -> Result<Vec<Alias>, sqlx::Error> {
        let aliases = sqlx::query_as!(
            Alias,
            "SELECT * FROM aliases WHERE person_id = ? ORDER BY name",
            person_id,
            )
            .fetch_all(pool)
            .await?;
        Ok(aliases)
    }

    pub async fn search(pool: &sqlx::SqlitePool, pattern: &str) -> Result<Vec<Alias>, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let aliases = sqlx::query_as!(
            Alias,
            "SELECT * FROM aliases WHERE name LIKE ? OR alias_normalized LIKE ? ORDER BY name",
            search_pattern,
            search_pattern
            )
        .fetch_all(pool)
        .await?;
        Ok(aliases)
    }
}
