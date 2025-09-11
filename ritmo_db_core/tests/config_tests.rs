use ritmo_db_core::connection::create_connection_pool;
use ritmo_db_core::config::optimize_database;
use tempfile::tempdir;

#[tokio::test]
async fn test_optimize_database() {
    // Creare un database di test
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_optimize.sqlite");
    
    let pool = create_connection_pool(&db_path, true).await.unwrap();
    
    // Creare alcune tabelle e dati di test
    sqlx::query("CREATE TABLE test (id INTEGER PRIMARY KEY, data TEXT)")
        .execute(&pool)
        .await
        .unwrap();
    
    for i in 0..100 {
        sqlx::query("INSERT INTO test (data) VALUES (?)")
            .bind(format!("test data {}", i))
            .execute(&pool)
            .await
            .unwrap();
    }
    
    // Eliminare alcuni dati per creare spazio libero nel file
    sqlx::query("DELETE FROM test WHERE id % 2 = 0")
        .execute(&pool)
        .await
        .unwrap();
    
    // Eseguire l'ottimizzazione
    let result = optimize_database(&pool).await;
    assert!(result.is_ok());
    
    // Verifichiamo che il database funzioni ancora correttamente
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM test")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(count.0, 50); // Dovrebbero rimanere metà delle righe
}