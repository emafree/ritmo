/// Tag management handlers
use ritmo_config::AppSettings;
use std::path::PathBuf;

pub async fn handle_tags_list(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Query all tags
    let tags = sqlx::query!("SELECT id, name, created_at FROM tags ORDER BY name")
        .fetch_all(&pool)
        .await?;

    let count = tags.len();

    match output {
        "json" => {
            let json_tags: Vec<serde_json::Value> = tags
                .iter()
                .map(|tag| {
                    serde_json::json!({
                        "id": tag.id.unwrap_or(0),
                        "name": &tag.name,
                        "created_at": tag.created_at,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_tags)?);
        }
        "simple" => {
            for tag in &tags {
                println!("{}", tag.name);
            }
        }
        _ => {
            // table format
            println!("{:<6} {}", "ID", "Name");
            println!("{}", "-".repeat(50));
            for tag in &tags {
                println!("{:<6} {}", tag.id.unwrap_or(0), tag.name);
            }
            println!("\nTotal: {} tags", count);
        }
    }

    Ok(())
}
