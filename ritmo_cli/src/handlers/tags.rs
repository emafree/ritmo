/// Tag management handlers
use ritmo_commands::entities::{ListTagsCommand, ListTagsInput};
use ritmo_commands::Command;
use ritmo_config::AppSettings;
use std::path::PathBuf;

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
