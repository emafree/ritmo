//! Content-related commands

use crate::formatter::{format_contents, OutputFormat};
use crate::helpers::get_library_path;
use ritmo_config::AppSettings;
use ritmo_core::service::{
    create_content, delete_content, link_content_to_book, unlink_content_from_book,
    update_content, ContentCreateMetadata, ContentUpdateMetadata,
};
use ritmo_db_core::{execute_contents_query, ContentFilters, ContentSortField, LibraryConfig};
use ritmo_errors::reporter::SilentReporter;
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

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

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
    people: Vec<String>,
    content_type: Option<String>,
    year: Option<i32>,
    notes: Option<String>,
    pages: Option<i64>,
    tags: Vec<String>,
    languages: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    println!("Aggiornamento contenuto ID {}...", content_id);

    // Parse people from format "Nome:Ruolo"
    let parsed_people = if !people.is_empty() {
        let mut result = Vec::new();
        for person_str in people {
            let parts: Vec<&str> = person_str.split(':').collect();
            if parts.len() != 2 {
                println!(
                    "⚠ Formato persona non valido: '{}'. Formato richiesto: 'Nome:Ruolo'",
                    person_str
                );
                continue;
            }
            result.push((parts[0].to_string(), parts[1].to_string()));
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    } else {
        None
    };

    // Parse languages from format "Name:iso2:iso3:role"
    let parsed_languages = if !languages.is_empty() {
        let mut result = Vec::new();
        for lang_str in languages {
            let parts: Vec<&str> = lang_str.split(':').collect();
            if parts.len() != 4 {
                println!(
                    "⚠ Formato lingua non valido: '{}'. Formato richiesto: 'Nome:iso2:iso3:role'",
                    lang_str
                );
                continue;
            }
            result.push((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
                parts[3].to_string(),
            ));
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    } else {
        None
    };

    let metadata = ContentUpdateMetadata {
        title,
        original_title,
        people: parsed_people,
        content_type,
        year,
        notes,
        pages,
        tags: if tags.is_empty() {
            None
        } else {
            Some(tags)
        },
        languages: parsed_languages,
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

/// Comando: add-content - Crea un nuovo contenuto
#[allow(clippy::too_many_arguments)]
pub async fn cmd_add_content(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    title: String,
    original_title: Option<String>,
    people: Vec<String>,
    content_type: Option<String>,
    year: Option<i32>,
    pages: Option<i64>,
    notes: Option<String>,
    book_id: Option<i64>,
    tags: Vec<String>,
    languages: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    println!("Creazione nuovo contenuto...");

    // Parse people from format "Nome:Ruolo"
    let parsed_people = if !people.is_empty() {
        let mut result = Vec::new();
        for person_str in people {
            let parts: Vec<&str> = person_str.split(':').collect();
            if parts.len() != 2 {
                println!(
                    "⚠ Formato persona non valido: '{}'. Formato richiesto: 'Nome:Ruolo'",
                    person_str
                );
                continue;
            }
            result.push((parts[0].to_string(), parts[1].to_string()));
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    } else {
        None
    };

    // Parse languages from format "Name:iso2:iso3:role"
    let parsed_languages = if !languages.is_empty() {
        let mut result = Vec::new();
        for lang_str in languages {
            let parts: Vec<&str> = lang_str.split(':').collect();
            if parts.len() != 4 {
                println!(
                    "⚠ Formato lingua non valido: '{}'. Formato richiesto: 'Nome:iso2:iso3:role'",
                    lang_str
                );
                continue;
            }
            result.push((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
                parts[3].to_string(),
            ));
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    } else {
        None
    };

    let metadata = ContentCreateMetadata {
        title,
        original_title,
        people: parsed_people,
        content_type,
        year,
        pages,
        notes,
        book_id,
        tags: if tags.is_empty() {
            None
        } else {
            Some(tags)
        },
        languages: parsed_languages,
    };

    match create_content(&pool, metadata).await {
        Ok(content_id) => {
            println!("✓ Contenuto creato con successo! ID: {}", content_id);
        }
        Err(e) => {
            println!("✗ Errore durante la creazione: {}", e);
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

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    println!("Eliminazione contenuto ID {}...", content_id);

    match delete_content(&pool, content_id, &mut reporter).await {
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

/// Comando: link-content - Associa un contenuto a un libro
pub async fn cmd_link_content(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    content_id: i64,
    book_id: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    println!(
        "Associazione contenuto {} al libro {}...",
        content_id, book_id
    );

    match link_content_to_book(&pool, content_id, book_id).await {
        Ok(_) => {
            println!("✓ Contenuto associato con successo!");
        }
        Err(e) => {
            println!("✗ Errore durante l'associazione: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

/// Comando: unlink-content - Rimuovi l'associazione tra un contenuto e un libro
pub async fn cmd_unlink_content(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    content_id: i64,
    book_id: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    println!(
        "Rimozione associazione tra contenuto {} e libro {}...",
        content_id, book_id
    );

    match unlink_content_from_book(&pool, content_id, book_id).await {
        Ok(_) => {
            println!("✓ Associazione rimossa con successo!");
        }
        Err(e) => {
            println!("✗ Errore durante la rimozione: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
