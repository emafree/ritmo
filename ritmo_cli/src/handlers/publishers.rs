/// Publisher management handlers
use ritmo_commands::entities::{
    CreatePublisherCommand, CreatePublisherInput, DeletePublisherCommand, DeletePublisherInput,
    ListPublishersCommand, ListPublishersInput, UpdatePublisherCommand, UpdatePublisherInput,
};
use ritmo_commands::Command;
use ritmo_config::AppSettings;
use std::path::PathBuf;

use crate::confirmation::{confirm_operation, ConfirmationConfig, ConfirmationResult, PreviewItem};

pub async fn handle_publishers_list(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = ListPublishersCommand;
    let input = ListPublishersInput;
    let result = command.execute(&config, &pool, input).await?;

    // Format output
    match output {
        "json" => {
            let json_publishers: Vec<serde_json::Value> = result
                .publishers
                .iter()
                .map(|pub_| {
                    serde_json::json!({
                        "id": pub_.id,
                        "name": &pub_.name,
                        "country": &pub_.country,
                        "website": &pub_.website,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_publishers)?);
        }
        "simple" => {
            for pub_ in &result.publishers {
                println!("{}", pub_.name);
            }
        }
        _ => {
            // table format
            println!("{:<6} {:<40} {:<20} {}", "ID", "Name", "Country", "Website");
            println!("{}", "-".repeat(100));
            for pub_ in &result.publishers {
                let country = pub_.country.as_deref().unwrap_or("");
                let website = pub_.website.as_deref().unwrap_or("");
                println!("{:<6} {:<40} {:<20} {}", pub_.id, pub_.name, country, website);
            }
            println!("\nTotal: {} publishers", result.total_count);
        }
    }

    Ok(())
}

/// Handle publishers create command
pub async fn handle_publishers_create(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    name: &str,
    country: &Option<String>,
    website: &Option<String>,
    notes: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = CreatePublisherCommand;
    let input = CreatePublisherInput {
        name: name.to_string(),
        country: country.clone(),
        website: website.clone(),
        notes: notes.clone(),
    };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ {}", result.message);
    println!("   Publisher ID: {}", result.publisher_id);
    println!("   Name: {}", result.name);
    println!("   Created: {}", result.created_at);

    Ok(())
}

/// Handle publishers update command
pub async fn handle_publishers_update(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    id: &i64,
    name: &Option<String>,
    country: &Option<String>,
    website: &Option<String>,
    notes: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = UpdatePublisherCommand;
    let input = UpdatePublisherInput {
        publisher_id: *id,
        name: name.clone(),
        country: country.clone(),
        website: website.clone(),
        notes: notes.clone(),
    };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ {}", result.message);
    println!("   Publisher ID: {}", result.publisher_id);
    println!("   Name: {}", result.name);
    println!("   Updated: {}", result.updated_at);

    Ok(())
}

/// Handle publishers delete command
pub async fn handle_publishers_delete(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    id: &i64,
    yes: &bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // First, get the publisher information for preview
    let publisher_info = sqlx::query!("SELECT id, name FROM publishers WHERE id = ?", id)
        .fetch_optional(&pool)
        .await?;

    let publisher = match publisher_info {
        Some(p) => p,
        None => {
            eprintln!("❌ Error: Publisher with ID {} not found", id);
            return Ok(());
        }
    };

    // Show confirmation for delete operation
    let preview_items = vec![PreviewItem {
        id: publisher.id,
        display_text: publisher.name.clone(),
    }];

    let confirmation = confirm_operation(ConfirmationConfig {
        items: preview_items,
        operation: "delete",
        entity_type: "publisher",
        force_yes: *yes,
        dry_run: false,
        warning: None,
    })?;

    if matches!(confirmation, ConfirmationResult::Declined) {
        println!("Operation cancelled");
        return Ok(());
    }

    // Execute command
    let command = DeletePublisherCommand;
    let input = DeletePublisherInput { publisher_id: *id };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ Publisher deleted successfully");
    println!("   Publisher ID: {}", result.publisher_id);
    println!("   Name: {}", result.name);
    println!("   Books affected: {}", result.books_affected);
    println!("   Deleted: {}", result.deleted_at);

    if let Some(warning) = result.warning {
        println!("\n⚠️  {}", warning);
    }

    Ok(())
}
