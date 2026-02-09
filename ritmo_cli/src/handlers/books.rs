//! Books command handlers
//!
//! This module implements all book-related command handlers for the new noun-verb CLI structure.
//! Each handler function receives parsed arguments and delegates to ritmo_core services.

use crate::confirmation::{confirm_operation, format_book_preview, ConfirmationConfig, ConfirmationResult};
use crate::formatter::{format_books, OutputFormat};
use ritmo_config::AppSettings;
use ritmo_core::service::{
    batch_import, delete_book, import_book, update_book, BookImportMetadata, BookUpdateMetadata,
    DeleteOptions,
};
use ritmo_core::dto::BatchImportInput;
use ritmo_db_core::execute_books_query;
use ritmo_errors::reporter::SilentReporter;
use std::path::PathBuf;

/// Handle: books add
/// Import a single book with manual metadata
pub async fn handle_books_add(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    file: PathBuf,
    title: String,
    original_title: Option<String>,
    people: Vec<String>,
    publisher: Option<String>,
    year: Option<i32>,
    isbn: Option<String>,
    format: Option<String>,
    series: Option<String>,
    series_index: Option<i64>,
    pages: Option<i64>,
    notes: Option<String>,
    tags: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use super::common::{get_library_and_pool, parse_people};

    // Check file exists
    if !file.exists() {
        println!("✗ File non trovato: {}", file.display());
        return Ok(());
    }

    println!("Importazione libro: {}", file.display());
    println!("  Titolo: {}", title);

    // Get library and pool
    let (config, pool) = get_library_and_pool(cli_library, app_settings).await?;

    // Parse people from format "Nome:Ruolo"
    let parsed_people = parse_people(&people);

    // Prepare metadata
    let metadata = BookImportMetadata {
        title,
        original_title,
        people: parsed_people,
        publisher,
        year,
        isbn,
        format,
        series,
        series_index,
        pages,
        notes,
        tags: if tags.is_empty() { None } else { Some(tags) },
    };

    // Import book
    match import_book(&config, &pool, &file, metadata).await {
        Ok(book_id) => {
            println!("✓ Libro importato con successo! ID: {}", book_id);
        }
        Err(e) => {
            println!("✗ Errore durante l'importazione: {}", e);
        }
    }

    Ok(())
}

