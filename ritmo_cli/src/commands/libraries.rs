//! Library management commands

use ritmo_config::{detect_portable_library, AppSettings};
use ritmo_db_core::LibraryConfig;
use rust_i18n::t;
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
        println!("{}", t!("cli.common.portable_mode_detected"));
        portable
    } else if let Some(path) = &app_settings.last_library_path {
        path.clone()
    } else {
        println!("{}", t!("cli.common.no_library_configured"));
        println!("{}", t!("cli.common.use_init"));
        return Ok(());
    };

    println!(
        "{}",
        t!("cli.info.current_library", path = library_path.display().to_string())
    );
    println!();

    // Carica config
    let config = LibraryConfig::new(&library_path);

    if !config.exists() {
        println!("{}", t!("cli.common.library_not_exist"));
        println!(
            "{}",
            t!("cli.info.use_init_to_create", path = library_path.display().to_string())
        );
        return Ok(());
    }

    // Mostra info
    println!("{}", t!("cli.info.structure_label"));
    println!(
        "{}",
        t!("cli.info.database_label", path = config.database_path.display().to_string())
    );
    println!(
        "{}",
        t!("cli.info.storage_label", path = config.storage_path.display().to_string())
    );
    println!(
        "{}",
        t!("cli.info.config_label", path = config.config_path.display().to_string())
    );
    println!(
        "{}",
        t!("cli.info.bootstrap_label", path = config.bootstrap_path.display().to_string())
    );
    println!();

    // Validazione
    if config.validate()? {
        println!("{}", t!("cli.info.structure_valid"));
    } else {
        println!("{}", t!("cli.info.structure_invalid"));
    }

    // Health check
    let issues = config.health_check();
    if issues.is_empty() {
        println!("{}", t!("cli.info.no_issues"));
    } else {
        println!("{}", t!("cli.info.issues_detected"));
        for issue in issues {
            println!("  - {}", issue);
        }
    }

    Ok(())
}

/// Comando: list-libraries - Lista tutte le librerie recenti
pub fn cmd_list_libraries(app_settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
    if app_settings.recent_libraries.is_empty() {
        println!("{}", t!("cli.list_libraries.no_recent"));
        println!("{}", t!("cli.common.use_init"));
        return Ok(());
    }

    println!("{}", t!("cli.list_libraries.recent_libraries"));
    for (i, path) in app_settings.recent_libraries.iter().enumerate() {
        let marker = if Some(path) == app_settings.last_library_path.as_ref() {
            "* "
        } else {
            "  "
        };
        println!("{}{}) {}", marker, i + 1, path.display());
    }

    if let Some(portable) = detect_portable_library() {
        println!(
            "{}",
            t!("cli.list_libraries.portable_mode", path = portable.display().to_string())
        );
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
        println!(
            "{}",
            t!("cli.set_library.not_exist", path = path.display().to_string())
        );
        println!(
            "{}",
            t!("cli.set_library.use_init", path = path.display().to_string())
        );
        return Ok(());
    }

    // Aggiorna settings
    app_settings.update_last_library(&path);
    app_settings.save(settings_path)?;

    println!(
        "{}",
        t!("cli.set_library.success", path = path.display().to_string())
    );

    Ok(())
}
