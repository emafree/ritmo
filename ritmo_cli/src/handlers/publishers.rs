/// Publisher management handlers
use ritmo_commands::entities::{ListPublishersCommand, ListPublishersInput};
use ritmo_commands::Command;
use ritmo_config::AppSettings;
use std::path::PathBuf;

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
