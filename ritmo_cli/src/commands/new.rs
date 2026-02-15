//! New command - Initialize a new library

use ritmo_config::AppSettings;
use ritmo_db_core::LibraryConfig;
use rust_i18n::t;
use std::path::PathBuf;

/// Command: new - Initialize a new library
pub async fn cmd_new(
    path: Option<PathBuf>,
    app_settings: &mut AppSettings,
    settings_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determina il path della libreria
    let library_path = path.unwrap_or_else(|| {
        dirs::home_dir()
            .map(|p| p.join("RitmoLibrary"))
            .unwrap_or_else(|| PathBuf::from("./RitmoLibrary"))
    });

    println!(
        "{}",
        t!("cli.new.initializing", path = library_path.display().to_string())
    );

    // Crea LibraryConfig
    let config = LibraryConfig::new(&library_path);

    // Inizializza directory
    config.initialize()?;
    println!("{}", t!("cli.new.directories_created"));

    // Inizializza database
    config.initialize_database().await?;
    println!("{}", t!("cli.new.database_initialized"));

    // Valida
    if config.validate()? {
        println!("{}", t!("cli.new.validation_completed"));
    } else {
        println!("{}", t!("cli.new.validation_issues"));
    }

    // Health check
    let issues = config.health_check();
    if !issues.is_empty() {
        println!("{}", t!("cli.new.issues_detected"));
        for issue in issues {
            println!("  - {}", issue);
        }
    }

    // Salva config della libreria
    config.save(config.main_config_file())?;
    println!("{}", t!("cli.new.config_saved"));

    // Crea preset di esempio per la libreria (load_or_create crea automaticamente gli esempi)
    let _library_presets = config.load_library_presets()?;
    println!("{}", t!("cli.new.presets_created"));

    // Aggiorna AppSettings
    app_settings.update_last_library(&library_path);
    app_settings.save(settings_path)?;
    println!("{}", t!("cli.new.library_set_current"));

    println!("{}", t!("cli.new.success"));
    println!(
        "{}",
        t!("cli.new.path_label", path = library_path.display().to_string())
    );
    println!("{}", t!("cli.new.use_list_presets"));

    Ok(())
}
