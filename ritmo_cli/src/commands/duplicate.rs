//! Duplicate command - Duplicate a portable library to a new location

use ritmo_config::{detect_portable_library, AppSettings};
use ritmo_db_core::{LibraryConfig, DB_TEMPLATE};
use rust_i18n::t;
use std::fs;
use std::path::PathBuf;

/// Comando: duplicate - Duplica la libreria portabile corrente in una nuova posizione
pub async fn cmd_duplicate(
    output_path: PathBuf,
    app_settings: &mut AppSettings,
    settings_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if running in portable mode
    let source_library = detect_portable_library().ok_or_else(|| {
        println!("{}", t!("cli.duplicate.error_not_portable"));
        println!("{}", t!("cli.duplicate.use_init_instead"));
        "Not running in portable mode"
    })?;

    println!(
        "{}",
        t!("cli.duplicate.initializing", path = output_path.display().to_string())
    );

    // Step 1: Copy entire library structure to output path
    println!("{}", t!("cli.duplicate.copying_files"));
    copy_directory_recursive(&source_library, &output_path)?;

    // Step 2: Reset database to template
    println!("{}", t!("cli.duplicate.resetting_database"));
    let config = LibraryConfig::new(&output_path);
    
    // Ensure database directory exists
    fs::create_dir_all(&config.database_path)?;
    
    // Write fresh template database
    let db_path = config.db_file_path();
    fs::write(&db_path, DB_TEMPLATE)?;

    // Step 3: Update library config (reinitialize to ensure paths are correct)
    println!("{}", t!("cli.duplicate.updating_config"));
    
    // Initialize ensures all directories exist with correct structure
    config.initialize()?;
    
    // Save config to new location
    config.save(config.main_config_file())?;

    // Create library presets
    let _library_presets = config.load_library_presets()?;

    // Step 4: Update AppSettings with new library as last_library
    app_settings.update_last_library(&output_path);
    app_settings.save(settings_path)?;

    println!("{}", t!("cli.duplicate.success"));
    println!(
        "{}",
        t!("cli.duplicate.path_label", path = output_path.display().to_string())
    );

    Ok(())
}

/// Recursively copy a directory and all its contents
fn copy_directory_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Create destination directory
    fs::create_dir_all(dst)?;

    // Read source directory
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if file_type.is_dir() {
            // Recursively copy subdirectory
            copy_directory_recursive(&src_path, &dst_path)?;
        } else {
            // Copy file
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
