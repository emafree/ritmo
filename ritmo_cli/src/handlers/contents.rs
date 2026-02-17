//! Contents command handlers
//!
//! This module implements all content-related command handlers for the new noun-verb CLI structure.
//! Each handler function receives parsed arguments and delegates to ritmo_commands.

use crate::confirmation::{confirm_operation, format_content_preview, ConfirmationConfig, ConfirmationResult};
use crate::formatter::{format_contents, OutputFormat};
use ritmo_commands::{
    Command,
    contents::{
        AddContentCommand, AddContentInput,
        ListContentsCommand, ListContentsInput,
        UpdateContentCommand, UpdateContentInput,
        DeleteContentCommand, DeleteContentInput,
        ModifyLinkContentCommand, ModifyLinkContentInput,
    }
};
use ritmo_config::AppSettings;
use ritmo_db_core::execute_contents_query;
use std::path::PathBuf;

/// Handle: contents add
/// Create new content with metadata
pub async fn handle_contents_add(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    title: String,
    original_title: Option<String>,
    people: Vec<String>,
    content_type: Option<String>,
    genre: Option<String>,
    year: Option<i32>,
    notes: Option<String>,
    tags: Vec<String>,
    languages: Vec<String>,
    book_id: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    use super::common::{get_library_and_pool, parse_people};

    println!("Creazione nuovo contenuto:");
    println!("  Titolo: {}", title);

    let (_config, pool) = get_library_and_pool(cli_library, app_settings).await?;

    // Parse people
    let parsed_people = parse_people(&people);

    // Parse languages (formato: "name:iso2:iso3:role" o "it:it:it:original")
    let parsed_languages = if !languages.is_empty() {
        let mut result = Vec::new();
        for lang_str in &languages {
            let parts: Vec<&str> = lang_str.split(':').collect();
            if parts.len() == 4 {
                result.push((
                    parts[0].to_string(),
                    parts[1].to_string(),
                    parts[2].to_string(),
                    parts[3].to_string(),
                ));
            } else if parts.len() == 2 {
                // Shortcut: "it:original" → ("Italian", "it", "ita", "original")
                let iso2 = parts[0];
                let role = parts[1];
                let (name, iso3) = match iso2 {
                    "it" => ("Italian", "ita"),
                    "en" => ("English", "eng"),
                    "fr" => ("French", "fra"),
                    "de" => ("German", "deu"),
                    "es" => ("Spanish", "spa"),
                    _ => (iso2, iso2), // Fallback
                };
                result.push((name.to_string(), iso2.to_string(), iso3.to_string(), role.to_string()));
            } else {
                println!(
                    "⚠ Formato lingua non valido: '{}'. Formati: 'codice:ruolo' (es. 'it:original') o 'name:iso2:iso3:role'",
                    lang_str
                );
            }
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    } else {
        None
    };

    // Prepare command input
    let input = AddContentInput {
        title,
        original_title,
        people: parsed_people,
        content_type,
        genre,
        year,
        notes,
        tags: if tags.is_empty() { None } else { Some(tags) },
        languages: parsed_languages,
        book_id,
    };

    // Execute command
    let command = AddContentCommand;
    match command.execute(&_config, &pool, input).await {
        Ok(result) => {
            println!("✓ Contenuto creato con successo! ID: {}", result.content_id);
            // book_id is guaranteed to be Some since it's required in the input
            if let Some(bid) = result.book_id {
                println!("  Associato al libro ID: {}", bid);
            }
        }
        Err(e) => {
            println!("✗ Errore durante la creazione: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

/// Handle: contents list
/// List contents with filters
pub async fn handle_contents_list(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    filter_args: crate::filter_args::ContentFilterArgs,
    output: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use super::common::get_library_and_pool;

    let (config, pool) = get_library_and_pool(cli_library, app_settings).await?;

    // Convert CLI args to filters
    let filters = filter_args.to_filters();

    // Prepare command input
    let input = ListContentsInput { filters };

    // Execute command
    let command = ListContentsCommand;
    let _result = command.execute(&config, &pool, input).await?;

    // Format output (TODO: create format_content_summaries similar to books)
    // For now, we need to use execute_contents_query for formatting compatibility
    let filters_for_display = filter_args.to_filters();
    let contents = execute_contents_query(&pool, &filters_for_display).await?;
    let output_format = OutputFormat::from_str(&output);
    let formatted = format_contents(&contents, &output_format);

    println!("{}", formatted);

    Ok(())
}

/// Handle: contents update
/// Update content(s) by ID or filters (with bulk support)
pub async fn handle_contents_update(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    selector: crate::filter_args::ContentBulkUpdateSelector,
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

    let (_config, pool) = get_library_and_pool(cli_library, app_settings).await?;

    // Get contents to update (by ID or filters)
    let contents = if let Some(content_id) = selector.id {
        // Single content by ID
        use ritmo_db_core::ContentFilters;
        let default_filters = ContentFilters::default();
        let all_contents = execute_contents_query(&pool, &default_filters).await?;
        let filtered: Vec<_> = all_contents
            .into_iter()
            .filter(|c| c.id == content_id)
            .collect();
        if filtered.is_empty() {
            println!("✗ Contenuto ID {} non trovato", content_id);
            return Ok(());
        }
        filtered
    } else {
        // Multiple contents by filters
        let content_filters = selector.to_filters();
        execute_contents_query(&pool, &content_filters).await?
    };

    if contents.is_empty() {
        println!("Nessun contenuto trovato");
        return Ok(());
    }

    // Preview and confirmation
    let preview_items = contents.iter().map(|c| format_content_preview(c)).collect();

    let confirmation = confirm_operation(ConfirmationConfig {
        items: preview_items,
        operation: "update",
        entity_type: "content(s)",
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

    // Parse languages (same format as in add)
    let parsed_languages = if !selector.set_languages.is_empty() {
        let mut result = Vec::new();
        for lang_str in &selector.set_languages {
            let parts: Vec<&str> = lang_str.split(':').collect();
            if parts.len() == 4 {
                result.push((
                    parts[0].to_string(),
                    parts[1].to_string(),
                    parts[2].to_string(),
                    parts[3].to_string(),
                ));
            } else if parts.len() == 2 {
                // Shortcut: "it:original" → ("Italian", "it", "ita", "original")
                let iso2 = parts[0];
                let role = parts[1];
                let (name, iso3) = match iso2 {
                    "it" => ("Italian", "ita"),
                    "en" => ("English", "eng"),
                    "fr" => ("French", "fra"),
                    "de" => ("German", "deu"),
                    "es" => ("Spanish", "spa"),
                    _ => (iso2, iso2),
                };
                result.push((name.to_string(), iso2.to_string(), iso3.to_string(), role.to_string()));
            }
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    } else {
        None
    };

    // Execute updates using command
    let mut success = 0;
    let mut errors = 0;
    let command = UpdateContentCommand;

    for content in contents {
        let input = UpdateContentInput {
            content_id: content.id,
            title: selector.set_title.clone(),
            original_title: selector.set_original_title.clone(),
            people: parsed_people.clone(),
            content_type: selector.set_content_type.clone(),
            genre: selector.set_genre.clone(),
            year: selector.set_year,
            notes: selector.set_notes.clone(),
            pages: selector.set_pages,
            tags: if selector.set_tags.is_empty() {
                None
            } else {
                Some(selector.set_tags.clone())
            },
            languages: parsed_languages.clone(),
        };

        match command.execute(&_config, &pool, input).await {
            Ok(_) => {
                success += 1;
                println!("✓ Updated content [{}]: {}", content.id, content.name);
            }
            Err(e) => {
                errors += 1;
                eprintln!("✗ Failed to update [{}]: {}", content.id, e);
            }
        }
    }

    // Summary
    print_summary(success, errors);

    Ok(())
}

/// Handle: contents delete
/// Delete content(s) by ID or filters (with bulk support)
pub async fn handle_contents_delete(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    id: Option<i64>,
    filters: crate::filter_args::ContentFilterArgs,
    yes: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use super::common::{get_library_and_pool, print_summary};

    let (_config, pool) = get_library_and_pool(cli_library, app_settings).await?;

    // Get contents to delete (by ID or filters)
    let contents = if let Some(content_id) = id {
        // Single content by ID
        use ritmo_db_core::ContentFilters;
        let default_filters = ContentFilters::default();
        let all_contents = execute_contents_query(&pool, &default_filters).await?;
        let filtered: Vec<_> = all_contents
            .into_iter()
            .filter(|c| c.id == content_id)
            .collect();
        if filtered.is_empty() {
            println!("✗ Contenuto ID {} non trovato", content_id);
            return Ok(());
        }
        filtered
    } else {
        // Multiple contents by filters
        let content_filters = filters.to_filters();
        execute_contents_query(&pool, &content_filters).await?
    };

    if contents.is_empty() {
        println!("Nessun contenuto trovato");
        return Ok(());
    }

    // Preview and confirmation
    let preview_items = contents.iter().map(|c| format_content_preview(c)).collect();

    let confirmation = confirm_operation(ConfirmationConfig {
        items: preview_items,
        operation: "delete",
        entity_type: "content(s)",
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

    // Execute deletions using command
    let mut success = 0;
    let mut errors = 0;
    let command = DeleteContentCommand;

    for content in contents {
        let input = DeleteContentInput {
            content_id: content.id,
        };

        match command.execute(&_config, &pool, input).await {
            Ok(_) => {
                success += 1;
                println!("✓ Deleted content [{}]: {}", content.id, content.name);
            }
            Err(e) => {
                errors += 1;
                eprintln!("✗ Failed to delete [{}]: {}", content.id, e);
            }
        }
    }

    // Summary
    print_summary(success, errors);

    Ok(())
}

/// Handle: contents modify-link
/// Modify the link between content and book
pub async fn handle_contents_modify_link(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    content_id: i64,
    new_book_id: i64,
    old_book_id: Option<i64>,
) -> Result<(), Box<dyn std::error::Error>> {
    use super::common::get_library_and_pool;

    let (config, pool) = get_library_and_pool(cli_library, app_settings).await?;

    if let Some(old_id) = old_book_id {
        println!(
            "Modifica associazione contenuto {} dal libro {} al libro {}...",
            content_id, old_id, new_book_id
        );
    } else {
        println!(
            "Associazione contenuto {} al libro {}...",
            content_id, new_book_id
        );
    }

    // Prepare command input
    let input = ModifyLinkContentInput {
        content_id,
        new_book_id,
        old_book_id,
    };

    // Execute command
    let command = ModifyLinkContentCommand;
    match command.execute(&config, &pool, input).await {
        Ok(result) => {
            if let Some(old_id) = result.old_book_id {
                if old_id != result.new_book_id {
                    println!("✓ Link modificato con successo!");
                    println!("  Contenuto {} ora associato al libro {}", result.content_id, result.new_book_id);
                } else {
                    println!("✓ Contenuto già associato a questo libro");
                }
            } else {
                println!("✓ Contenuto associato con successo al libro {}", result.new_book_id);
            }
        }
        Err(e) => {
            println!("✗ Errore durante la modifica: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
