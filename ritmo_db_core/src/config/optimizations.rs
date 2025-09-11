use sqlx::{Pool, Sqlite, query};
use ritmo_errors::{RitmoErr, RitmoResult};

/// Applica ottimizzazioni al database
pub async fn optimize_database(pool: &Pool<Sqlite>) -> RitmoResult<()> {
    // Ottimizzazioni
    query("ANALYZE;")
        .execute(pool)
        .await
        .map_err(|e| RitmoErr::DatabaseQueryFailed(e.to_string()))?;

    query("PRAGMA optimize;")
        .execute(pool)
        .await
        .map_err(|e| RitmoErr::DatabaseQueryFailed(e.to_string()))?;

    Ok(())
}