use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Series {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub total_books: Option<i64>,
    pub completed: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Series {
    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "INSERT INTO series (name, description, total_books, completed, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            self.name,
            self.description,
            self.total_books,
            self.completed,
            now,
            now
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Series>, sqlx::Error> {
        let series = sqlx::query_as!(
            Series,
            "SELECT * FROM series WHERE id = ?",
            id
            )
            .fetch_optional(pool)
            .await?;
        Ok(series)
    }

    pub async fn get_by_name(
        pool: &sqlx::SqlitePool,
        name: &str,
    ) -> Result<Option<Series>, sqlx::Error> {
        let series = sqlx::query_as!(
            Series,
            "SELECT * FROM series WHERE name = ?",
            name
            )
            .fetch_optional(pool)
            .await?;
        Ok(series)
    }

    pub async fn update(&self, pool: &sqlx::SqlitePool) -> Result<u64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "UPDATE series SET name = ?, description = ?, total_books = ?, completed = ?, updated_at = ? WHERE id = ?",
            self.name,
            self.description,
            self.total_books,
            self.completed,
            now,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM series WHERE id = ?",
            id
            )
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_all(pool: &sqlx::SqlitePool) -> Result<Vec<Series>, sqlx::Error> {
        let all = sqlx::query_as!(
            Series,
            "SELECT * FROM series ORDER BY name"
            )
            .fetch_all(pool)
            .await?;
        Ok(all)
    }

    pub async fn search(pool: &sqlx::SqlitePool, pattern: &str) -> Result<Vec<Series>, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let found = sqlx::query_as!(
            Series,
            "SELECT * FROM series WHERE name LIKE ? OR description LIKE ? ORDER BY name",
            search_pattern,
            search_pattern
        )
        .fetch_all(pool)
        .await?;
        Ok(found)
    }
}
