/// Series management handlers
use ritmo_commands::entities::{ListSeriesCommand, ListSeriesInput};
use ritmo_commands::Command;
use ritmo_config::AppSettings;
use std::path::PathBuf;

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
