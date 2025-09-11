pub mod config;
pub mod connection;
pub mod database;
pub mod maintenance;
pub mod library;

pub use database::Database;
pub use library::create_full_database_library;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
        Self::new("./ritmo_library")
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

        // Verifica template
        let template_path = self.template_db_path();
        if !template_path.exists() {
            issues.push(format!(
                "Template database non trovato: {}",
                template_path.display()
            ));
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

    /// Inizializza il database copiando dal template
    pub async fn initialize_database(&self) -> Result<(), ritmo_errors::RitmoErr> {
        let db_path = self.db_file_path();
        let template_path = self.template_db_path();

        // Assicurati che la directory del database esista
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ritmo_errors::RitmoErr::DatabaseConnectionFailed(e.to_string()))?;
        }

        // Se il database non esiste, copialo dal template
        if !db_path.exists() {
            if template_path.exists() {
                fs::copy(&template_path, &db_path).map_err(|e| {
                    ritmo_errors::RitmoErr::DatabaseConnectionFailed(format!(
                        "Impossibile copiare template database: {}",
                        e
                    ))
                })?;
            } else {
                return Err(ritmo_errors::RitmoErr::DatabaseConnectionFailed(format!(
                    "Template database non trovato: {}",
                    template_path.display()
                )));
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = LibraryConfig::new(temp_dir.path());

        assert_eq!(config.root_path, temp_dir.path());
        assert_eq!(config.db_filename, "ritmo.db");
        assert_eq!(config.max_db_connections, 10);
    }

    #[test]
    fn test_initialize() {
        let temp_dir = TempDir::new().unwrap();
        let config = LibraryConfig::new(temp_dir.path());

        config.initialize().unwrap();

        assert!(config.validate().unwrap());
    }

    #[test]
    fn test_save_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.toml");

        let config = LibraryConfig::new(temp_dir.path());
        config.save(&config_file).unwrap();

        let loaded_config = LibraryConfig::load_or_create(&config_file).unwrap();
        assert_eq!(config.root_path, loaded_config.root_path);
    }
}
