use chrono::Utc;
use ritmo_core::ContentDto;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Default)]
pub struct Content {
    /// Vale lo stesso che per Book, quando si immette un nuovo Content il suo Id è None, memorizzandolo viene assegnato.
    pub id: Option<i64>,
    pub name: String,
    pub original_title: Option<String>,
    pub type_id: Option<i64>,
    pub publication_date: Option<i64>,
    pub pages: Option<i64>,
    pub notes: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Content {
    pub fn from_dto(dto: &ContentDto) -> Self {
        let now = Utc::now().timestamp();

        Self {
            name: dto.name.clone(),
            original_title: dto.original_title.clone(),
            type_id: dto.type_id,
            publication_date: dto.publication_date,
            notes: dto.notes.clone(),
            created_at: now,
            ..Default::default()
        }
    }

    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "INSERT INTO contents (
                name, original_title, type_id, publication_date, notes, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
            self.name,
            self.original_title,
            self.type_id,
            self.publication_date,
            self.notes,
            now,
            now
            )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Content>, sqlx::Error> {
        let content = sqlx::query_as!(
            Content,
            "SELECT * FROM contents WHERE id = ?",
            id
            )
            .fetch_optional(pool)
            .await?;
        Ok(content)
    }

    pub async fn update(&self, pool: &sqlx::SqlitePool) -> Result<u64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "UPDATE contents SET
                name = ?, original_title = ?, type_id = ?, publication_date = ?, pages = ?, notes = ?, updated_at = ?
            WHERE id = ?",
            self.name,
            self.original_title,
            self.type_id,
            self.publication_date,
            self.pages,
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
            "DELETE FROM contents WHERE id = ?",
            id
            )
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_all(pool: &sqlx::SqlitePool) -> Result<Vec<Content>, sqlx::Error> {
        let all = sqlx::query_as!(
            Content,
            "SELECT * FROM contents ORDER BY name"
            )
            .fetch_all(pool)
            .await?;
        Ok(all)
    }

    pub async fn search(
        pool: &sqlx::SqlitePool,
        pattern: &str,
    ) -> Result<Vec<Content>, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let found = sqlx::query_as!(
            Content,
            "SELECT * FROM contents WHERE name LIKE ? OR original_title LIKE ? OR notes LIKE ? ORDER BY name",
            search_pattern,
            search_pattern,
            search_pattern
        )
        .fetch_all(pool)
        .await?;
        Ok(found)
    }
}
