use ritmo_errors::RitmoErr;
use ritmo_errors::RitmoResult;
use serde_json::Value;

use chrono::Utc;
use sqlx::{FromRow, Row, SqlitePool};
use std::collections::HashMap;

/// Struttura principale che rappresenta un database RitmoDB
pub struct Database {
    pool: SqlitePool,
    db_metadata: DatabaseMetadata,
}

/// Metadati del database
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseMetadata {
    pub version: String,
    pub updated_at: i64,
    pub created_at: i64,
}

/// Report dello stato di salute del database
#[derive(Debug)]
pub struct DatabaseHealthReport {
    pub is_healthy: bool,
    pub issues: Vec<String>,
    pub metadata: DatabaseMetadata,
    pub table_counts: HashMap<String, u64>,
}

impl Database {
    /// Crea una nuova istanza Database da un pool esistente
    pub async fn from_pool(pool: SqlitePool) -> RitmoResult<Self> {
        // Verifica che il database sia accessibile
        Self::verify_database_connection(&pool).await?;

        println!("verified connection");
        // Configura SQLite per performance ottimali
        Self::configure_sqlite(&pool).await?;
        println!("configured pool");
        // Carica o crea i metadati del database
        let db_metadata = Self::load_or_create_metadata(&pool).await?;

        // Esegui eventuali migrazioni o verifiche di schema
        Self::verify_schema(&pool).await?;

        Ok(Database { pool, db_metadata })
    }

    /// Configura SQLite per prestazioni ottimali
    async fn configure_sqlite(pool: &SqlitePool) -> RitmoResult<()> {
        // Abilita foreign keys (importante per integrità referenziale)
        sqlx::query!("PRAGMA foreign_keys = ON")
            .execute(pool)
            .await
            .map_err(|e| {
                RitmoErr::DatabaseConnectionFailed(format!("Failed to enable foreign keys: {}", e))
            })?;

        // Configura journal mode per migliori performance
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(pool)
            .await
            .map_err(|e| {
                RitmoErr::DatabaseConnectionFailed(format!("Failed to set journal mode: {}", e))
            })?;

        // Configura sincronizzazione
        sqlx::query!("PRAGMA synchronous = NORMAL")
            .execute(pool)
            .await
            .map_err(|e| {
                RitmoErr::DatabaseConnectionFailed(format!("Failed to set synchronous mode: {}", e))
            })?;

        Ok(())
    }

    /// Verifica che la connessione al database sia funzionante
    async fn verify_database_connection(pool: &SqlitePool) -> RitmoResult<()> {
        sqlx::query!("SELECT 1 as test")
            .fetch_one(pool)
            .await
            .map_err(|e| {
                RitmoErr::DatabaseConnectionFailed(format!(
                    "Database connection test failed: {}",
                    e
                ))
            })?;

        Ok(())
    }

    /// Verifica che lo schema del database sia corretto
    async fn verify_schema(pool: &SqlitePool) -> RitmoResult<()> {
        // Lista delle tabelle richieste con il loro tipo (table/view)
        let required_schema = [
            ("metadata", "table"),
            ("books", "table"),
            ("people", "table"),
            ("publishers", "table"),
            ("series", "table"),
            ("tags", "table"),
            ("running_languages", "table"),
            ("formats", "table"),
            ("contents", "table"),
            ("aliases", "table"),
            ("roles", "table"),
            ("types", "table"),
            // Tabelle di relazione
            ("x_books_contents", "table"),
            ("x_books_people_roles", "table"),
            ("x_books_tags", "table"),
            ("x_contents_languages", "table"),
            ("x_contents_people_roles", "table"),
            ("x_contents_tags", "table"),
        ];

        for (schema_name, schema_type) in &required_schema {
            let exists = sqlx::query!(
                "SELECT name FROM sqlite_master WHERE type = ?1 AND name = ?2",
                schema_type,
                schema_name
            )
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                RitmoErr::DatabaseConnectionFailed(format!(
                    "Schema verification failed for {}: {}",
                    schema_name, e
                ))
            })?;

