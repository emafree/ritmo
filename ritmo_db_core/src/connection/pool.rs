use super::options::create_sqlite_options;
use ritmo_errors::{RitmoErr, RitmoResult};
use sqlx::{Pool, Sqlite};
use std::{fs, path::Path};

/// Crea un pool di connessioni al database
pub async fn create_connection_pool<P: AsRef<Path>>(
    db_path: P,
    create: bool,
) -> RitmoResult<Pool<Sqlite>> {
    let path = db_path.as_ref();

    if create && !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::File::create(path)
            .map_err(|e| RitmoErr::IoError(format!("Failed to create database file: {}", e)))?;
    }

    let options = create_sqlite_options(path, create)?;

    let pool = sqlx::SqlitePool::connect_with(options)
        .await
        .map_err(|e| RitmoErr::SqlxError(e))?;

    Ok(pool)
}
