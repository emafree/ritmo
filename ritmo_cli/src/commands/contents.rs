//! Content-related commands

use crate::formatter::{format_contents, OutputFormat};
use crate::helpers::get_library_path;
use ritmo_config::AppSettings;
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
