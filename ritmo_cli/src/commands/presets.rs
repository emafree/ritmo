//! Preset management commands

use crate::helpers::{get_library_path, parse_date_to_timestamp};
use ritmo_config::{AppSettings, BookFilterPreset, ContentFilterPreset, NamedPreset, PresetType};
use ritmo_db_core::LibraryConfig;
use std::collections::HashMap;
use std::path::PathBuf;

/// Helper: formatta i filtri di un book preset come stringa
fn format_book_filters(filters: &BookFilterPreset) -> String {
    let mut parts = Vec::new();

    if let Some(a) = &filters.author {
        parts.push(format!("autore={}", a));
    }
    if let Some(p) = &filters.publisher {
        parts.push(format!("editore={}", p));
    }
    if let Some(s) = &filters.series {
        parts.push(format!("serie={}", s));
    }
    if let Some(f) = &filters.format {
        parts.push(format!("formato={}", f));
    }
    if let Some(y) = filters.year {
        parts.push(format!("anno={}", y));
    }
    if let Some(i) = &filters.isbn {
        parts.push(format!("isbn={}", i));
    }
    if let Some(s) = &filters.search {
        parts.push(format!("ricerca={}", s));
    }
    parts.push(format!("ordina={}", filters.sort));
    if let Some(l) = filters.limit {
        parts.push(format!("limite={}", l));
    }

    parts.join(", ")
}

/// Helper: formatta i filtri di un content preset come stringa
fn format_content_filters(filters: &ContentFilterPreset) -> String {
    let mut parts = Vec::new();

    if let Some(a) = &filters.author {
        parts.push(format!("autore={}", a));
    }
    if let Some(t) = &filters.content_type {
        parts.push(format!("tipo={}", t));
    }
    if let Some(y) = filters.year {
        parts.push(format!("anno={}", y));
    }
    if let Some(s) = &filters.search {
        parts.push(format!("ricerca={}", s));
    }
    parts.push(format!("ordina={}", filters.sort));
    if let Some(l) = filters.limit {
        parts.push(format!("limite={}", l));
    }

    parts.join(", ")
}

/// Helper: mostra una lista di book preset
fn display_book_presets(
    title: &str,
    presets: &HashMap<String, NamedPreset<BookFilterPreset>>,
    default_preset: Option<String>,
) {
    println!("{}:", title);
    println!("{}", "-".repeat(50));

    for (name, preset) in presets {
        println!("• {}", name);
        if let Some(desc) = &preset.description {
            println!("  Descrizione: {}", desc);
        }
        println!("  Filtri: {}", format_book_filters(&preset.filters));
        println!();
    }

    if let Some(default) = default_preset {
        println!("Default: {}", default);
        println!();
    }
}

/// Helper: mostra una lista di content preset
fn display_content_presets(
    title: &str,
    presets: &HashMap<String, NamedPreset<ContentFilterPreset>>,
    default_preset: Option<String>,
) {
    println!("{}:", title);
    println!("{}", "-".repeat(50));

    for (name, preset) in presets {
        println!("• {}", name);
        if let Some(desc) = &preset.description {
            println!("  Descrizione: {}", desc);
        }
        println!("  Filtri: {}", format_content_filters(&preset.filters));
        println!();
    }

    if let Some(default) = default_preset {
        println!("Default: {}", default);
        println!();
    }
}

