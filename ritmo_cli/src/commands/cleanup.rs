//! Cleanup and maintenance commands
//!
//! Questo modulo fornisce comandi per la pulizia e manutenzione del database,
//! in particolare la rimozione di entità orfane che non sono più referenziate
//! da libri o contenuti.
//!
//! ## Comando cleanup
//!
//! Rimuove dal database:
//! - Persone (autori, traduttori, etc.) non associate a libri o contenuti
//! - Editori non referenziati da libri
//! - Serie non referenziate da libri
//! - Formati non usati da libri
//! - Tipi non usati da contenuti
//! - Tag non associati a libri o contenuti
//!
//! Tipicamente usato dopo aver eliminato libri con `delete-book`.

use crate::helpers::get_library_path;
use ritmo_config::AppSettings;
use ritmo_core::service::cleanup_orphaned_entities;
use ritmo_db_core::LibraryConfig;
use ritmo_errors::reporter::SilentReporter;
use std::path::PathBuf;

/// Comando: cleanup - Rimuove entità orfane dal database
pub async fn cmd_cleanup(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    if dry_run {
        println!("🔍 Modalità dry-run: nessuna modifica verrà applicata");
        println!("   (funzionalità non ancora implementata)");
        return Ok(());
    }

    println!("🧹 Pulizia entità orfane in corso...");

    match cleanup_orphaned_entities(&pool).await {
        Ok(stats) => {
            if stats.has_changes() {
                println!("✓ Pulizia completata!");
                println!("  📊 Statistiche:");
                if stats.people_removed > 0 {
                    println!("     - Persone rimosse: {}", stats.people_removed);
                }
                if stats.publishers_removed > 0 {
                    println!("     - Editori rimossi: {}", stats.publishers_removed);
                }
                if stats.series_removed > 0 {
                    println!("     - Serie rimosse: {}", stats.series_removed);
                }
                if stats.formats_removed > 0 {
                    println!("     - Formati rimossi: {}", stats.formats_removed);
                }
                if stats.types_removed > 0 {
                    println!("     - Tipi rimossi: {}", stats.types_removed);
                }
                if stats.tags_removed > 0 {
                    println!("     - Tag rimossi: {}", stats.tags_removed);
                }
                println!("  🎯 Totale: {} entità rimosse", stats.total());
            } else {
                println!("✓ Nessuna entità orfana trovata. Il database è pulito!");
            }
        }
        Err(e) => {
            println!("✗ Errore durante la pulizia: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
