//! Library auto-repair functionality
//!
//! This module handles automatic repair of corrupted or incomplete Ritmo libraries.
//! It preserves existing books during the repair process.

use std::path::Path;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone)]
struct BookData {
    filename: String,
    content: String,
}

/// Auto-repair the library structure while preserving existing books
pub fn repair_library(library_path: &Path) -> Result<()> {
    // 1. Backup existing books first
    let books_backup = backup_books_if_exist(library_path)?;
    
    // 2. Recreate missing directories
    recreate_directory_structure(library_path)?;
    
    // 3. Recreate configs if missing
    create_default_configs(library_path)?;
    
    // 4. Recreate database structure if missing
    create_database_structure(library_path)?;
    
    // 5. Restore books
    if let Some(books) = books_backup {
        let book_count = books.len();
        restore_books(library_path, books)?;
        println!("📚 Restored {} books from backup", book_count);
    }
    
    Ok(())
}

/// Recreate the required directory structure
fn recreate_directory_structure(library_path: &Path) -> Result<()> {
    let dirs = vec![
        "ritmo_library",
        "ritmo_library/config",
        "ritmo_library/database",
        "ritmo_library/storage",
        "bootstrap/portable_app",
    ];
    
    for dir in dirs {
        std::fs::create_dir_all(library_path.join(dir))?;
    }
    
    Ok(())
}

/// Create default configuration files if they don't exist
pub fn create_default_configs(library_path: &Path) -> Result<()> {
    let config_path = library_path.join("ritmo_library/config");
    
    // Create config.toml if missing
    let config_file = config_path.join("config.toml");
    if !config_file.exists() {
        std::fs::write(
            config_file,
            r#"[library]
name = "My Ritmo Library"
version = "1.0"
"#,
        )?;
    }
    
    Ok(())
}

/// Initialize database structure (placeholder for actual DB initialization)
fn create_database_structure(library_path: &Path) -> Result<()> {
    let _db_path = library_path.join("ritmo_library/database");
    
    // Actual DB initialization is delegated to ritmo_commands
    // This just ensures the directory exists
    
    Ok(())
}

/// Backup all book files from storage if they exist
fn backup_books_if_exist(library_path: &Path) -> Result<Option<Vec<BookData>>> {
    let storage_path = library_path.join("ritmo_library/storage");
    
    if !storage_path.exists() {
        return Ok(None);
    }
    
    // Read all book files from storage
    let mut books = Vec::new();
    
    for entry in std::fs::read_dir(&storage_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().is_some_and(|ext| ext == "toml") {
            let content = std::fs::read_to_string(&path)?;
            books.push(BookData {
                filename: path.file_name().unwrap().to_string_lossy().to_string(),
                content,
            });
        }
    }
    
    Ok(if books.is_empty() { None } else { Some(books) })
}

/// Restore backed up books to storage
fn restore_books(library_path: &Path, books: Vec<BookData>) -> Result<()> {
    let storage_path = library_path.join("ritmo_library/storage");
    std::fs::create_dir_all(&storage_path)?;
    
    for book in books {
        let file_path = storage_path.join(&book.filename);
        std::fs::write(file_path, &book.content)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_repair_library_preserves_books() {
        let temp_dir = std::env::temp_dir().join("ritmo_test_repair");
        let _ = fs::remove_dir_all(&temp_dir);
        
        // Create library with books
        let storage = temp_dir.join("ritmo_library/storage");
        fs::create_dir_all(&storage).unwrap();
        fs::write(storage.join("book1.toml"), "test book 1").unwrap();
        fs::write(storage.join("book2.toml"), "test book 2").unwrap();
        
        // Repair (should preserve books)
        repair_library(&temp_dir).unwrap();
        
        // Verify books are preserved
        assert!(storage.join("book1.toml").exists());
        assert!(storage.join("book2.toml").exists());
        
        let content1 = fs::read_to_string(storage.join("book1.toml")).unwrap();
        assert_eq!(content1, "test book 1");
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_create_default_configs() {
        let temp_dir = std::env::temp_dir().join("ritmo_test_config");
        let _ = fs::remove_dir_all(&temp_dir);
        
        fs::create_dir_all(temp_dir.join("ritmo_library/config")).unwrap();
        
        create_default_configs(&temp_dir).unwrap();
        
        let config_file = temp_dir.join("ritmo_library/config/config.toml");
        assert!(config_file.exists());
        
        let content = fs::read_to_string(config_file).unwrap();
        assert!(content.contains("My Ritmo Library"));
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
