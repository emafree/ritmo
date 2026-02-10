/// People management handlers
use ritmo_commands::entities::{ListPeopleCommand, ListPeopleInput};
use ritmo_commands::Command;
use ritmo_config::AppSettings;
use std::path::PathBuf;

pub async fn handle_people_list(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = ListPeopleCommand;
    let input = ListPeopleInput;
    let result = command.execute(&config, &pool, input).await?;

    // Format output
    match output {
        "json" => {
            let json_people: Vec<serde_json::Value> = result
                .people
                .iter()
                .map(|person| {
                    serde_json::json!({
                        "id": person.id,
                        "name": &person.name,
                        "display_name": &person.display_name,
                        "verified": person.verified,
                        "confidence": person.confidence,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_people)?);
        }
        "simple" => {
            for person in &result.people {
                let display = person.display_name.as_ref().unwrap_or(&person.name);
                println!("{}", display);
            }
        }
        _ => {
            // table format
            println!("{:<6} {:<40} {:<40} {:<10} {}", "ID", "Name", "Display Name", "Verified", "Confidence");
            println!("{}", "-".repeat(110));
            for person in &result.people {
                let verified_str = if person.verified { "Yes" } else { "No" };
                let display_name = person.display_name.as_deref().unwrap_or("-");
                println!(
                    "{:<6} {:<40} {:<40} {:<10} {:.2}",
                    person.id, person.name, display_name, verified_str, person.confidence
                );
            }
            println!("\nTotal: {} people", result.total_count);
        }
    }

    Ok(())
}