            if exists.is_none() {
                return Err(RitmoErr::DatabaseConnectionFailed(format!(
                    "Required {} '{}' not found in database",
                    schema_type, schema_name
                )));
            }
        }

        Ok(())
    }

    /// Carica i metadati esistenti o ne crea di nuovi
    async fn load_or_create_metadata(pool: &SqlitePool) -> RitmoResult<DatabaseMetadata> {
        // Prima verifica se la tabella metadata esiste
        let table_exists =
            sqlx::query!("SELECT name FROM sqlite_master WHERE type='table' AND name = 'metadata'")
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    RitmoErr::DatabaseConnectionFailed(format!(
                        "Failed to check metadata table: {}",
                        e
                    ))
                })?;

        dbg!(&table_exists);

        if table_exists.is_none() {
            return Err(RitmoErr::DatabaseConnectionFailed(
                "metadata table not found. Database may not be properly initialized.".to_string(),
            ));
        };

        // Prova a caricare i metadati esistenti usando query_as!
        let existing_metadata = sqlx::query!(
            "SELECT version, updated_at, created_at FROM metadata ORDER BY created_at DESC LIMIT 1"
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            RitmoErr::DatabaseConnectionFailed(format!("Failed to load database metadata: {}", e))
        })?;

        match existing_metadata {
            Some(row) => {
                let metadata = DatabaseMetadata {
                    version: row.version,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                };

                // Aggiorna l'updated_at timestamp
                let now = Utc::now().timestamp();
                sqlx::query!(
                    "UPDATE metadata SET updated_at = ?1 WHERE version = ?2 AND created_at = ?3",
                    now,
                    metadata.version,
                    metadata.created_at
                )
                .execute(pool)
                .await
                .map_err(|e| {
                    RitmoErr::DatabaseConnectionFailed(format!(
                        "Failed to update metadata timestamp: {}",
                        e
                    ))
                })?;

                return Ok(metadata);
            }
            None => {
                // Se non ci sono metadati, creane di nuovi
                let now = Utc::now().timestamp();
                let version = env!("CARGO_PKG_VERSION").to_string(); // Usa la versione del package

                let metadata = DatabaseMetadata {
                    version: version.clone(),
                    created_at: now,
                    updated_at: now,
                };

                sqlx::query!(
                    "INSERT INTO metadata (version, created_at, updated_at) VALUES (?1, ?2, ?3)",
                    metadata.version,
                    metadata.created_at,
                    metadata.updated_at
                )
                .execute(pool)
                .await
                .map_err(|e| {
                    RitmoErr::DatabaseConnectionFailed(format!(
                        "Failed to create database metadata: {}",
                        e
                    ))
                })?;

                Ok(metadata)
            }
        }
    }

    /// Ottieni una referenza al pool di connessioni
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Ottieni i metadati del database
    pub fn metadata(&self) -> &DatabaseMetadata {
        &self.db_metadata
    }

    /// Aggiorna i metadati del database
    pub async fn update_metadata(&mut self, new_version: Option<String>) -> RitmoResult<()> {
        let now = Utc::now().timestamp();
        let version = new_version.unwrap_or_else(|| self.db_metadata.version.to_string());

        sqlx::query!(
            "UPDATE metadata SET version = ?1, updated_at = ?2 WHERE created_at = ?3",
            version,
            now,
            self.db_metadata.created_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            RitmoErr::DatabaseConnectionFailed(format!("Failed to update metadata: {}", e))
        })?;

        // Aggiorna i metadati in memoria
        self.db_metadata.version = version;
        self.db_metadata.updated_at = now;

        Ok(())
    }

    /// Esegui un health check del database
    pub async fn health_check(&self) -> RitmoResult<DatabaseHealthReport> {
        let mut report = DatabaseHealthReport {
            is_healthy: true,
            issues: Vec::new(),
            metadata: self.db_metadata.clone(),
            table_counts: HashMap::new(),
        };

        // Verifica connessione
        if let Err(e) = Self::verify_database_connection(&self.pool).await {
            report.is_healthy = false;
            report.issues.push(format!("Connection failed: {}", e));
            return Ok(report);
        }

        // Conta record nelle tabelle principali usando query! macro
        let main_tables = [
            "books",
            "people",
            "publishers",
            "series",
            "tags",
            "languages",
        ];

        for table in &main_tables {
            // Usa dynamic query per nomi tabella variabili
            let count_result = sqlx::query(&format!("SELECT COUNT(*) as count FROM {}", table))
                .fetch_one(&self.pool)
                .await;

            match count_result {
                Ok(row) => {
                    let count: i64 = row.get("count");
                    report.table_counts.insert(table.to_string(), count as u64);
                }
                Err(e) => {
                    report.is_healthy = false;
                    report
                        .issues
                        .push(format!("Failed to count {}: {}", table, e));
                }
            }
        }

        // Verifica integrità database
        let integrity_result = sqlx::query!("PRAGMA integrity_check")
            .fetch_one(&self.pool)
            .await;

        match integrity_result {
            Ok(row) => {
                // Accedi al valore all'interno dell'Option
                // Usiamo un match per gestire sia Some("ok") che altri valori
                let result = &row.integrity_check;
                match result {
                    Some(value) if value == "ok" => {
                        // Il database è integro, non fare nulla
                    }
                    Some(value) => {
                        // L'integrità fallisce, il valore contiene il messaggio di errore
                        report.is_healthy = false;
                        report
                            .issues
                            .push(format!("Integrity check failed: {}", value));
                    }
                    None => {
                        // Il valore è NULL (None), che è un altro tipo di fallimento dell'integrità
                        report.is_healthy = false;
                        report
                            .issues
                            .push("Integrity check returned a NULL value.".to_string());
                    }
                }
            }
            Err(e) => {
                // Gestione dell'errore di connessione o esecuzione della query
                report.is_healthy = false;
                report.issues.push(format!("Integrity check error: {}", e));
            }
        }

        // Verifica foreign keys
        let fk_result = sqlx::query!("PRAGMA foreign_key_check")
            .fetch_all(&self.pool)
            .await;

        match fk_result {
            Ok(violations) => {
                if !violations.is_empty() {
                    report.is_healthy = false;
                    report
                        .issues
                        .push(format!("Found {} foreign key violations", violations.len()));
                }
            }
            Err(e) => {
                report.is_healthy = false;
                report
                    .issues
                    .push(format!("Foreign key check error: {}", e));
            }
        }

        Ok(report)
    }

    /// Ottieni statistiche del database
    pub async fn get_database_stats(&self) -> RitmoResult<HashMap<String, serde_json::Value>> {
        let mut stats = HashMap::new();

        // Dimensione database
        if let Ok(row) = sqlx::query!("PRAGMA page_size").fetch_one(&self.pool).await {
            stats.insert("page_size".to_string(), Value::from(row.page_size));
        }

        if let Ok(row) = sqlx::query!("PRAGMA page_count")
            .fetch_one(&self.pool)
            .await
        {
            stats.insert(
                "page_count".to_string(),
                serde_json::Value::from(row.page_count),
            );
        }

        // Versione SQLite
        if let Ok(row) = sqlx::query!("SELECT sqlite_version() as version")
            .fetch_one(&self.pool)
            .await
        {
            stats.insert(
                "sqlite_version".to_string(),
                serde_json::Value::from(row.version),
            );
        }

        // Journal mode
        if let Ok(row) = sqlx::query("PRAGMA journal_mode")
            .fetch_one(&self.pool)
            .await
        {
            let journal_mode: String = row.get(0);
            stats.insert(
                "journal_mode".to_string(),
                serde_json::Value::from(journal_mode),
            );
        }
        Ok(stats)
    }

    /// Esegui vacuum del database per ottimizzarlo
    pub async fn vacuum(&self) -> RitmoResult<()> {
        sqlx::query!("VACUUM")
            .execute(&self.pool)
            .await
            .map_err(|e| RitmoErr::DatabaseConnectionFailed(format!("Vacuum failed: {}", e)))?;

        Ok(())
    }

    /// Chiudi la connessione al database
    pub async fn close(self) {
        self.pool.close().await;
    }
}

