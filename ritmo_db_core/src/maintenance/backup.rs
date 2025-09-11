use std::{fs, path::PathBuf};
use sqlx::{Pool, Sqlite, query};
use ritmo_errors::{RitmoErr, RitmoResult};

/// Esegue un backup del database in un altro file
pub async fn backup_database(pool: &Pool<Sqlite>, source_path: &PathBuf, destination: &PathBuf) -> RitmoResult<()> {
    // Verifica che il database sorgente sia aperto
    if !source_path.exists() {
        return Err(RitmoErr::DatabaseError(
            "Il database sorgente non esiste".to_string(),
        ));
    }

    // Assicurati che la directory di destinazione esista
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    // Eseguiamo un checkpoint WAL completo
    query("PRAGMA wal_checkpoint(FULL);")
        .execute(pool)
        .await
        .map_err(|e| RitmoErr::DatabaseQueryFailed(format!(
            "Errore nel checkpoint WAL: {}", e
        )))?;

    // Proviamo a usare VACUUM INTO
    let vacuum_result = sqlx::query("VACUUM INTO ?")
        .bind(destination.to_string_lossy().to_string())
        .execute(pool)
        .await;

    // Se VACUUM INTO fallisce, utilizziamo il metodo di copia diretta
    if vacuum_result.is_err() {
        fs::copy(source_path, destination)
            .map_err(|e| RitmoErr::IoError(format!(
                "Errore nella copia del file: {}", e
            )))?;

        // Gestione dei file WAL e SHM
        let wal_path = source_path.with_extension("db-wal");
        let shm_path = source_path.with_extension("db-shm");

        if wal_path.exists() {
            let dest_wal = destination.with_extension("db-wal");
            let _ = fs::copy(&wal_path, &dest_wal);
        }

        if shm_path.exists() {
            let dest_shm = destination.with_extension("db-shm");
            let _ = fs::copy(&shm_path, &dest_shm);
        }
    }

    Ok(())
}