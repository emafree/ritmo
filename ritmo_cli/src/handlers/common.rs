//! Common helper functions for handlers
//!
//! This module provides shared utilities to reduce code duplication across handlers.

use ritmo_config::AppSettings;
use ritmo_db_core::LibraryConfig;
use ritmo_errors::reporter::SilentReporter;
use sqlx::SqlitePool;
use std::path::PathBuf;

/// Get library path from CLI arg, app settings, or portable detection
pub fn get_library_path(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    use ritmo_config::detect_portable_library;

    if let Some(path) = cli_library {
        Ok(path.clone())
    } else if let Some(portable) = detect_portable_library() {
        Ok(portable)
    } else if let Some(path) = &app_settings.last_library_path {
        Ok(path.clone())
    } else {
        Err("✗ Nessuna libreria configurata. Usa 'ritmo init' per inizializzare una libreria".into())
    }
}

/// Get library config and database pool
pub async fn get_library_and_pool(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
) -> Result<(LibraryConfig, SqlitePool), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;
    let config = LibraryConfig::new(&library_path);

    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    Ok((config, pool))
}

/// Parse people strings from format "Nome:Ruolo" to (name, role) tuples
pub fn parse_people(people: &[String]) -> Option<Vec<(String, String)>> {
    if people.is_empty() {
        return None;
    }

    let mut result = Vec::new();
    for person_str in people {
        let parts: Vec<&str> = person_str.split(':').collect();
        if parts.len() == 2 {
            result.push((parts[0].to_string(), parts[1].to_string()));
        } else {
            println!(
                "⚠ Formato persona non valido: '{}'. Formato: 'Nome:Ruolo'",
                person_str
            );
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// Print operation summary with success and error counts
pub fn print_summary(success: usize, errors: usize) {
    println!("\n📊 Summary: ✓ {} | ✗ {}", success, errors);
}
