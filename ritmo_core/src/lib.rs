pub mod dto;
pub mod storage;
pub use dto::*;
pub use storage::create_storage_dirs;

//pub mod service;

//use std::fs;
//use std::path::{Path, PathBuf};
//
//pub struct LibraryConfig {
//    pub root_path: PathBuf,
//    pub database_path: PathBuf,
//    pub storage_path: PathBuf,
//    pub config_path: PathBuf,
//    pub bootstrap_path: PathBuf,
//}
//
//impl LibraryConfig {
//    pub fn new<P: AsRef<Path>>(root_path: P) -> Self {
//        let root = root_path.as_ref().to_path_buf();
//        Self {
//            root_path: root.clone(),
//            database_path: root.join("database"),
//            storage_path: root.join("storage"),
//            config_path: root.join("config"),
//            bootstrap_path: root.join("bootstrap"),
//        }
//    }
//
//    /// Canonicalizza un path se possibile, altrimenti restituisce l'originale
//    fn canonicalize_path(path: &Path) -> PathBuf {
//        fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
//    }
//
//    pub fn canonical_root_path(&self) -> PathBuf {
//        Self::canonicalize_path(&self.root_path)
//    }
//    pub fn canonical_database_path(&self) -> PathBuf {
//        Self::canonicalize_path(&self.database_path)
//    }
//    pub fn canonical_storage_path(&self) -> PathBuf {
//        Self::canonicalize_path(&self.storage_path)
//    }
//    pub fn canonical_config_path(&self) -> PathBuf {
//        Self::canonicalize_path(&self.config_path)
//    }
//    pub fn canonical_bootstrap_path(&self) -> PathBuf {
//        Self::canonicalize_path(&self.bootstrap_path)
//    }
//
//    pub fn db_file_path(&self) -> PathBuf {
//        self.canonical_database_path().join("ritmo.db")
//    }
//
//    pub fn initialize(&self) -> Result<(), std::io::Error> {
//        fs::create_dir_all(&self.root_path)?;
//        fs::create_dir_all(&self.database_path)?;
//        fs::create_dir_all(&self.storage_path)?;
//        fs::create_dir_all(&self.config_path)?;
//        fs::create_dir_all(&self.bootstrap_path)?;
//        fs::create_dir_all(self.bootstrap_path.join("portable_app"))?;
//        Ok(())
//    }
//
//    pub fn validate(&self) -> Result<bool, std::io::Error> {
//        let dirs = [
//            &self.root_path,
//            &self.database_path,
//            &self.storage_path,
//            &self.config_path,
//            &self.bootstrap_path,
//        ];
//        Ok(dirs.iter().all(|d| d.exists()))
//    }
//
//    pub async fn initialize_database(&self) -> Result<(), ritmo_errors::RitmoErr> {
//        let db_path = self.db_file_path();
//        // Assicurati che la directory esista
//        if let Some(parent) = db_path.parent() {
//            fs::create_dir_all(parent)
//                .map_err(|e| ritmo_errors::RitmoErr::DatabaseConnectionFailed(e.to_string()))?;
//        }
//        // Se vuoi che il file esista, crealo vuoto (opzionale)
//        // let _ = fs::OpenOptions::new().create(true).write(true).open(&db_path);
//
//        Ok(())
//    }
//
//    pub async fn create_pool(&self) -> Result<sqlx::SqlitePool, ritmo_errors::RitmoErr> {
//        let db_path = self.db_file_path();
//
//        fn normalize_db_path(path: &std::path::Path) -> String {
//            let c = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
//            let s = c.to_str().unwrap();
//            let s = s
//                .strip_prefix(r"\\?\")
//                .or_else(|| s.strip_prefix("//?/"))
//                .unwrap_or(s);
//            s.replace("\\", "/")
//        }
//
//        let db_url = format!("sqlite://{}", normalize_db_path(&db_path));
//        println!("Canonical pool URL: {}", db_url);
//
//        sqlx::SqlitePool::connect(&db_url)
//            .await
//            .map_err(|e| ritmo_errors::RitmoErr::DatabaseConnectionFailed(e.to_string()))
//    }
//}
//