/// Comando: save-preset - Salva un preset di filtri
#[allow(clippy::too_many_arguments)]
pub fn cmd_save_preset(
    cli_library: &Option<PathBuf>,
    app_settings: &mut AppSettings,
    settings_path: &PathBuf,
    preset_type: String,
    name: String,
    in_library: bool,
    description: Option<String>,
    author: Option<String>,
    publisher: Option<String>,
    series: Option<String>,
    format: Option<String>,
    year: Option<i32>,
    isbn: Option<String>,
    search: Option<String>,
    acquired_after: Option<String>,
    acquired_before: Option<String>,
    content_type: Option<String>,
    sort: String,
    limit: Option<i64>,
    offset: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let preset_type_enum = PresetType::from_str(&preset_type).ok_or_else(|| {
        format!(
            "Tipo preset non valido: '{}'. Usa 'books' o 'contents'",
            preset_type
        )
    })?;

    // Converti date da stringa a timestamp se presenti
    let acquired_after_ts = if let Some(date_str) = &acquired_after {
        Some(parse_date_to_timestamp(date_str)?)
    } else {
        None
    };

    let acquired_before_ts = if let Some(date_str) = &acquired_before {
        Some(parse_date_to_timestamp(date_str)?)
    } else {
        None
    };

    // Se in_library è true, salva nei preset della libreria
    if in_library {
        let library_path = get_library_path(cli_library, app_settings)?;
        let config = LibraryConfig::new(&library_path);

        if !config.exists() {
            return Err(format!("La libreria non esiste: {}", library_path.display()).into());
        }

        let mut library_presets = config.load_library_presets()?;

        match preset_type_enum {
            PresetType::Books => {
                let filters = BookFilterPreset {
                    author,
                    publisher,
                    series,
                    format,
                    year,
                    isbn,
                    search,
                    acquired_after: acquired_after_ts,
                    acquired_before: acquired_before_ts,
                    sort,
                    limit,
                    offset,
                };

                let preset = NamedPreset {
                    name: name.clone(),
                    description,
                    filters,
                };

                library_presets.add_book_preset(preset);
                config.save_library_presets(&library_presets)?;

                println!("✓ Preset '{}' salvato nella libreria per libri", name);
            }
            PresetType::Contents => {
                let filters = ContentFilterPreset {
                    author,
                    content_type,
                    year,
                    search,
                    sort,
                    limit,
                    offset,
                };

                let preset = NamedPreset {
                    name: name.clone(),
                    description,
                    filters,
                };

                library_presets.add_content_preset(preset);
                config.save_library_presets(&library_presets)?;

                println!("✓ Preset '{}' salvato nella libreria per contenuti", name);
            }
        }
    } else {
        // Salva nei preset globali
        match preset_type_enum {
            PresetType::Books => {
                let filters = BookFilterPreset {
                    author,
                    publisher,
                    series,
                    format,
                    year,
                    isbn,
                    search,
                    acquired_after: acquired_after_ts,
                    acquired_before: acquired_before_ts,
                    sort,
                    limit,
                    offset,
                };

                let preset = NamedPreset {
                    name: name.clone(),
                    description,
                    filters,
                };

                app_settings.presets.add_book_preset(preset);
                app_settings.save(settings_path)?;

                println!("✓ Preset '{}' salvato globalmente per libri", name);
            }
            PresetType::Contents => {
                let filters = ContentFilterPreset {
                    author,
                    content_type,
                    year,
                    search,
                    sort,
                    limit,
                    offset,
                };

                let preset = NamedPreset {
                    name: name.clone(),
                    description,
                    filters,
                };

                app_settings.presets.add_content_preset(preset);
                app_settings.save(settings_path)?;

                println!("✓ Preset '{}' salvato globalmente per contenuti", name);
            }
        }
    }

    Ok(())
}

