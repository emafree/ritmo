use chrono::Utc;
use ritmo_db_core::database::*;
use ritmo_errors::RitmoResult;
use serial_test::serial;
use sqlx::Row;
use std::path::{Path, PathBuf};

// Crea un percorso temporaneo per il database di test
fn temp_db_path() -> PathBuf {
    let timestamp = Utc::now().timestamp();
    let random_suffix = rand::random::<u16>();
    PathBuf::from(format!("./target/test_db_{timestamp}_{random_suffix}.db"))
}

// Funzione helper per cancellare un file se esiste
fn clean_up_db_file(path: &Path) {
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}

#[tokio::test]
#[serial] // Esegue i test in serie per evitare conflitti sul filesystem
async fn test_database_create() {
    let db_path = temp_db_path();
    clean_up_db_file(&db_path);

    // Test creazione database
    let result = Database::create(&db_path).await;
    assert!(
        result.is_ok(),
        "Errore nella creazione del database: {:?}",
        result.err()
    );

    let db = result.unwrap();

    // Verifica che il pool sia funzionante
    let pool = db.get_pool();
    let result = sqlx::query("SELECT 1").fetch_one(pool).await;
    assert!(
        result.is_ok(),
        "Il pool di connessioni non funziona correttamente"
    );

    // Verifica che i metadati siano corretti
    let metadata = db.get_metadata();
    assert_eq!(metadata.version, env!("CARGO_PKG_VERSION"));
    assert!(metadata.created_at > 0);
    assert!(metadata.updated_at > 0);

    // Verifica che le tabelle principali esistano
    let tables = vec![
        "metadata",
        "formats",
        "publishers",
        "series",
        "books",
        // Aggiungi qui altre tabelle che dovrebbero esistere
    ];

    for table in tables {
        let result = sqlx::query(&format!(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='{}'",
            table
        ))
        .fetch_optional(pool)
        .await;
        assert!(
            result.is_ok(),
            "Errore nella query per verificare la tabella {}: {:?}",
            table,
            result.err()
        );
        assert!(result.unwrap().is_some(), "La tabella {} non esiste", table);
    }

    // Verifica che i dati di base siano stati inizializzati (formati)
    let formats = sqlx::query("SELECT COUNT(*) as count FROM formats")
        .fetch_one(pool)
        .await;
    assert!(
        formats.is_ok(),
        "Errore nella query per contare i formati: {:?}",
        formats.err()
    );
    let count: i64 = formats.unwrap().get("count");
    assert!(count > 0, "Nessun formato inizializzato nel database");

    // Pulisci dopo il test
    clean_up_db_file(&db_path);
}

#[tokio::test]
#[serial]
async fn test_database_open_non_existent() {
    let db_path = temp_db_path();
    clean_up_db_file(&db_path);

    // Prova ad aprire un database che non esiste
    let result = Database::open(&db_path).await;
    assert!(
        result.is_err(),
        "Apertura riuscita di un database inesistente"
    );

    // Verifica il tipo di errore
    if let Err(err) = result {
        match err {
            crate::errors::Error::DatabaseNotFound(_) => {} // Questo è il comportamento atteso
            _ => panic!("Tipo di errore inaspettato: {:?}", err),
        }
    }
}

#[tokio::test]
#[serial]
async fn test_database_create_open_cycle() {
    let db_path = temp_db_path();
    clean_up_db_file(&db_path);

    // Crea un nuovo database
    let create_result = Database::create(&db_path).await;
    assert!(
        create_result.is_ok(),
        "Errore nella creazione del database: {:?}",
        create_result.err()
    );

    // Chiudi il database (il drop implicito)
    drop(create_result.unwrap());

    // Riapri il database
    let open_result = Database::open(&db_path).await;
    assert!(
        open_result.is_ok(),
        "Errore nell'apertura del database: {:?}",
        open_result.err()
    );

    let db = open_result.unwrap();

    // Verifica che i metadati siano stati caricati correttamente
    let metadata = db.get_metadata();
    assert_eq!(metadata.version, env!("CARGO_PKG_VERSION"));

    // Pulisci dopo il test
    clean_up_db_file(&db_path);
}

#[tokio::test]
#[serial]
async fn test_database_metadata_persistence() {
    let db_path = temp_db_path();
    clean_up_db_file(&db_path);

    // Crea un nuovo database e memorizza il timestamp di creazione
    let db1 = Database::create(&db_path)
        .await
        .expect("Impossibile creare il database");
    let created_at = db1.get_metadata().created_at;
    drop(db1);

    // Aspetta un secondo per assicurarsi che il timestamp di aggiornamento sia diverso
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Riapri il database
    let db2 = Database::open(&db_path)
        .await
        .expect("Impossibile aprire il database");

    // Verifica che il timestamp di creazione sia lo stesso, ma quello di aggiornamento sia cambiato
    assert_eq!(
        db2.get_metadata().created_at,
        created_at,
        "Il timestamp di creazione è cambiato"
    );
    assert!(
        db2.get_metadata().updated_at > created_at,
        "Il timestamp di aggiornamento non è stato aggiornato"
    );

    // Pulisci dopo il test
    clean_up_db_file(&db_path);
}

#[tokio::test]
#[serial]
async fn test_database_migration_upgrades() {
    // Questo test è più complesso e richiederebbe una simulazione
    // di una versione precedente del database per testare le migrazioni.
    // Una soluzione semplificata è creare manualmente un database con metadati di versione precedente
    let db_path = temp_db_path();
    clean_up_db_file(&db_path);

    // Crea un database manualmente con una versione precedente
    let db_url = format!("sqlite:{}", db_path.display());
    sqlx::Sqlite::create_database(&db_url)
        .await
        .expect("Impossibile creare il database");
    let pool = sqlx::SqlitePool::connect(&db_url)
        .await
        .expect("Impossibile connettersi al database");

    // Crea la tabella metadata manualmente
    sqlx::query("CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
        .execute(&pool)
        .await
        .expect("Impossibile creare la tabella metadata");

    // Inserisci una versione precedente
    let old_version = "0.1.0"; // Versione precedente
    let now = Utc::now().timestamp();
    sqlx::query("INSERT INTO metadata (key, value) VALUES ('version', ?), ('created_at', ?), ('updated_at', ?)")
        .bind(old_version)
        .bind(now)
        .bind(now)
        .execute(&pool).await.expect("Impossibile inserire i metadati");

    drop(pool); // Chiudi la connessione

    // Ora apri il database, che dovrebbe rilevare la versione precedente e aggiornarlo
    let db = Database::open(&db_path)
        .await
        .expect("Impossibile aprire il database");

    // Verifica che la versione sia stata aggiornata
    assert_eq!(
        db.get_metadata().version,
        env!("CARGO_PKG_VERSION"),
        "La versione non è stata aggiornata"
    );
    assert!(
        db.get_metadata().updated_at > now,
        "Il timestamp di aggiornamento non è stato aggiornato dopo la migrazione"
    );

    // Pulisci dopo il test
    clean_up_db_file(&db_path);
}
