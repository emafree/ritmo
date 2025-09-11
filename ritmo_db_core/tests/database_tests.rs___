use ritmo_db_core::Database;
use tempfile::tempdir;

#[tokio::test]
async fn test_create_database() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("new_db.sqlite");
    
    // Creare un nuovo database
    let db = Database::create(&db_path).await.unwrap();
    
    // Verificare che il database sia stato creato
    assert!(db_path.exists());
    
    // Verificare che possiamo ottenere una connessione
//    let conn = db.get_connection().await.unwrap();
    let pool = db.get_pool();

    // Verificare che possiamo eseguire query
    let result: Result<(i64,), sqlx::Error> = sqlx::query_as("SELECT 1")
        .fetch_one(pool)
        .await;
        
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_open_database() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("open_test.sqlite");
    
    // Prima creiamo un database
    {
        let db = Database::create(&db_path).await.unwrap();
        let pool = db.get_pool();
        
        // Creare una tabella di test
        sqlx::query("CREATE TABLE test_open (id INTEGER PRIMARY KEY)")
            .execute(pool)
            .await
            .unwrap();
    }
    
    // Ora lo riapriamo
    let db = Database::open(&db_path).await.unwrap();
    let pool = db.get_pool();
    
    // Verificare che possiamo accedere alla tabella
    let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='test_open'")
        .fetch_optional(pool)
        .await;
        
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[tokio::test]
async fn test_backup_to() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("original.sqlite");
    let backup_path = temp_dir.path().join("backup.sqlite");
    
    // Creare e configurare il database
    let db = Database::create(&db_path).await.unwrap();
    let pool = db.get_pool();
    
    // Aggiungere alcuni dati
    sqlx::query("CREATE TABLE test_backup (value TEXT)")
        .execute(pool)
        .await
        .unwrap();
        
    sqlx::query("INSERT INTO test_backup VALUES ('test data')")
        .execute(pool)
        .await
        .unwrap();
    
    // Eseguire il backup
    db.backup_to(&backup_path).await.unwrap();
    
    // Verificare che il backup sia stato creato
    assert!(backup_path.exists());
    
    // Verificare che il backup contenga i dati
    let backup_db = Database::open(&backup_path).await.unwrap();
    let backup_pool = backup_db.get_pool();
    
    let result: (String,) = sqlx::query_as("SELECT value FROM test_backup")
        .fetch_one(backup_pool)
        .await
        .unwrap();
        
    assert_eq!(result.0, "test data");
}

#[tokio::test]
async fn test_optimize() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("optimize_test.sqlite");
    
    // Creare il database
    let db = Database::create(&db_path).await.unwrap();
    let pool = db.get_pool();
    
    // Aggiungere e poi eliminare alcuni dati per creare spazio non ottimizzato
    sqlx::query("CREATE TABLE test_opt (id INTEGER PRIMARY KEY, data TEXT)")
        .execute(pool)
        .await
        .unwrap();
        
    for i in 0..100 {
        sqlx::query("INSERT INTO test_opt (data) VALUES (?)")
            .bind(format!("data {}", i))
            .execute(pool)
            .await
            .unwrap();
    }
    
    sqlx::query("DELETE FROM test_opt WHERE id % 2 = 0")
        .execute(pool)
        .await
        .unwrap();
    
    // Applicare l'ottimizzazione
    let result = db.optimize().await;
    assert!(result.is_ok());
    
    // Verificare che il database funzioni ancora
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM test_opt")
        .fetch_one(pool)
        .await
        .unwrap();
        
    assert_eq!(count.0, 50);
}