use sqlx::{Pool, Sqlite, query_as, query};
use ritmo_errors::{RitmoErr, RitmoResult};

/// Verifica l'integrità del database
pub async fn check_integrity(pool: &Pool<Sqlite>) -> RitmoResult<bool> {
    // Verifica l'integrità del database
    let integrity_check: (String,) = query_as("PRAGMA integrity_check(1)")
        .fetch_one(pool)
        .await
        .map_err(|e| RitmoErr::DatabaseQueryFailed(e.to_string()))?;

    if integrity_check.0 != "ok" {
        return Ok(false);
    }

    // Verifica la presenza delle tabelle principali
    let _tables_exist = query(
        "SELECT count(*) FROM sqlite_master WHERE type='table' AND
         name IN ('books', 'people', 'contents')"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| RitmoErr::DatabaseQueryFailed(e.to_string()))?;

    Ok(true)
}

/// Ottiene la versione del database
pub async fn get_database_version(pool: &Pool<Sqlite>) -> RitmoResult<i64> {
    let version: (i64,) = sqlx::query_as("PRAGMA user_version")
        .fetch_one(pool)
        .await
        .map_err(|e| RitmoErr::DatabaseQueryFailed(e.to_string()))?;

    Ok(version.0)
}
