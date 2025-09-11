use sqlx::{Pool, Sqlite, query};
use ritmo_errors::{RitmoErr, RitmoResult};

/// Esegue un VACUUM sul database per ottimizzare lo spazio
pub async fn perform_vacuum(pool: &Pool<Sqlite>) -> RitmoResult<()> {
    query("VACUUM;")
        .execute(pool)
        .await
        .map_err(|e| RitmoErr::DatabaseQueryFailed(format!(
            "Errore durante l'esecuzione di VACUUM: {}", e
        )))?;
    
    Ok(())
}