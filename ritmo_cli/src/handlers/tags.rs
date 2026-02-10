/// Tag management handlers
use ritmo_commands::entities::{
    CreateTagCommand, CreateTagInput, DeleteTagCommand, DeleteTagInput, ListTagsCommand,
    ListTagsInput, UpdateTagCommand, UpdateTagInput,
};
use ritmo_commands::Command;
use ritmo_config::AppSettings;
use std::path::PathBuf;

use crate::confirmation::{confirm_operation, ConfirmationConfig, ConfirmationResult, PreviewItem};

pub async fn handle_tags_list(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = ListTagsCommand;
    let input = ListTagsInput;
    let result = command.execute(&config, &pool, input).await?;

    // Format output
    match output {
        "json" => {
            let json_tags: Vec<serde_json::Value> = result
                .tags
                .iter()
                .map(|tag| {
                    serde_json::json!({
                        "id": tag.id,
                        "name": &tag.name,
                        "created_at": tag.created_at,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_tags)?);
        }
        "simple" => {
            for tag in &result.tags {
                println!("{}", tag.name);
            }
        }
        _ => {
            // table format
            println!("{:<6} {}", "ID", "Name");
            println!("{}", "-".repeat(50));
            for tag in &result.tags {
                println!("{:<6} {}", tag.id, tag.name);
            }
            println!("\nTotal: {} tags", result.total_count);
        }
    }

    Ok(())
}

/// Handle tags create command
pub async fn handle_tags_create(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    name: &str,
    description: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = CreateTagCommand;
    let input = CreateTagInput {
        name: name.to_string(),
        description: description.clone(),
    };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ {}", result.message);
    println!("   Tag ID: {}", result.tag_id);
    println!("   Name: {}", result.name);
    println!("   Created: {}", result.created_at);

    Ok(())
}

/// Handle tags update command
pub async fn handle_tags_update(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    id: &i64,
    name: &Option<String>,
    description: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = UpdateTagCommand;
    let input = UpdateTagInput {
        tag_id: *id,
        name: name.clone(),
        description: description.clone(),
    };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ {}", result.message);
    println!("   Tag ID: {}", result.tag_id);
    println!("   Name: {}", result.name);
    println!("   Updated: {}", result.updated_at);

    Ok(())
}

/// Handle tags delete command
pub async fn handle_tags_delete(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    id: &i64,
    yes: &bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // First, get the tag information for preview
    let tag_info = sqlx::query!("SELECT id, name FROM tags WHERE id = ?", id)
        .fetch_optional(&pool)
        .await?;

    let tag = match tag_info {
        Some(t) => t,
        None => {
            eprintln!("❌ Error: Tag with ID {} not found", id);
            return Ok(());
        }
    };

    // Show confirmation for delete operation
    let preview_items = vec![PreviewItem {
        id: tag.id,
        display_text: tag.name.clone(),
    }];

    let confirmation = confirm_operation(ConfirmationConfig {
        items: preview_items,
        operation: "delete",
        entity_type: "tag",
        force_yes: *yes,
        dry_run: false,
        warning: None,
    })?;

    if matches!(confirmation, ConfirmationResult::Declined) {
        println!("Operation cancelled");
        return Ok(());
    }

    // Execute command
    let command = DeleteTagCommand;
    let input = DeleteTagInput { tag_id: *id };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ Tag deleted successfully");
    println!("   Tag ID: {}", result.tag_id);
    println!("   Name: {}", result.name);
    println!("   Books affected: {}", result.books_affected);
    println!("   Deleted: {}", result.deleted_at);

    if let Some(warning) = result.warning {
        println!("\n⚠️  {}", warning);
    }

    Ok(())
}
