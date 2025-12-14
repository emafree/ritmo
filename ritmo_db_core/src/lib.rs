pub mod config;
pub mod connection;
pub mod database;
pub mod filters;
pub mod maintenance;
pub mod query_builder;
pub mod results;

pub use database::Database;
pub use filters::{BookFilters, BookSortField, ContentFilters, ContentSortField};
pub use query_builder::{
    build_books_query, build_contents_query, execute_books_query, execute_contents_query,
};
pub use results::{BookResult, ContentResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const DB_TEMPLATE: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/template.db"));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryConfig {
    pub root_path: PathBuf,
    pub database_path: PathBuf,
    pub storage_path: PathBuf,
    pub config_path: PathBuf,
    pub bootstrap_path: PathBuf,
    #[serde(default = "default_db_name")]
    pub db_filename: String,
    #[serde(default = "default_max_connections")]
    pub max_db_connections: u32,
    #[serde(default)]
    pub auto_vacuum: bool,
}

fn default_db_name() -> String {
    "ritmo.db".to_string()
}

fn default_max_connections() -> u32 {
    10
}

impl Default for LibraryConfig {
    fn default() -> Self {
        Self::new("/ritmo_library")
    }
}

impl LibraryConfig {
    pub fn new<P: AsRef<Path>>(root_path: P) -> Self {
        let root = root_path.as_ref().to_path_buf();
        Self {
            root_path: root.clone(),
            database_path: root.join("database"),
            storage_path: root.join("storage"),
            config_path: root.join("config"),
            bootstrap_path: root.join("bootstrap"),
            db_filename: default_db_name(),
            max_db_connections: default_max_connections(),
            auto_vacuum: false,
        }
    }

    /// Controlla se la directory principale della libreria esiste su disco
    pub fn exists(&self) -> bool {
        self.root_path.exists() && self.root_path.is_dir()
    }

    /// Controlla se tutte le directory essenziali della libreria esistono
    pub fn all_dirs_exist(&self) -> bool {
        self.root_path.exists()
            && self.database_path.exists()
            && self.storage_path.exists()
            && self.config_path.exists()
            && self.bootstrap_path.exists()
    }

