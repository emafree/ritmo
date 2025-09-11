use ritmo_db_core::connection::create_connection_pool;
use ritmo_db_core::maintenance::{
    integrity::{check_integrity, get_database_version},
    backup::backup_database
};
use std::path::PathBuf;
use tempfile::tempdir;

async fn setup_test_db() -> (tempfile::TempDir, PathBuf, sqlx::Pool<sqlx::Sqlite>) {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_maint.sqlite");
    
    let pool = create_connection_pool(&db_path, true).await.unwrap();
    
    // Creare alcune tabelle di test per verificare l'integrità
    sqlx::query(
        "CREATE TABLE books (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL
        )"
    )
    .execute(&pool)
    .await
    .unwrap();
    
    sqlx::query(
        "CREATE TABLE authors (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        )"
    )
    .execute(&pool)
    .await
    .unwrap();
    
    sqlx::query(
        "CREATE TABLE tags (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        )"
    )
    .execute(&pool)
    .await
    .unwrap();
    
    (temp_dir, db_path, pool)
}

#[tokio::test]
async fn test_check_integrity() {
    let (_temp_dir, _db_path, pool) = setup_test_db().await;
    
    // Verificare l'integrità
    let integrity_result = check_integrity(&pool).await.unwrap();
    assert!(integrity_result);
}

#[tokio::test]
async fn test_get_database_version() {
    let (_temp_dir, _db_path, pool) = setup_test_db().await;
    
    // Impostare una versione di test
    sqlx::query("PRAGMA user_version = 42")
        .execute(&pool)
        .await
        .unwrap();
    
    // Verificare che possiamo leggere la versione
    let version = get_database_version(&pool).await.unwrap();
    assert_eq!(version, 42);
}

#[tokio::test]
async fn test_backup_database() {
    let (temp_dir, db_path, pool) = setup_test_db().await;
    
    // Inserire dati di test
    sqlx::query("INSERT INTO books (title) VALUES ('Test Book')")
        .execute(&pool)
        .await
        .unwrap();
    
    // Creare un percorso per il backup
    let backup_path = temp_dir.path().join("backup.sqlite");
    
    // Eseguire il backup
    backup_database(&pool, &db_path, &backup_path).await.unwrap();
    
    // Verificare che il backup esista
    assert!(backup_path.exists());
    
    // Verificare che il backup contenga i nostri dati
    let backup_pool = create_connection_pool(&backup_path, false).await.unwrap();
    let result: (String,) = sqlx::query_as("SELECT title FROM books")
        .fetch_one(&backup_pool)
        .await
        .unwrap();
    
    assert_eq!(result.0, "Test Book");
}