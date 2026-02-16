/// Role management handlers
use ritmo_commands::entities::{
    CreateRoleCommand, CreateRoleInput, DeleteRoleCommand, DeleteRoleInput, ListRolesCommand,
    ListRolesInput, UpdateRoleCommand, UpdateRoleInput,
};
use ritmo_commands::Command;
use ritmo_config::AppSettings;
use std::path::PathBuf;

use crate::confirmation::{confirm_operation, ConfirmationConfig, ConfirmationResult, PreviewItem};

pub async fn handle_roles_list(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = ListRolesCommand;
    let input = ListRolesInput;
    let result = command.execute(&config, &pool, input).await?;

    // Format output
    match output {
        "json" => {
            let json_roles: Vec<serde_json::Value> = result
                .roles
                .iter()
                .map(|role| {
                    serde_json::json!({
                        "id": role.id,
                        "key": &role.key,
                        "display_name": &role.display_name,
                        "created_at": role.created_at,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_roles)?);
        }
        "simple" => {
            for role in &result.roles {
                println!("{}", role.display_name);
            }
        }
        _ => {
            // table format
            println!("{:<6} {:<30} {}", "ID", "Key", "Display Name");
            println!("{}", "-".repeat(70));
            for role in &result.roles {
                println!("{:<6} {:<30} {}", role.id, role.key, role.display_name);
            }
            println!("\nTotal: {} roles", result.total_count);
        }
    }

    Ok(())
}

/// Handle roles create command
pub async fn handle_roles_create(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = CreateRoleCommand;
    let input = CreateRoleInput {
        key: key.to_string(),
    };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ {}", result.message);
    println!("   Role ID: {}", result.role_id);
    println!("   Key: {}", result.key);
    println!("   Display name: {}", result.display_name);
    println!("   Created: {}", result.created_at);

    Ok(())
}

/// Handle roles update command
pub async fn handle_roles_update(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    id: &i64,
    key: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // Execute command
    let command = UpdateRoleCommand;
    let input = UpdateRoleInput {
        role_id: *id,
        key: key.clone(),
    };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ {}", result.message);
    println!("   Role ID: {}", result.role_id);
    println!("   Key: {}", result.key);
    println!("   Display name: {}", result.display_name);
    println!("   Updated: {}", result.updated_at);

    Ok(())
}

/// Handle roles delete command
pub async fn handle_roles_delete(
    library_path: &Option<PathBuf>,
    app_settings: &AppSettings,
    id: &i64,
    yes: &bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, pool) = super::common::get_library_and_pool(library_path, app_settings).await?;

    // First, get the role information for preview
    let role_info = sqlx::query!("SELECT id, key FROM roles WHERE id = ?", id)
        .fetch_optional(&pool)
        .await?;

    let role = match role_info {
        Some(r) => r,
        None => {
            eprintln!("❌ Error: Role with ID {} not found", id);
            return Ok(());
        }
    };

    // Show confirmation for delete operation
    let preview_items = vec![PreviewItem {
        id: role.id,
        display_text: role.key.clone(),
    }];

    let confirmation = confirm_operation(ConfirmationConfig {
        items: preview_items,
        operation: "delete",
        entity_type: "role",
        force_yes: *yes,
        dry_run: false,
        warning: None,
    })?;

    if matches!(confirmation, ConfirmationResult::Declined) {
        println!("Operation cancelled");
        return Ok(());
    }

    // Execute command
    let command = DeleteRoleCommand;
    let input = DeleteRoleInput { role_id: *id };
    let result = command.execute(&config, &pool, input).await?;

    // Display success message
    println!("\n✅ Role deleted successfully");
    println!("   Role ID: {}", result.role_id);
    println!("   Key: {}", result.key);
    println!("   Display name: {}", result.display_name);
    println!("   Books affected: {}", result.books_affected);
    println!("   Contents affected: {}", result.contents_affected);
    println!("   Deleted: {}", result.deleted_at);

    if let Some(warning) = result.warning {
        println!("\n⚠️  {}", warning);
    }

    Ok(())
}
