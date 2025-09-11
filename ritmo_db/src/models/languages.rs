use ritmo_errors::RitmoResult;
use ritmo_core::LanguageDto;
use sqlx::FromRow;

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

    /// questi sono placeholder
    pub fn from_dto(dto: &LanguageDto) -> Self {
        Self::set_language_data(dto)
    }

    pub fn set_language_data(_dto: &LanguageDto) -> RunningLanguages {
        RunningLanguages::new()
    }

    pub async fn get(
        _pool: &sqlx::SqlitePool,
        _id: i64,
    ) -> RitmoResult<Option<RunningLanguages>> {
        Ok(Some(RunningLanguages::new()))
    }

    pub async fn update(_pool: &sqlx::SqlitePool, _id: i64, _name: &str) -> RitmoResult<()> {
        Ok(())
    }

    pub async fn delete(_pool: &sqlx::SqlitePool, _id: i64) -> RitmoResult<()> {
        Ok(())
    }
}