    /// Carica configurazione da file, crea default se non esiste
    pub fn load_or_create<P: AsRef<Path>>(
        config_file: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let path = config_file.as_ref();

        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: Self = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save(path)?;
            Ok(config)
        }
    }

    /// Salva configurazione su file
    pub fn save<P: AsRef<Path>>(&self, config_file: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = config_file.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        dbg!(&content);
        fs::write(config_file, content)?;
        Ok(())
    }

    /// Canonicalizza un path se possibile, altrimenti restituisce l'originale
    fn canonicalize_path(path: &Path) -> PathBuf {
        fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }

    pub fn canonical_root_path(&self) -> PathBuf {
        Self::canonicalize_path(&self.root_path)
    }

    pub fn canonical_database_path(&self) -> PathBuf {
        Self::canonicalize_path(&self.database_path)
    }

    pub fn canonical_storage_path(&self) -> PathBuf {
        Self::canonicalize_path(&self.storage_path)
    }

    pub fn canonical_config_path(&self) -> PathBuf {
        Self::canonicalize_path(&self.config_path)
    }

    pub fn canonical_bootstrap_path(&self) -> PathBuf {
        Self::canonicalize_path(&self.bootstrap_path)
    }

    pub fn canonical_portable_bootstrap_path(&self) -> PathBuf {
        self.canonical_bootstrap_path().join("portable")
    }

    pub fn db_file_path(&self) -> PathBuf {
        self.canonical_database_path().join(&self.db_filename)
    }

    /// Percorso del file di configurazione principale
    pub fn main_config_file(&self) -> PathBuf {
        self.canonical_config_path().join("ritmo.toml")
    }

    /// Percorso del database template per bootstrap
    pub fn template_db_path(&self) -> PathBuf {
        self.canonical_bootstrap_path().join("template.db")
    }

    /// Inizializza tutte le directory necessarie
    pub fn initialize(&self) -> Result<(), std::io::Error> {
        let dirs = [
            &self.root_path,
            &self.database_path,
            &self.storage_path,
            &self.config_path,
            &self.bootstrap_path,
        ];

        for dir in dirs {
            fs::create_dir_all(dir)?;
        }

        // Crea sottodirectory specifiche
        fs::create_dir_all(self.bootstrap_path.join("portable_app"))?;
        fs::create_dir_all(self.storage_path.join("books"))?;
        fs::create_dir_all(self.storage_path.join("covers"))?;
        fs::create_dir_all(self.storage_path.join("temp"))?;

        Ok(())
    }

    /// Valida che tutte le directory esistano
    pub fn validate(&self) -> Result<bool, std::io::Error> {
        let dirs = [
            &self.root_path,
            &self.database_path,
            &self.storage_path,
            &self.config_path,
            &self.bootstrap_path,
        ];

        for dir in &dirs {
            if !dir.exists() {
                return Ok(false);
            }
            if !dir.is_dir() {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Controlla se la configurazione è valida e completa
    pub fn health_check(&self) -> Vec<String> {
        let mut issues = Vec::new();

        // Verifica directory
        if let Ok(false) = self.validate() {
            issues.push("Una o più directory richieste non esistono".to_string());
        }

        // Verifica database
        let db_path = self.db_file_path();
        if !db_path.exists() {
            issues.push(format!("Database non trovato: {}", db_path.display()));
        }

        // Verifica permissions (solo su sistemi Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&self.root_path) {
                let permissions = metadata.permissions();
                if permissions.mode() & 0o200 == 0 {
                    issues.push("Directory root non scrivibile".to_string());
                }
            }
        }

        issues
    }

    pub async fn initialize_database(&self) -> Result<(), ritmo_errors::RitmoErr> {
        use tokio::fs;

        let db_path = self.db_file_path();

        // Assicurati che la directory del database esista
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| ritmo_errors::RitmoErr::DatabaseConnectionFailed(e.to_string()))?;
        }

        // Se il database non esiste, scrivilo dai bytes dell'include
        if !db_path.exists() {
            fs::write(&db_path, DB_TEMPLATE).await.map_err(|e| {
                ritmo_errors::RitmoErr::DatabaseConnectionFailed(format!(
                    "Impossibile scrivere template database: {}",
                    e
                ))
            })?;
        }

        Ok(())
    }

    /// Crea un nuovo database da zero (per sviluppo/testing)
    pub async fn create_fresh_database(&self) -> Result<(), ritmo_errors::RitmoErr> {
        let db_path = self.db_file_path();

        // Rimuovi database esistente se presente
        if db_path.exists() {
            fs::remove_file(&db_path)
                .map_err(|e| ritmo_errors::RitmoErr::DatabaseConnectionFailed(e.to_string()))?;
        }

        self.initialize_database().await
    }

    fn normalize_db_path(path: &Path) -> String {
        let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let path_str = canonical.to_str().unwrap_or("");

        // Rimuovi prefissi Windows UNC se presenti
        let cleaned = path_str
            .strip_prefix(r"\\?\")
            .or_else(|| path_str.strip_prefix("//?/"))
            .unwrap_or(path_str);

        // Normalizza separatori per SQLite
        cleaned.replace('\\', "/")
    }

    pub async fn create_pool(&self) -> Result<sqlx::SqlitePool, ritmo_errors::RitmoErr> {
        let db_path = self.db_file_path();
        let normalized_path = Self::normalize_db_path(&db_path);

        // Costruisci URL con opzioni
        let mut db_url = format!("sqlite://{}?mode=rwc", normalized_path);

        // Aggiungi opzioni aggiuntive
        if self.auto_vacuum {
            db_url.push_str("&auto_vacuum=INCREMENTAL");
        }

        println!("Connecting to database: {}", db_url);

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(self.max_db_connections)
            .connect(&db_url)
            .await
            .map_err(|e| ritmo_errors::RitmoErr::DatabaseConnectionFailed(e.to_string()))?;

        Ok(pool)
    }

    /// Crea una connessione Database completa
    pub async fn create_database(&self) -> Result<Database, ritmo_errors::RitmoErr> {
        let pool = self.create_pool().await?;
        Database::from_pool(pool).await
    }

    /// Backup del database
    pub async fn backup_database<P: AsRef<Path>>(
        &self,
        backup_path: P,
    ) -> Result<(), ritmo_errors::RitmoErr> {
        let db_path = self.db_file_path();
        fs::copy(&db_path, backup_path).map_err(|e| {
            ritmo_errors::RitmoErr::DatabaseConnectionFailed(format!("Backup fallito: {}", e))
        })?;
        Ok(())
    }
}
