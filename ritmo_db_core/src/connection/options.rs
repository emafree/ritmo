use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use std::{path::Path, str::FromStr};
use ritmo_errors::{RitmoErr, RitmoResult};

/// Crea le opzioni di connessione per SQLite con configurazioni ottimizzate
pub fn create_sqlite_options<P: AsRef<Path>>(db_path: P, create: bool) -> RitmoResult<SqliteConnectOptions> {
    let database_url = format!("sqlite:///{}", db_path.as_ref().to_string_lossy());
    
    let mut options = SqliteConnectOptions::from_str(&database_url)
        .map_err(|e| RitmoErr::SqlxError(e))?
        .create_if_missing(create)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    options = options
        .pragma("cache_size", "-64000")
        .pragma("temp_store", "MEMORY")
        .pragma("foreign_keys", "ON");

    Ok(options)
}