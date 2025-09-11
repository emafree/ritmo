use ritmo_db_core::connection::{create_connection_pool, create_sqlite_options};
use tempfile::tempdir;

#[tokio::test]
async fn test_create_sqlite_options() {
    // Creare un percorso temporaneo per il test
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db.sqlite");
    
    // Testare la creazione delle opzioni
    let options = create_sqlite_options(&db_path, true);
    
    // Verificare che le opzioni siano state create correttamente
    // Nota: Non possiamo verificare direttamente i valori delle opzioni perché 
    // SqliteConnectOptions non espone i campi interni, ma possiamo verificare che non fallisca
    assert!(options.is_ok());
}

#[tokio::test]
async fn test_create_connection_pool() {
    // Creare un percorso temporaneo per il test
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_pool_db.sqlite");
    
    // Testare la creazione del pool
    let pool = create_connection_pool(&db_path, true).await.unwrap();
    
    // Verificare che il pool funzioni eseguendo una query semplice
    let result: Result<(i64,), sqlx::Error> = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, 1);
    
    // Verificare che il file del database sia stato creato
    assert!(db_path.exists());
}

#[tokio::test]
async fn test_connection_pool_with_existing_db() {
    // Creare un percorso temporaneo per il test
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("existing_db.sqlite");
    
    // Prima creiamo il database
    {
        let pool = create_connection_pool(&db_path, true).await.unwrap();
        let _ = sqlx::query("CREATE TABLE test_table (id INTEGER PRIMARY KEY)")
            .execute(&pool)
            .await;
    }
    
    // Ora riapriamo il database esistente
    let pool = create_connection_pool(&db_path, false).await.unwrap();
    
    // Verificare che possiamo accedere alla tabella creata
    let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='test_table'")
        .fetch_optional(&pool)
        .await;
    
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}