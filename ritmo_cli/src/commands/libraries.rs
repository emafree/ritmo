//! Library management commands

use ritmo_config::{detect_portable_library, AppSettings};
use ritmo_db_core::LibraryConfig;
use std::path::PathBuf;

/// Comando: info - Mostra informazioni sulla libreria corrente
pub async fn cmd_info(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determina quale libreria usare
    let library_path = if let Some(path) = cli_library {
        path.clone()
    } else if let Some(portable) = detect_portable_library() {
        println!("ℹ Modalità portabile rilevata");
        portable
    } else if let Some(path) = &app_settings.last_library_path {
        path.clone()
    } else {
        println!("✗ Nessuna libreria configurata");
        println!("  Usa 'ritmo init' per inizializzare una libreria");
        return Ok(());
    };

    println!("Libreria corrente: {}", library_path.display());
    println!();

    // Carica config
    let config = LibraryConfig::new(&library_path);

    if !config.exists() {
        println!("✗ La libreria non esiste");
        println!("  Usa 'ritmo init {}' per crearla", library_path.display());
        return Ok(());
    }

    // Mostra info
    println!("Struttura:");
    println!("  Database:  {}", config.database_path.display());
    println!("  Storage:   {}", config.storage_path.display());
    println!("  Config:    {}", config.config_path.display());
    println!("  Bootstrap: {}", config.bootstrap_path.display());
    println!();

    // Validazione
    if config.validate()? {
        println!("✓ Struttura valida");
    } else {
        println!("✗ Struttura non valida");
    }

    // Health check
    let issues = config.health_check();
    if issues.is_empty() {
        println!("✓ Nessun problema rilevato");
    } else {
        println!("⚠ Problemi rilevati:");
        for issue in issues {
            println!("  - {}", issue);
        }
    }

    Ok(())
}

/// Comando: list-libraries - Lista tutte le librerie recenti
pub fn cmd_list_libraries(app_settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
    if app_settings.recent_libraries.is_empty() {
        println!("Nessuna libreria recente trovata");
        println!("Usa 'ritmo init' per inizializzare una libreria");
        return Ok(());
    }

    println!("Librerie recenti:");
    for (i, path) in app_settings.recent_libraries.iter().enumerate() {
        let marker = if Some(path) == app_settings.last_library_path.as_ref() {
            "* "
        } else {
            "  "
        };
        println!("{}{}) {}", marker, i + 1, path.display());
    }

    if let Some(portable) = detect_portable_library() {
        println!("\nℹ Modalità portabile: {}", portable.display());
    }

    Ok(())
}

/// Comando: set-library - Imposta la libreria corrente
pub fn cmd_set_library(
    path: PathBuf,
    app_settings: &mut AppSettings,
    settings_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Verifica che la libreria esista
    let config = LibraryConfig::new(&path);
    if !config.exists() {
        println!("✗ La libreria non esiste: {}", path.display());
        println!("  Usa 'ritmo init {}' per crearla", path.display());
        return Ok(());
    }

    // Aggiorna settings
    app_settings.update_last_library(&path);
    app_settings.save(settings_path)?;

    println!("✓ Libreria impostata come corrente: {}", path.display());

    Ok(())
}
