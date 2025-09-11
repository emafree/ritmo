use ritmo_db_core::Database;
use tempfile::tempdir;

#[tokio::test]
async fn test_full_database_lifecycle() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("lifecycle.sqlite");
    let backup_path = temp_dir.path().join("lifecycle_backup.sqlite");
    
    // 1. Creare un nuovo database
    let db = Database::create(&db_path).await.unwrap();
    let pool = db.get_pool();
    
    // 2. Creare schema e inserire dati
    sqlx::query(
        "CREATE TABLE books (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            author TEXT NOT NULL,
            year INTEGER
        )"
    )
    .execute(pool)
    .await
    .unwrap();
    
    sqlx::query("INSERT INTO books (title, author, year) VALUES (?, ?, ?)")
        .bind("Il Nome della Rosa")
        .bind("Umberto Eco")
        .bind(1980)
        .execute(pool)
        .await
        .unwrap();
        
    sqlx::query("INSERT INTO books (title, author, year) VALUES (?, ?, ?)")
        .bind("1984")
        .bind("George Orwell")
        .bind(1949)
        .execute(pool)
        .await
        .unwrap();
    
    // 3. Verificare che i dati siano stati inseriti
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM books")
        .fetch_one(pool)
        .await
        .unwrap();
        
    assert_eq!(count.0, 2);
    
    // 4. Backup del database
    db.backup_to(&backup_path).await.unwrap();
    assert!(backup_path.exists());
    
    // 5. Ottimizzare il database
    db.optimize().await.unwrap();
    
    // 6. Riaprire il database
    drop(db);  // Chiudiamo esplicitamente il database
    
    let reopened_db = Database::open(&db_path).await.unwrap();
    let reopened_pool = reopened_db.get_pool();
    
    // 7. Verificare che i dati siano ancora presenti
    let books: Vec<(i64, String, String, i64)> = sqlx::query_as(
        "SELECT id, title, author, year FROM books ORDER BY year"
    )
    .fetch_all(reopened_pool)
    .await
    .unwrap();
    
    assert_eq!(books.len(), 2);
    assert_eq!(books[0].1, "1984");
    assert_eq!(books[0].2, "George Orwell");
    assert_eq!(books[0].3, 1949);
    
    assert_eq!(books[1].1, "Il Nome della Rosa");
    assert_eq!(books[1].2, "Umberto Eco");
    assert_eq!(books[1].3, 1980);
    
    // 8. Verificare che il backup contenga gli stessi dati
    let backup_db = Database::open(&backup_path).await.unwrap();
    let backup_pool = backup_db.get_pool();
    
    let backup_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM books")
        .fetch_one(backup_pool)
        .await
        .unwrap();
        
    assert_eq!(backup_count.0, 2);
}