use ritmo_errors::RitmoResult;
use sqlx::FromRow;

/// Language role constants for i18n
/// These are canonical keys that map to translated strings
pub mod language_role {
    pub const ORIGINAL: &str = "language_role.original";
    pub const SOURCE: &str = "language_role.source";
    pub const ACTUAL: &str = "language_role.actual";
}

/// Running language with i18n support for language roles
/// Uses canonical keys (e.g., "language_role.original") instead of translated strings
#[derive(Debug, Clone, FromRow)]
pub struct RunningLanguages {
    pub id: Option<i64>,
    pub name: String,
    pub role: String,
    pub iso_code_2char: Option<String>,
    pub iso_code_3char: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

impl RunningLanguages {
    pub fn new() -> Self {
        Self {
            id: None,
            name: String::new(),
            role: String::new(),
            iso_code_2char: None,
            iso_code_3char: None,
            created_at: None,
            updated_at: None,
        }
    }

    /// Get the display name for this language role in the current UI language
    /// Uses the i18n system to translate role keys (e.g., "language_role.original" -> "Original Language")
    pub fn display_role(&self) -> String {
        use rust_i18n::t;

        // Map language_role key to translation key
        // "language_role.original" -> "db.language_role.original"
        let translation_key = format!("db.{}", self.role);
        t!(&translation_key).to_string()
    }

    pub async fn save(&self, pool: &sqlx::SqlitePool) -> RitmoResult<i64> {
        let now = chrono::Utc::now().timestamp();
        let result =
            sqlx::query!(
                "INSERT INTO running_languages (official_name, language_role, iso_code_2char, iso_code_3char, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                self.name,
                self.role,
                self.iso_code_2char,
                self.iso_code_3char,
                now,
                now
                )
                .execute(pool)
                .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> RitmoResult<Option<RunningLanguages>> {
        let result = sqlx::query_as!(
            RunningLanguages,
            r#"SELECT id, official_name as "name", language_role as "role",
               iso_code_2char, iso_code_3char, created_at, updated_at
               FROM running_languages WHERE id = ?"#,
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    pub async fn get_by_iso_and_role(
        pool: &sqlx::SqlitePool,
        iso_code_2char: &str,
        iso_code_3char: &str,
        role: &str,
    ) -> RitmoResult<Option<RunningLanguages>> {
        let result = sqlx::query_as!(
            RunningLanguages,
            r#"SELECT id, official_name as "name", language_role as "role",
               iso_code_2char, iso_code_3char, created_at, updated_at
               FROM running_languages
               WHERE iso_code_2char = ? AND iso_code_3char = ? AND language_role = ?
               LIMIT 1"#,
            iso_code_2char,
            iso_code_3char,
            role
        )
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    /// Get or create a language by ISO codes and role, returning the language ID
    ///
    /// # Arguments
    /// * `official_name` - The official name of the language (e.g., "Italian")
    /// * `iso_code_2char` - ISO 639-1 two-letter code (e.g., "it")
    /// * `iso_code_3char` - ISO 639-2 three-letter code (e.g., "ita")
    /// * `role` - The language role: "Original", "Source", or "Actual"
    pub async fn get_or_create_by_iso_and_role(
        pool: &sqlx::SqlitePool,
        official_name: &str,
        iso_code_2char: &str,
        iso_code_3char: &str,
        role: &str,
    ) -> RitmoResult<i64> {
        if let Some(lang) = Self::get_by_iso_and_role(pool, iso_code_2char, iso_code_3char, role).await? {
            return Ok(lang.id.unwrap_or(0));
        }
        let lang = RunningLanguages {
            id: None,
            name: official_name.to_string(),
            role: role.to_string(),
            iso_code_2char: Some(iso_code_2char.to_string()),
            iso_code_3char: Some(iso_code_3char.to_string()),
            created_at: None,
            updated_at: None,
        };
        lang.save(pool).await
    }

    pub async fn update(_pool: &sqlx::SqlitePool, _id: i64, _name: &str) -> RitmoResult<()> {
        Ok(())
    }

    pub async fn delete(_pool: &sqlx::SqlitePool, _id: i64) -> RitmoResult<()> {
        Ok(())
    }
}
