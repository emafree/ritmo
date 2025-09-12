use ritmo_db_core::create_full_database_library;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from("./mylibcli");

    let db = create_full_database_library(&root).await?;

    println!("Database creato. Versione: {}", db.metadata().version);

    Ok(())
}
