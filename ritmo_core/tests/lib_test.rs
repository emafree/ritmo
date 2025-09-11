use ritmo_db_core::LibraryConfig;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_library_config_new() {
    let root_path = Path::new("/test/path");
    let config = LibraryConfig::new(root_path);

    println!("Canonical root: {:?}", config.canonical_root_path());
    println!("Canonical database: {:?}", config.canonical_database_path());
    println!("Canonical storage: {:?}", config.canonical_storage_path());
    println!("Canonical config: {:?}", config.canonical_config_path());
    println!(
        "Canonical bootstrap: {:?}",
        config.canonical_bootstrap_path()
    );

    assert_eq!(config.root_path, Path::new("/test/path"));
    assert_eq!(config.database_path, Path::new("/test/path/database"));
    assert_eq!(config.storage_path, Path::new("/test/path/storage"));
    assert_eq!(config.config_path, Path::new("/test/path/config"));
    assert_eq!(config.bootstrap_path, Path::new("/test/path/bootstrap"));
}

#[test]
fn test_library_config_initialize() {
    let temp_dir = tempdir().expect("Impossibile creare directory temporanea");
    let root_path = temp_dir.path();

    let config = LibraryConfig::new(root_path);
    config.initialize().expect("Inizializzazione fallita");

    println!("Canonical root: {:?}", config.canonical_root_path());
    println!("Canonical database: {:?}", config.canonical_database_path());

    assert!(config.canonical_root_path().exists());
    assert!(config.canonical_database_path().exists());
    assert!(config.canonical_storage_path().exists());
    assert!(config.canonical_config_path().exists());
    assert!(config.canonical_bootstrap_path().exists());
    assert!(config
        .canonical_bootstrap_path()
        .join("portable_app")
        .exists());
}

#[test]
fn test_library_config_initialize_existing_non_empty() {
    let temp_dir = tempdir().expect("Impossibile creare directory temporanea");
    let root_path = temp_dir.path();

    // Crea file per rendere la directory non vuota
    fs::write(root_path.join("some_file.txt"), b"test content")
        .expect("Impossibile scrivere file di test");

    let config = LibraryConfig::new(root_path);
    config.initialize().expect("Inizializzazione fallita");

    println!("Canonical root: {:?}", config.canonical_root_path());
    assert!(config.canonical_root_path().exists());
}

#[test]
fn test_library_config_validate() {
    let temp_dir = tempdir().expect("Impossibile creare directory temporanea");
    let root_path = temp_dir.path();

    let config = LibraryConfig::new(root_path);

    // Prima dell'inizializzazione, la validazione dovrebbe fallire
    let validate_result = config.validate();
    assert!(validate_result.is_ok());
    assert_eq!(validate_result.unwrap(), false);

    // Dopo l'inizializzazione, la validazione dovrebbe riuscire
    config.initialize().expect("Inizializzazione fallita");
    let validate_result = config.validate();
    assert!(validate_result.is_ok());
    assert_eq!(validate_result.unwrap(), true);
}

#[tokio::test]
async fn test_initialize_database_and_pool() {
    let temp_dir = tempfile::tempdir().expect("Impossibile creare directory temporanea");
    let root_path = temp_dir.path();

    let config = LibraryConfig::new(root_path);
    config.initialize().expect("Inizializzazione fallita");

    let canonical_db_file = config.db_file_path();
    println!("DB canonical path: {:?}", canonical_db_file);

    // Assicurati che la directory esista
    assert!(
        canonical_db_file.parent().unwrap().exists(),
        "La directory del database non esiste!"
    );

    // CREA IL FILE VUOTO prima del pool!
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&canonical_db_file)
        .expect("Impossibile creare il file del database");

    // Inizializza il database
    let db_result = config.initialize_database().await;
    assert!(
        db_result.is_ok(),
        "Inizializzazione database fallita: {:?}",
        db_result
    );

    // Crea un pool di connessione
    let pool_result = config.create_pool().await;
    assert!(
        pool_result.is_ok(),
        "Creazione pool fallita: {:?}",
        pool_result
    );

    let pool = pool_result.unwrap();
    let version_row: (i64,) = sqlx::query_as!("PRAGMA user_version")
        .fetch_one(&pool)
        .await
        .expect("Query fallita");
    assert!(version_row.0 >= 0);
}
