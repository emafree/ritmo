/// People management handlers
use ritmo_commands::entities::{
    CreatePersonCommand, CreatePersonInput, DeletePersonCommand, DeletePersonInput,
    ListPeopleCommand, ListPeopleInput, UpdatePersonCommand, UpdatePersonInput,
};
use ritmo_commands::Command;
use ritmo_config::AppSettings;
use std::path::PathBuf;

use crate::confirmation::{confirm_operation, ConfirmationConfig, ConfirmationResult, PreviewItem};

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

/// Handle people create command
pub async fn handle_people_create(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    name: &str,
    display_name: &Option<String>,
    given_name: &Option<String>,
    surname: &Option<String>,
    nationality: &Option<String>,
    biography: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = CreatePersonCommand;
    let input = CreatePersonInput {
        name: name.to_string(),
        display_name: display_name.clone(),
        given_name: given_name.clone(),
        surname: surname.clone(),
        middle_names: None,
        title: None,
        suffix: None,
        nationality: nationality.clone(),
        birth_date: None,
        death_date: None,
        biography: biography.clone(),
    };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ {}", result.message);
    println!("   Person ID: {}", result.person_id);
    println!("   Name: {}", result.name);
    if let Some(display) = &result.display_name {
        println!("   Display Name: {}", display);
    }
    println!("   Created: {}", result.created_at);

    Ok(())
}

/// Handle people update command
pub async fn handle_people_update(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    id: &i64,
    name: &Option<String>,
    display_name: &Option<String>,
    given_name: &Option<String>,
    surname: &Option<String>,
    nationality: &Option<String>,
    biography: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = UpdatePersonCommand;
    let input = UpdatePersonInput {
        person_id: *id,
        name: name.clone(),
        display_name: display_name.clone(),
        given_name: given_name.clone(),
        surname: surname.clone(),
        middle_names: None,
        title: None,
        suffix: None,
        nationality: nationality.clone(),
        birth_date: None,
        death_date: None,
        biography: biography.clone(),
    };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ {}", result.message);
    println!("   Person ID: {}", result.person_id);
    println!("   Name: {}", result.name);
    if let Some(display) = &result.display_name {
        println!("   Display Name: {}", display);
    }
    println!("   Updated: {}", result.updated_at);

    Ok(())
}

/// Handle people delete command
pub async fn handle_people_delete(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    id: &i64,
    yes: &bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // First, get the person information for preview
    let person_info = sqlx::query!(
        "SELECT id, name, display_name FROM people WHERE id = ?",
        id
    )
    .fetch_optional(&pool)
    .await?;

    let person = match person_info {
        Some(p) => p,
        None => {
            eprintln!("❌ Error: Person with ID {} not found", id);
            return Ok(());
        }
    };

    let display_text = person
        .display_name
        .unwrap_or_else(|| person.name.clone());

    // Show confirmation for delete operation
    let preview_items = vec![PreviewItem {
        id: person.id,
        display_text,
    }];

    let confirmation = confirm_operation(ConfirmationConfig {
        items: preview_items,
        operation: "delete",
        entity_type: "person",
        force_yes: *yes,
        dry_run: false,
        warning: None,
    })?;

    if matches!(confirmation, ConfirmationResult::Declined) {
        println!("Operation cancelled");
        return Ok(());
    }

    // Execute command
    let command = DeletePersonCommand;
    let input = DeletePersonInput { person_id: *id };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ Person deleted successfully");
    println!("   Person ID: {}", result.person_id);
    println!("   Name: {}", result.name);
    println!("   Books affected: {}", result.books_affected);
    println!("   Deleted: {}", result.deleted_at);

    if let Some(warning) = result.warning {
        println!("\n⚠️  {}", warning);
    }

    Ok(())
}
