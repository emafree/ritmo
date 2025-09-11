use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Person {
    pub id: Option<i64>,
    pub name: String,
    pub display_name: Option<String>,
    pub given_name: Option<String>,
    pub surname: Option<String>,
    pub middle_names: Option<String>,
    pub title: Option<String>,
    pub suffix: Option<String>,
    pub nationality: Option<String>,
    pub birth_date: Option<i64>,
    pub death_date: Option<i64>,
    pub biography: Option<String>,
    pub normalized_key: Option<String>,
    pub confidence: f64,
    pub source: String,
    pub verified: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Person {
    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "INSERT INTO people (
                name, display_name, given_name, surname, middle_names, title, suffix, nationality,
                birth_date, death_date, biography, normalized_key, confidence, source, verified,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            self.name,
            self.display_name,
            self.given_name,
            self.surname,
            self.middle_names,
            self.title,
            self.suffix,
            self.nationality,
            self.birth_date,
            self.death_date,
            self.biography,
            self.normalized_key,
            self.confidence,
            self.source,
            self.verified,
            now,
            now
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }
    pub fn from_dto() {}

    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Person>, sqlx::Error> {
        let person = sqlx::query_as!(
            Person,
            "SELECT * FROM people WHERE id = ?",
            id
            )
            .fetch_optional(pool)
            .await?;
        Ok(person)
    }

    pub async fn update(&self, pool: &sqlx::SqlitePool) -> Result<u64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "UPDATE people SET
                name = ?, display_name = ?, given_name = ?, surname = ?, middle_names = ?, title = ?, suffix = ?,
                nationality = ?, birth_date = ?, death_date = ?, biography = ?, normalized_key = ?, confidence = ?,
                source = ?, verified = ?, updated_at = ?
            WHERE id = ?",
        self.name,
        self.display_name,
        self.given_name,
        self.surname,
        self.middle_names,
        self.title,
        self.suffix,
        self.nationality,
        self.birth_date,
        self.death_date,
        self.biography,
        self.normalized_key,
        self.confidence,
        self.source,
        self.verified,
        now,
        self.id
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM people WHERE id = ?",
            id
            )
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_all(pool: &sqlx::SqlitePool) -> Result<Vec<Person>, sqlx::Error> {
        let all = sqlx::query_as!(
            Person,
            "SELECT * FROM people ORDER BY name"
            )
            .fetch_all(pool)
            .await?;
        Ok(all)
    }

    pub async fn search(
        pool: &sqlx::SqlitePool,
        pattern: &str,
    ) -> Result<Vec<Person>, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let found = sqlx::query_as!(
            Person,
            "SELECT * FROM people WHERE name LIKE ? OR display_name LIKE ? OR given_name LIKE ? OR surname LIKE ? OR biography LIKE ? ORDER BY name",
        search_pattern,
        search_pattern,
        search_pattern,
        search_pattern,
        search_pattern
        )
        .fetch_all(pool)
        .await?;
        Ok(found)
    }
}
