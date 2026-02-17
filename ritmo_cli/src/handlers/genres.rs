/// Genre management handlers
use ritmo_commands::entities::{
    CreateGenreCommand, CreateGenreInput, DeleteGenreCommand, DeleteGenreInput,
    ListGenresCommand, ListGenresInput, UpdateGenreCommand, UpdateGenreInput,
};
use ritmo_commands::Command;
use ritmo_config::AppSettings;
use std::path::PathBuf;

use crate::confirmation::{confirm_operation, ConfirmationConfig, ConfirmationResult, PreviewItem};

pub async fn handle_genres_list(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = ListGenresCommand;
    let input = ListGenresInput;
    let result = command.execute(&config, &pool, input).await?;

    // Format output
    match output {
        "json" => {
            let json_genres: Vec<serde_json::Value> = result
                .genres
                .iter()
                .map(|genre| {
                    serde_json::json!({
                        "id": genre.id,
                        "name": &genre.name,
                        "description": &genre.description,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_genres)?);
        }
        "simple" => {
            for genre in &result.genres {
                println!("{}", genre.name);
            }
        }
        _ => {
            // table format
            println!("{:<6} {:<40} {}", "ID", "Name", "Description");
            println!("{}", "-".repeat(80));
            for genre in &result.genres {
                let desc = genre.description.as_deref().unwrap_or("");
                println!("{:<6} {:<40} {}", genre.id, genre.name, desc);
            }
            println!("\nTotal: {} genres", result.total_count);
        }
    }

    Ok(())
}

/// Handle genre create command
pub async fn handle_genres_create(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    name: &str,
    description: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = CreateGenreCommand;
    let input = CreateGenreInput {
        name: name.to_string(),
        description: description.clone(),
    };

    let result = command.execute(&config, &pool, input).await?;

    println!("✓ {}", result.message);
    println!("  Genre ID: {}", result.genre_id);
    println!("  Name: {}", result.name);
    if let Some(ref desc) = description {
        println!("  Description: {}", desc);
    }

    Ok(())
}

/// Handle genre update command
pub async fn handle_genres_update(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    id: i64,
    name: &Option<String>,
    description: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = UpdateGenreCommand;
    let input = UpdateGenreInput {
        id,
        name: name.clone(),
        description: description.clone(),
    };

    let result = command.execute(&config, &pool, input).await?;

    println!("✓ {}", result.message);
    println!("  Genre ID: {}", result.genre_id);
    println!("  Name: {}", result.name);

    Ok(())
}

/// Handle genre delete command
pub async fn handle_genres_delete(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    id: i64,
    yes: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Get genre details for confirmation
    let genre = sqlx::query!("SELECT id, name FROM genres WHERE id = ?", id)
        .fetch_optional(&pool)
        .await?;

    let genre = genre.ok_or_else(|| format!("Genre with ID {} not found", id))?;

    // Confirm deletion if not using --yes flag
    if !yes {
        let preview_items = vec![PreviewItem {
            id: genre.id,
            display_text: genre.name.clone(),
        }];

        let conf_config = ConfirmationConfig {
            items: preview_items,
            operation: "delete",
            entity_type: "genre",
            force_yes: yes,
            dry_run: false,
            warning: Some("This will permanently delete the genre"),
        };

        match confirm_operation(conf_config)? {
            ConfirmationResult::Confirmed | ConfirmationResult::Skip => {}
            ConfirmationResult::Declined => {
                println!("Operation cancelled");
                return Ok(());
            }
        }
    }

    // Execute command
    let command = DeleteGenreCommand;
    let input = DeleteGenreInput { id };

    let result = command.execute(&config, &pool, input).await?;

    println!("✓ {}", result.message);
    println!("  Genre ID: {}", result.genre_id);
    println!("  Name: {}", result.name);

    Ok(())
}
