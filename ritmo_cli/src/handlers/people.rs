/// People management handlers
use ritmo_config::AppSettings;
use std::path::PathBuf;

pub async fn handle_people_list(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Query all people
    let people = sqlx::query!("SELECT id, name, display_name, verified, confidence FROM people ORDER BY name")
        .fetch_all(&pool)
        .await?;

    let count = people.len();

    match output {
        "json" => {
            let json_people: Vec<serde_json::Value> = people
                .iter()
                .map(|person| {
                    serde_json::json!({
                        "id": person.id,
                        "name": &person.name,
                        "display_name": &person.display_name,
                        "verified": person.verified == 1,
                        "confidence": person.confidence,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_people)?);
        }
        "simple" => {
            for person in &people {
                let display = person.display_name.as_ref().unwrap_or(&person.name);
                println!("{}", display);
            }
        }
        _ => {
            // table format
            println!("{:<6} {:<40} {:<40} {:<10} {}", "ID", "Name", "Display Name", "Verified", "Confidence");
            println!("{}", "-".repeat(110));
            for person in &people {
                let verified_str = if person.verified == 1 { "Yes" } else { "No" };
                let display_name = person.display_name.as_deref().unwrap_or("-");
                println!(
                    "{:<6} {:<40} {:<40} {:<10} {:.2}",
                    person.id, person.name, display_name, verified_str, person.confidence
                );
            }
            println!("\nTotal: {} people", count);
        }
    }

    Ok(())
}
