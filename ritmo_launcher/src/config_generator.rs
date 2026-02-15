//! Configuration file generation
//!
//! This module handles generation of default configuration files
//! for the Ritmo library.

use std::path::Path;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Generate default configuration files if they don't exist
pub fn generate_default_config(library_path: &Path) -> Result<()> {
    let config_path = library_path.join("ritmo_library/config");
    std::fs::create_dir_all(&config_path)?;
    
    // Create config.toml if missing
    let config_file = config_path.join("config.toml");
    if !config_file.exists() {
        std::fs::write(
            config_file,
            r#"[library]
name = "My Ritmo Library"
version = "1.0"

[settings]
language = "en"
theme = "default"
"#,
        )?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_generate_default_config() {
        let temp_dir = std::env::temp_dir().join("ritmo_test_config_gen");
        let _ = fs::remove_dir_all(&temp_dir);
        
        generate_default_config(&temp_dir).unwrap();
        
        let config_file = temp_dir.join("ritmo_library/config/config.toml");
        assert!(config_file.exists());
        
        let content = fs::read_to_string(config_file).unwrap();
        assert!(content.contains("[library]"));
        assert!(content.contains("My Ritmo Library"));
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
