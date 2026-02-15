/// Series management handlers
use ritmo_commands::entities::{
    CreateSeriesCommand, CreateSeriesInput, DeleteSeriesCommand, DeleteSeriesInput,
    ListSeriesCommand, ListSeriesInput, UpdateSeriesCommand, UpdateSeriesInput,
};
use ritmo_commands::Command;
use ritmo_config::AppSettings;
use std::path::PathBuf;

use crate::confirmation::{confirm_operation, ConfirmationConfig, ConfirmationResult, PreviewItem};

pub async fn handle_series_list(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = ListSeriesCommand;
    let input = ListSeriesInput;
    let result = command.execute(&config, &pool, input).await?;

    // Format output
    match output {
        "json" => {
            let json_series: Vec<serde_json::Value> = result
                .series
                .iter()
                .map(|series| {
                    serde_json::json!({
                        "id": series.id,
                        "name": &series.name,
                        "description": &series.description,
                        "total_books": series.total_books,
                        "completed": series.completed,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_series)?);
        }
        "simple" => {
            for series in &result.series {
                println!("{}", series.name);
            }
        }
        _ => {
            // table format
            println!("{:<6} {:<40} {:<10} {}", "ID", "Name", "Completed", "Total Books");
            println!("{}", "-".repeat(80));
            for series in &result.series {
                let completed_str = if series.completed { "Yes" } else { "No" };
                let total_books = series.total_books.unwrap_or(0);
                println!("{:<6} {:<40} {:<10} {}", series.id, series.name, completed_str, total_books);
            }
            println!("\nTotal: {} series", result.total_count);
        }
    }

    Ok(())
}

/// Handle series create command
pub async fn handle_series_create(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    name: &str,
    description: &Option<String>,
    total_books: &Option<i64>,
    completed: &bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = CreateSeriesCommand;
    let input = CreateSeriesInput {
        name: name.to_string(),
        description: description.clone(),
        total_books: *total_books,
        completed: *completed,
    };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ {}", result.message);
    println!("   Series ID: {}", result.series_id);
    println!("   Name: {}", result.name);
    println!("   Created: {}", result.created_at);

    Ok(())
}

/// Handle series update command
pub async fn handle_series_update(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    id: &i64,
    name: &Option<String>,
    description: &Option<String>,
    total_books: &Option<i64>,
    completed: &Option<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = UpdateSeriesCommand;
    let input = UpdateSeriesInput {
        series_id: *id,
        name: name.clone(),
        description: description.clone(),
        total_books: *total_books,
        completed: *completed,
    };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ {}", result.message);
    println!("   Series ID: {}", result.series_id);
    println!("   Name: {}", result.name);
    println!("   Updated: {}", result.updated_at);

    Ok(())
}

/// Handle series delete command
pub async fn handle_series_delete(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    id: &i64,
    yes: &bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // First, get the series information for preview
    let series_info = sqlx::query!("SELECT id, name FROM series WHERE id = ?", id)
        .fetch_optional(&pool)
        .await?;

    let series = match series_info {
        Some(s) => s,
        None => {
            eprintln!("❌ Error: Series with ID {} not found", id);
            return Ok(());
        }
    };

    // Show confirmation for delete operation
    let preview_items = vec![PreviewItem {
        id: series.id,
        display_text: series.name.clone(),
    }];

    let confirmation = confirm_operation(ConfirmationConfig {
        items: preview_items,
        operation: "delete",
        entity_type: "series",
        force_yes: *yes,
        dry_run: false,
        warning: None,
    })?;

    if matches!(confirmation, ConfirmationResult::Declined) {
        println!("Operation cancelled");
        return Ok(());
    }

    // Execute command
    let command = DeleteSeriesCommand;
    let input = DeleteSeriesInput { series_id: *id };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ Series deleted successfully");
    println!("   Series ID: {}", result.series_id);
    println!("   Name: {}", result.name);
    println!("   Books affected: {}", result.books_affected);
    println!("   Deleted: {}", result.deleted_at);

    if let Some(warning) = result.warning {
        println!("\n⚠️  {}", warning);
    }

    Ok(())
}