impl Default for DatabaseMetadata {
    fn default() -> Self {
        let now = Utc::now().timestamp();
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use tempfile::NamedTempFile;

    async fn create_test_database() -> SqlitePool {
        let temp_db = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite://{}", temp_db.path().to_str().unwrap());

        let pool = SqlitePoolOptions::new().connect(&db_url).await.unwrap();

        // Crea schema minimo per i test
        sqlx::query(
            r#"
            CREATE TABLE database_metadata (
                version TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // Crea altre tabelle richieste
        let tables = [
            "books",
            "people",
            "publishers",
            "series",
            "tags",
            "languages",
            "formats",
            "contents",
            "aliases",
            "roles",
            "types",
            "x_books_contents",
            "x_books_people_roles",
            "x_books_tags",
            "x_contents_languages",
            "x_contents_people_roles",
            "x_contents_tags",
        ];

        for table in &tables {
            sqlx::query(&format!("CREATE TABLE {} (id INTEGER PRIMARY KEY)", table))
                .execute(&pool)
                .await
                .unwrap();
        }

        pool
    }

    #[tokio::test]
    async fn test_database_from_pool() {
        let pool = create_test_database().await;
        let db = Database::from_pool(pool).await.unwrap();

        assert_eq!(db.metadata().version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let pool = create_test_database().await;
        let db = Database::from_pool(pool).await.unwrap();

        let health = db.health_check().await.unwrap();
        assert!(health.is_healthy);
        assert!(health.table_counts.contains_key("books"));
    }

    #[tokio::test]
    async fn test_database_stats() {
        let pool = create_test_database().await;
        let db = Database::from_pool(pool).await.unwrap();

        let stats = db.get_database_stats().await.unwrap();
        assert!(stats.contains_key("sqlite_version"));
        assert!(stats.contains_key("journal_mode"));
    }
}
