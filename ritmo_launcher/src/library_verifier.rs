//! Library structure verification
//!
//! This module verifies the integrity of the Ritmo library structure,
//! checks for the existence of required binaries, and counts books.

use std::path::Path;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Verify that the library structure contains all required directories
pub fn verify_library_structure(library_path: &Path) -> bool {
    let required_dirs = ["ritmo_library",
        "ritmo_library/config",
        "ritmo_library/database",
        "ritmo_library/storage",
        "bootstrap",
        "bootstrap/portable_app"];
    
    required_dirs.iter().all(|dir| {
        library_path.join(dir).exists()
    })
}

/// Check if the required binaries exist in the expected location
pub fn binaries_exist(library_path: &Path) -> bool {
    let launcher_name = if cfg!(windows) {
        "ritmo_launcher.exe"
    } else {
        "ritmo_launcher"
    };
    
    let gui_name = if cfg!(windows) {
        "ritmo_gui.exe"
    } else {
        "ritmo_gui"
    };
    
    let launcher = library_path.join("bootstrap/portable_app").join(launcher_name);
    let gui = library_path.join("bootstrap/portable_app").join(gui_name);
    
    launcher.exists() && gui.exists()
}

/// Count the number of books in the library storage
pub fn count_books(library_path: &Path) -> Result<usize> {
    let storage_path = library_path.join("ritmo_library/storage");
    
    if !storage_path.exists() {
        return Ok(0);
    }
    
    // Count all book files in storage
    let count = std::fs::read_dir(storage_path)?
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            entry.path().extension().is_some_and(|ext| ext == "toml")
        })
        .count();
    
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_verify_library_structure() {
        let temp_dir = std::env::temp_dir().join("ritmo_test_verify");
        let _ = fs::remove_dir_all(&temp_dir);
        
        // Create minimal structure
        fs::create_dir_all(temp_dir.join("ritmo_library/config")).unwrap();
        fs::create_dir_all(temp_dir.join("ritmo_library/database")).unwrap();
        fs::create_dir_all(temp_dir.join("ritmo_library/storage")).unwrap();
        fs::create_dir_all(temp_dir.join("bootstrap/portable_app")).unwrap();
        
        assert!(verify_library_structure(&temp_dir));
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_count_books() {
        let temp_dir = std::env::temp_dir().join("ritmo_test_books");
        let _ = fs::remove_dir_all(&temp_dir);
        
        let storage_path = temp_dir.join("ritmo_library/storage");
        fs::create_dir_all(&storage_path).unwrap();
        
        // Create some test book files
        fs::write(storage_path.join("book1.toml"), "test").unwrap();
        fs::write(storage_path.join("book2.toml"), "test").unwrap();
        fs::write(storage_path.join("readme.txt"), "not a book").unwrap();
        
        let count = count_books(&temp_dir).unwrap();
        assert_eq!(count, 2);
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
