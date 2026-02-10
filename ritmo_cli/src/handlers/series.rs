/// Series management handlers
use ritmo_config::AppSettings;
use std::path::PathBuf;

pub async fn handle_series_list(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Query all series
    let series_list = sqlx::query!("SELECT id, name, description, total_books, completed FROM series ORDER BY name")
        .fetch_all(&pool)
        .await?;

    let count = series_list.len();

    match output {
        "json" => {
            let json_series: Vec<serde_json::Value> = series_list
                .iter()
                .map(|series| {
                    serde_json::json!({
                        "id": series.id,
                        "name": &series.name,
                        "description": &series.description,
                        "total_books": series.total_books,
                        "completed": series.completed == 1,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_series)?);
        }
        "simple" => {
            for series in &series_list {
                println!("{}", series.name);
            }
        }
        _ => {
            // table format
            println!("{:<6} {:<40} {:<10} {}", "ID", "Name", "Completed", "Total Books");
            println!("{}", "-".repeat(80));
            for series in &series_list {
                let completed_str = if series.completed == 1 { "Yes" } else { "No" };
                let total_books = series.total_books.unwrap_or(0);
                println!("{:<6} {:<40} {:<10} {}", series.id, series.name, completed_str, total_books);
            }
            println!("\nTotal: {} series", count);
        }
    }

    Ok(())
}
