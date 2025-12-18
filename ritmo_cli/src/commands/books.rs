//! Book-related commands

use crate::formatter::{format_books, OutputFormat};
use crate::helpers::{
    get_library_path, parse_date_to_timestamp, timestamp_days_ago, timestamp_months_ago,
};
use ritmo_config::{detect_portable_library, AppSettings};
use ritmo_core::service::{
    delete_book, import_book, update_book, BookImportMetadata, BookUpdateMetadata, DeleteOptions,
};
use ritmo_db_core::{execute_books_query, BookFilters, BookSortField, LibraryConfig};
use std::path::PathBuf;

/// Comando: list-books - Lista libri con filtri
#[allow(clippy::too_many_arguments)]
pub async fn cmd_list_books(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    preset: Option<String>,
    author: Option<String>,
    publisher: Option<String>,
    series: Option<String>,
    format: Option<String>,
    year: Option<i32>,
    isbn: Option<String>,
    search: Option<String>,
    acquired_after: Option<String>,
    acquired_before: Option<String>,
    last_days: Option<i64>,
    last_months: Option<i64>,
    recent_count: Option<i64>,
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

    // Gestisci filtri di data relativi
    let acquired_after_ts = if let Some(days) = last_days {
        // Usa filtro relativo: ultimi N giorni
        Some(timestamp_days_ago(days))
    } else if let Some(months) = last_months {
        // Usa filtro relativo: ultimi N mesi
        Some(timestamp_months_ago(months))
    } else if let Some(date_str) = &acquired_after {
        // Usa filtro assoluto
        Some(parse_date_to_timestamp(date_str)?)
    } else {
        None
    };

    let acquired_before_ts = if let Some(date_str) = &acquired_before {
        Some(parse_date_to_timestamp(date_str)?)
    } else {
        None
    };

    // Gestisci recent_count: override sort e limit
    let (final_sort, final_limit) = if let Some(count) = recent_count {
        ("date_added".to_string(), Some(count))
    } else {
        (sort, limit)
    };

    // Carica preset della libreria per resolution
    let library_presets = config.load_library_presets().ok();

    // Costruisci filtri usando builder pattern
    let mut filters = BookFilters::default();

    // Se c'è un preset, carica i suoi valori come base
    if let Some(preset_name) = preset {
        // Risolvi preset: library > global
        let preset = if let Some(ref lib_presets) = library_presets {
            lib_presets
                .get_book_preset(&preset_name)
                .or_else(|| app_settings.presets.get_book_preset(&preset_name))
        } else {
            app_settings.presets.get_book_preset(&preset_name)
        }
        .ok_or_else(|| format!("Preset '{}' non trovato", preset_name))?;

        // Applica valori dal preset (i parametri CLI sovrascriveranno questi)
        filters = filters
            .set_author_opt(preset.filters.author.clone())
            .set_publisher_opt(preset.filters.publisher.clone())
            .set_series_opt(preset.filters.series.clone())
            .set_format_opt(preset.filters.format.clone());

        filters.year = preset.filters.year;
        filters.isbn = preset.filters.isbn.clone();
        filters.search = preset.filters.search.clone();
        filters.acquired_after = preset.filters.acquired_after;
        filters.acquired_before = preset.filters.acquired_before;
        filters.limit = preset.filters.limit;
    }

    // Applica parametri CLI (hanno priorità su preset)
    filters = filters
        .set_author_opt(author)
        .set_publisher_opt(publisher)
        .set_series_opt(series)
        .set_format_opt(format);

    if let Some(y) = year {
        filters.year = Some(y);
    }
    if let Some(i) = isbn {
        filters.isbn = Some(i);
    }
    if let Some(s) = search {
        filters.search = Some(s);
    }
    if let Some(aa) = acquired_after_ts {
        filters.acquired_after = Some(aa);
    }
    if let Some(ab) = acquired_before_ts {
        filters.acquired_before = Some(ab);
    }

    filters.sort = BookSortField::from_str(&final_sort);
    filters.limit = final_limit;
    filters.offset = offset;

    // Esegui query
    let books = execute_books_query(&pool, &filters).await?;

    // Formatta output
    let output_format = OutputFormat::from_str(&output);
    let formatted = format_books(&books, &output_format);

    println!("{}", formatted);

    Ok(())
}

/// Comando: update-book - Aggiorna metadati di un libro esistente
#[allow(clippy::too_many_arguments)]
pub async fn cmd_update_book(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    book_id: i64,
    title: Option<String>,
    original_title: Option<String>,
    author: Option<String>,
    publisher: Option<String>,
    year: Option<i32>,
    isbn: Option<String>,
    format: Option<String>,
    series: Option<String>,
    series_index: Option<i64>,
    notes: Option<String>,
    pages: Option<i64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let pool = config.create_pool().await?;

    println!("Aggiornamento libro ID {}...", book_id);

    let metadata = BookUpdateMetadata {
        title,
        original_title,
        author,
        publisher,
        year,
        isbn,
        format,
        series,
        series_index,
        notes,
        pages,
    };

    match update_book(&pool, book_id, metadata).await {
        Ok(_) => {
            println!("✓ Libro aggiornato con successo!");
        }
        Err(e) => {
            println!("✗ Errore durante l'aggiornamento: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

/// Comando: delete-book - Elimina un libro dal database
pub async fn cmd_delete_book(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    book_id: i64,
    delete_file: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let pool = config.create_pool().await?;

    println!("Eliminazione libro ID {}...", book_id);
    if delete_file {
        println!("  ⚠ Il file fisico verrà eliminato");
    }

    let options = DeleteOptions { delete_file, force };

    match delete_book(&config, &pool, book_id, &options).await {
        Ok(_) => {
            println!("✓ Libro eliminato con successo!");
        }
        Err(e) => {
            println!("✗ Errore durante l'eliminazione: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

/// Comando: add - Importa un libro nella libreria
#[allow(clippy::too_many_arguments)]
pub async fn cmd_add(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    file: PathBuf,
    title: String,
    author: Option<String>,
    publisher: Option<String>,
    year: Option<i32>,
    isbn: Option<String>,
    format: Option<String>,
    series: Option<String>,
    series_index: Option<i64>,
    notes: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determina quale libreria usare
    let library_path = if let Some(path) = cli_library {
        path.clone()
    } else if let Some(portable) = detect_portable_library() {
        portable
    } else if let Some(path) = &app_settings.last_library_path {
        path.clone()
    } else {
        println!("✗ Nessuna libreria configurata");
        println!("  Usa 'ritmo init' per inizializzare una libreria");
        return Ok(());
    };

    // Verifica che il file esista
    if !file.exists() {
        println!("✗ File non trovato: {}", file.display());
        return Ok(());
    }

    println!("Importazione libro: {}", file.display());
    println!("  Titolo: {}", title);
    if let Some(ref a) = author {
        println!("  Autore: {}", a);
    }
    if let Some(ref p) = publisher {
        println!("  Editore: {}", p);
    }
    if let Some(y) = year {
        println!("  Anno: {}", y);
    }

    // Crea config e pool
    let config = LibraryConfig::new(&library_path);
    let pool = config.create_pool().await?;

    // Prepara metadati
    let metadata = BookImportMetadata {
        title,
        author,
        publisher,
        year,
        isbn,
        format,
        series,
        series_index,
        notes,
    };

    // Importa il libro
    match import_book(&config, &pool, &file, metadata).await {
        Ok(book_id) => {
            println!("✓ Libro importato con successo!");
            println!("  ID: {}", book_id);
        }
        Err(e) => {
            println!("✗ Errore durante l'importazione: {}", e);
        }
    }

    Ok(())
}
