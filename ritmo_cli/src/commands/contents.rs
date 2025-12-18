//! Content-related commands

use crate::formatter::{format_contents, OutputFormat};
use crate::helpers::get_library_path;
use ritmo_config::AppSettings;
use ritmo_core::service::{delete_content, update_content, ContentUpdateMetadata};
use ritmo_db_core::{execute_contents_query, ContentFilters, ContentSortField, LibraryConfig};
use std::path::PathBuf;

/// Comando: list-contents - Lista contenuti con filtri
#[allow(clippy::too_many_arguments)]
pub async fn cmd_list_contents(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    preset: Option<String>,
    author: Option<String>,
    content_type: Option<String>,
    year: Option<i32>,
    search: Option<String>,
    sort: String,
    limit: Option<i64>,
    offset: i64,
    output: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determina quale libreria usare
    let library_path = get_library_path(cli_library, app_settings)?;

    // Crea LibraryConfig e pool di connessioni
    let config = LibraryConfig::new(&library_path);

    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let pool = config.create_pool().await?;

    // Carica preset della libreria per resolution
    let library_presets = config.load_library_presets().ok();

    // Costruisci filtri usando builder pattern
    let mut filters = ContentFilters::default();

    // Se c'è un preset, carica i suoi valori come base
    if let Some(preset_name) = preset {
        // Risolvi preset: library > global
        let preset = if let Some(ref lib_presets) = library_presets {
            lib_presets
                .get_content_preset(&preset_name)
                .or_else(|| app_settings.presets.get_content_preset(&preset_name))
        } else {
            app_settings.presets.get_content_preset(&preset_name)
        }
        .ok_or_else(|| format!("Preset '{}' non trovato", preset_name))?;

        // Applica valori dal preset
        filters = filters
            .set_author_opt(preset.filters.author.clone())
            .set_content_type_opt(preset.filters.content_type.clone());

        filters.year = preset.filters.year;
        filters.search = preset.filters.search.clone();
        filters.limit = preset.filters.limit;
    }

    // Applica parametri CLI (hanno priorità su preset)
    filters = filters
        .set_author_opt(author)
        .set_content_type_opt(content_type);

    if let Some(y) = year {
        filters.year = Some(y);
    }
    if let Some(s) = search {
        filters.search = Some(s);
    }

    filters.sort = ContentSortField::from_str(&sort);
    filters.limit = limit;
    filters.offset = offset;

    // Esegui query
    let contents = execute_contents_query(&pool, &filters).await?;

    // Formatta output
    let output_format = OutputFormat::from_str(&output);
    let formatted = format_contents(&contents, &output_format);

    println!("{}", formatted);

    Ok(())
}

/// Comando: update-content - Aggiorna metadati di un contenuto esistente
#[allow(clippy::too_many_arguments)]
pub async fn cmd_update_content(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    content_id: i64,
    title: Option<String>,
    original_title: Option<String>,
    author: Option<String>,
    content_type: Option<String>,
    year: Option<i32>,
    notes: Option<String>,
    pages: Option<i64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let pool = config.create_pool().await?;

    println!("Aggiornamento contenuto ID {}...", content_id);

    let metadata = ContentUpdateMetadata {
        title,
        original_title,
        author,
        content_type,
        year,
        notes,
        pages,
    };

    match update_content(&pool, content_id, metadata).await {
        Ok(_) => {
            println!("✓ Contenuto aggiornato con successo!");
        }
        Err(e) => {
            println!("✗ Errore durante l'aggiornamento: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

/// Comando: delete-content - Elimina un contenuto dal database
pub async fn cmd_delete_content(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    content_id: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let pool = config.create_pool().await?;

    println!("Eliminazione contenuto ID {}...", content_id);

    match delete_content(&pool, content_id).await {
        Ok(_) => {
            println!("✓ Contenuto eliminato con successo!");
        }
        Err(e) => {
            println!("✗ Errore durante l'eliminazione: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
