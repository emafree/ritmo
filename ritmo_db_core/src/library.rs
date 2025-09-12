use crate::connection::pool::create_connection_pool;
use crate::database::Database;
use crate::LibraryConfig;
use ritmo_errors::RitmoErr;
use std::fs;
use std::path::Path;

/// Tutta questa parte è da rivedere e ripensare.
/// Crea tutta la struttura di una nuova libreria/database atomica.
/// Usa LibraryConfig per ottenere tutti i path canonici!
pub async fn create_full_database_library<P: AsRef<Path>>(root: P) -> Result<Database, RitmoErr> {
    let config = LibraryConfig::new(&root);

    // Crea tutte le directory canoniche
    fs::create_dir_all(config.canonical_database_path())?;
    fs::create_dir_all(config.canonical_config_path())?;
    fs::create_dir_all(config.canonical_bootstrap_path())?;
    fs::create_dir_all(config.canonical_portable_bootstrap_path())?;

    // Crea il file fisico del database nel path canonico
    let db_path = config.db_file_path();

    dbg!(&db_path);

    // Scrivi il contenuto della costante (i byte del template) sul nuovo file
    fs::write(db_path, DB_TEMPLATE).unwrap();

    let pool = create_connection_pool(&db_path, true).await?;

    println!("created pool");

    let db = Database::from_pool(pool).await?;

    Ok(db)
}
