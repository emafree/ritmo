//! Init command - Initialize a new library

use ritmo_config::AppSettings;
use ritmo_db_core::LibraryConfig;
use std::path::PathBuf;

/// Comando: init - Inizializza una nuova libreria
pub async fn cmd_init(
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

    println!("Inizializzazione libreria: {}", library_path.display());

    // Crea LibraryConfig
    let config = LibraryConfig::new(&library_path);

    // Inizializza directory
    config.initialize()?;
    println!("✓ Directory create");

    // Inizializza database
    config.initialize_database().await?;
    println!("✓ Database inizializzato");

    // Valida
    if config.validate()? {
        println!("✓ Validazione completata");
    } else {
        println!("⚠ Problemi nella validazione");
    }

    // Health check
    let issues = config.health_check();
    if !issues.is_empty() {
        println!("⚠ Problemi rilevati:");
        for issue in issues {
            println!("  - {}", issue);
        }
    }

    // Salva config della libreria
    config.save(config.main_config_file())?;
    println!("✓ Configurazione salvata");

    // Crea preset di esempio per la libreria (load_or_create crea automaticamente gli esempi)
    let _library_presets = config.load_library_presets()?;
    println!("✓ Preset di esempio creati (epub_only, pdf_only, novels)");

    // Aggiorna AppSettings
    app_settings.update_last_library(&library_path);
    app_settings.save(settings_path)?;
    println!("✓ Libreria impostata come corrente");

    println!("\n✓ Libreria inizializzata con successo!");
    println!("  Path: {}", library_path.display());
    println!("\nUsa 'ritmo list-presets' per vedere i preset disponibili.");

    Ok(())
}