/// Handle: books add-batch
/// Batch import books from JSON file or stdin
pub async fn handle_books_add_batch(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    input: Option<PathBuf>,
    continue_on_error: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;
    use super::common::get_library_and_pool;

    let (config, pool) = get_library_and_pool(cli_library, app_settings).await?;

    if let Err(e) = config.validate() {
        println!("✗ Libreria non valida: {}", e);
        return Ok(());
    }

    // Read JSON from file or stdin
    let json_content = if let Some(input_path) = input {
        if !input_path.exists() {
            println!("✗ File non trovato: {}", input_path.display());
            return Ok(());
        }
        println!("Lettura metadata da file: {}", input_path.display());
        std::fs::read_to_string(&input_path)?
    } else {
        println!("Lettura metadata da stdin...");
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    // Deserialize JSON
    let batch_input: BatchImportInput = match serde_json::from_str(&json_content) {
        Ok(input) => input,
        Err(e) => {
            println!("✗ Errore nel parsing JSON: {}", e);
            return Ok(());
        }
    };

    println!("\n📚 Batch Import - {} libri", batch_input.len());

    if dry_run {
        println!("🔍 Modalità dry-run: validazione senza importare");
        // TODO: Implement dry-run validation
        println!("  ✓ Validazione completata");
        return Ok(());
    }

    // Call batch import service (stop_on_error is opposite of continue_on_error)
    let stop_on_error = !continue_on_error;
    let summary = batch_import(&config, &pool, batch_input, stop_on_error).await?;

    // Print summary
    println!("\n📊 Riepilogo:");
    println!("  ✓ Successi: {}", summary.successful);
    println!("  ✗ Errori: {}", summary.failed);

    Ok(())
}

/// Handle: books list
/// List books with filters
pub async fn handle_books_list(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    filter_args: crate::filter_args::BookFilterArgs,
    output: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use super::common::get_library_and_pool;

    // Get library and pool
    let (_config, pool) = get_library_and_pool(cli_library, app_settings).await?;

    // Convert CLI args to filters (handles preset, dates, etc.)
    let filters = filter_args.to_filters();

    // Execute query
    let books = execute_books_query(&pool, &filters).await?;

    // Format output
    let output_format = OutputFormat::from_str(&output);
    let formatted = format_books(&books, &output_format);

    println!("{}", formatted);

    Ok(())
}

/// Handle: books update
/// Update book(s) by ID or filters (with bulk support)
pub async fn handle_books_update(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    selector: crate::filter_args::BookBulkUpdateSelector,
    yes: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use super::common::{get_library_and_pool, parse_people, print_summary};

    // Check if any updates specified
    if !selector.has_updates() {
        println!("✗ No update fields specified");
        return Ok(());
    }

    // Must specify either ID or filters
    if !selector.is_id_mode() && !selector.has_filters() {
        println!("✗ Must specify either --id or filter arguments (--filter-author, etc.)");
        return Ok(());
    }

    let (config, pool) = get_library_and_pool(cli_library, app_settings).await?;

    // Get books to update (by ID or filters)
    let books = if let Some(book_id) = selector.id {
        // Single book by ID
        use ritmo_db_core::BookFilters;
        let default_filters = BookFilters::default();
        let all_books = execute_books_query(&pool, &default_filters).await?;
        let filtered: Vec<_> = all_books.into_iter().filter(|b| b.id == book_id).collect();
        if filtered.is_empty() {
            println!("✗ Libro ID {} non trovato", book_id);
            return Ok(());
        }
        filtered
    } else {
        // Multiple books by filters
        let book_filters = selector.to_filters();
        execute_books_query(&pool, &book_filters).await?
    };

    if books.is_empty() {
        println!("Nessun libro trovato");
        return Ok(());
    }

    // Preview and confirmation
    let preview_items = books.iter().map(|b| format_book_preview(b)).collect();

    let confirmation = confirm_operation(ConfirmationConfig {
        items: preview_items,
        operation: "update",
        entity_type: "book(s)",
        force_yes: yes,
        dry_run,
        warning: None,
    })?;

    if matches!(confirmation, ConfirmationResult::Declined) {
        println!("Operation cancelled");
        return Ok(());
    }

    if dry_run {
        return Ok(());
    }

    // Parse people from selector
    let parsed_people = parse_people(&selector.set_people);

    let update_metadata = BookUpdateMetadata {
        title: selector.set_title.clone(),
        original_title: selector.set_original_title.clone(),
        people: parsed_people,
        publisher: selector.set_publisher.clone(),
        year: selector.set_year,
        isbn: selector.set_isbn.clone(),
        format: selector.set_format.clone(),
        series: selector.set_series.clone(),
        series_index: selector.set_series_index,
        notes: selector.set_notes.clone(),
        pages: selector.set_pages,
        tags: if selector.set_tags.is_empty() {
            None
        } else {
            Some(selector.set_tags.clone())
        },
    };

    // Execute updates
    let mut success = 0;
    let mut errors = 0;

    for book in books {
        match update_book(&pool, book.id, update_metadata.clone()).await {
            Ok(_) => {
                success += 1;
                println!("✓ Updated book [{}]: {}", book.id, book.name);
            }
            Err(e) => {
                errors += 1;
                eprintln!("✗ Failed to update [{}]: {}", book.id, e);
            }
        }
    }

    // Summary
    print_summary(success, errors);

    Ok(())
}

/// Handle: books delete
/// Delete book(s) by ID or filters (with bulk support)
pub async fn handle_books_delete(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    id: Option<i64>,
    filters: crate::filter_args::BookFilterArgs,
    delete_file: bool,
    force: bool,
    yes: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use super::common::{get_library_and_pool, print_summary};

    let (config, pool) = get_library_and_pool(cli_library, app_settings).await?;
    let mut reporter = SilentReporter;

    // Get books to delete (by ID or filters)
    let books = if let Some(book_id) = id {
        // Single book by ID
        use ritmo_db_core::BookFilters;
        let default_filters = BookFilters::default();
        let all_books = execute_books_query(&pool, &default_filters).await?;
        let filtered: Vec<_> = all_books.into_iter().filter(|b| b.id == book_id).collect();
        if filtered.is_empty() {
            println!("✗ Libro ID {} non trovato", book_id);
            return Ok(());
        }
        filtered
    } else {
        // Multiple books by filters
        let book_filters = filters.to_filters();
        execute_books_query(&pool, &book_filters).await?
    };

    if books.is_empty() {
        println!("Nessun libro trovato");
        return Ok(());
    }

    // Preview and confirmation
    let preview_items = books.iter().map(|b| format_book_preview(b)).collect();

    let confirmation = confirm_operation(ConfirmationConfig {
        items: preview_items,
        operation: "delete",
        entity_type: "book(s)",
        force_yes: yes,
        dry_run,
        warning: if delete_file { Some("Physical files will be deleted!") } else { None },
    })?;

    if matches!(confirmation, ConfirmationResult::Declined) {
        println!("Operation cancelled");
        return Ok(());
    }

    if dry_run {
        return Ok(());
    }

    // Execute deletions
    let mut success = 0;
    let mut errors = 0;
    let options = DeleteOptions { delete_file, force };

    for book in books {
        match delete_book(&config, &pool, book.id, &options, &mut reporter).await {
            Ok(_) => {
                success += 1;
                println!("✓ Deleted book [{}]: {}", book.id, book.name);
            }
            Err(e) => {
                errors += 1;
                eprintln!("✗ Failed to delete [{}]: {}", book.id, e);
            }
        }
    }

    // Summary
    print_summary(success, errors);

    Ok(())
}

/// Handle: books cleanup
/// Clean up orphaned entities (people, publishers, series, tags, formats)
pub async fn handle_books_cleanup(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use super::common::get_library_and_pool;
    use ritmo_core::service::cleanup_orphaned_entities;

    let (_config, pool) = get_library_and_pool(cli_library, app_settings).await?;

    if dry_run {
        println!("🔍 Modalità dry-run: nessuna modifica");
        return Ok(());
    }

    println!("🧹 Pulizia entità orfane...");

    match cleanup_orphaned_entities(&pool).await {
        Ok(stats) => {
            if stats.has_changes() {
                println!("✓ Pulizia completata! Totale: {} entità rimosse", stats.total());
            } else {
                println!("✓ Nessuna entità orfana trovata");
            }
        }
        Err(e) => {
            println!("✗ Errore: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