/// Comando: list-presets - Lista tutti i preset salvati
pub fn cmd_list_presets(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    preset_type: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let show_books = preset_type.is_none()
        || preset_type.as_ref().map(|s| s.to_lowercase()) == Some("books".to_string())
        || preset_type.as_ref().map(|s| s.to_lowercase()) == Some("book".to_string());

    let show_contents = preset_type.is_none()
        || preset_type.as_ref().map(|s| s.to_lowercase()) == Some("contents".to_string())
        || preset_type.as_ref().map(|s| s.to_lowercase()) == Some("content".to_string());

    // Prova a caricare preset della libreria
    let library_presets = if let Ok(library_path) = get_library_path(cli_library, app_settings) {
        let config = LibraryConfig::new(&library_path);
        if config.exists() {
            config.load_library_presets().ok()
        } else {
            None
        }
    } else {
        None
    };

    let mut found_any = false;

    // Mostra preset della libreria
    if let Some(ref lib_presets) = library_presets {
        if show_books && !lib_presets.books.is_empty() {
            display_book_presets(
                "Preset per Libri (Libreria)",
                &lib_presets.books,
                lib_presets
                    .get_default_books_preset()
                    .map(|s| s.to_string()),
            );
            found_any = true;
        }

        if show_contents && !lib_presets.contents.is_empty() {
            display_content_presets(
                "Preset per Contenuti (Libreria)",
                &lib_presets.contents,
                lib_presets
                    .get_default_contents_preset()
                    .map(|s| s.to_string()),
            );
            found_any = true;
        }
    }

    // Mostra preset globali
    if show_books && !app_settings.presets.books.is_empty() {
        display_book_presets(
            "Preset per Libri (Globali)",
            &app_settings.presets.books,
            None,
        );
        found_any = true;
    }

    if show_contents && !app_settings.presets.contents.is_empty() {
        if found_any {
            println!();
        }
        display_content_presets(
            "Preset per Contenuti (Globali)",
            &app_settings.presets.contents,
            None,
        );
        found_any = true;
    }

    if !found_any {
        println!("Nessun preset salvato.");
        println!("Usa 'ritmo save-preset' per salvare un nuovo preset.");
    }

    Ok(())
}

/// Comando: delete-preset - Elimina un preset
pub fn cmd_delete_preset(
    app_settings: &mut AppSettings,
    settings_path: &PathBuf,
    preset_type: String,
    name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let preset_type_enum = PresetType::from_str(&preset_type).ok_or_else(|| {
        format!(
            "Tipo preset non valido: '{}'. Usa 'books' o 'contents'",
            preset_type
        )
    })?;

    let removed = match preset_type_enum {
        PresetType::Books => app_settings.presets.remove_book_preset(&name).is_some(),
        PresetType::Contents => app_settings.presets.remove_content_preset(&name).is_some(),
    };

    if removed {
        app_settings.save(settings_path)?;
        println!("✓ Preset '{}' eliminato", name);
    } else {
        println!("✗ Preset '{}' non trovato", name);
    }

    Ok(())
}

/// Comando: set-default-filter - Imposta il preset di default per una libreria
pub fn cmd_set_default_filter(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    preset_type: String,
    preset_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let preset_type_enum = PresetType::from_str(&preset_type).ok_or_else(|| {
        format!(
            "Tipo preset non valido: '{}'. Usa 'books' o 'contents'",
            preset_type
        )
    })?;

    let library_path = get_library_path(cli_library, app_settings)?;
    let config = LibraryConfig::new(&library_path);

    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let mut library_presets = config.load_library_presets()?;

    // Controlla se rimuovere il default
    if preset_name.to_lowercase() == "none" {
        match preset_type_enum {
            PresetType::Books => {
                library_presets.set_default_books_preset(None);
                println!("✓ Preset di default rimosso per libri");
            }
            PresetType::Contents => {
                library_presets.set_default_contents_preset(None);
                println!("✓ Preset di default rimosso per contenuti");
            }
        }
    } else {
        // Verifica che il preset esista
        let exists = match preset_type_enum {
            PresetType::Books => library_presets.get_book_preset(&preset_name).is_some(),
            PresetType::Contents => library_presets.get_content_preset(&preset_name).is_some(),
        };

        if !exists {
            return Err(format!(
                "Preset '{}' non trovato nella libreria. Usa 'ritmo save-preset --in-library' per crearlo.",
                preset_name
            )
            .into());
        }

        match preset_type_enum {
            PresetType::Books => {
                library_presets.set_default_books_preset(Some(preset_name.clone()));
                println!(
                    "✓ Preset '{}' impostato come default per libri",
                    preset_name
                );
            }
            PresetType::Contents => {
                library_presets.set_default_contents_preset(Some(preset_name.clone()));
                println!(
                    "✓ Preset '{}' impostato come default per contenuti",
                    preset_name
                );
            }
        }
    }

    config.save_library_presets(&library_presets)?;

    Ok(())
}
