//! Command to update a role

use crate::{Command, CommandResult, UpdateRoleResult};
use async_trait::async_trait;
use ritmo_db::models::Role;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to update a role
///
/// This command updates an existing role's key.
/// Validates that the role exists and the new key is valid.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{UpdateRoleCommand, UpdateRoleInput};
/// use ritmo_commands::Command;
///
/// let command = UpdateRoleCommand;
/// let input = UpdateRoleInput {
///     role_id: 1,
///     key: Some("role.new_key".to_string()),
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Role updated with ID: {}", result.role_id);
/// ```
#[derive(Debug, Clone)]
pub struct UpdateRoleCommand;

/// Input parameters for updating a role
#[derive(Debug, Clone)]
pub struct UpdateRoleInput {
    /// Role ID to update
    pub role_id: i64,

    /// New role key (optional)
    pub key: Option<String>,
}

#[async_trait]
impl Command for UpdateRoleCommand {
    type Input = UpdateRoleInput;
    type Output = UpdateRoleResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate at least one field is being updated
        if input.key.is_none() {
            return Err(crate::CommandError::Validation(
                "No fields to update provided".to_string()
            ));
        }

        // Validate key if provided
        if let Some(ref key) = input.key {
            if key.trim().is_empty() {
                return Err(crate::CommandError::Validation(
                    "Role key cannot be empty".to_string()
                ));
            }
            if key.len() > 100 {
                return Err(crate::CommandError::Validation(
                    "Role key too long (max 100 characters)".to_string()
                ));
            }
        }

        Ok(())
    }

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Validate input
        self.validate(&input)?;

        // Check if role exists
        let existing = sqlx::query!("SELECT id, key FROM roles WHERE id = ?", input.role_id)
            .fetch_optional(pool)
            .await?;

        if existing.is_none() {
            return Err(crate::CommandError::NotFound(format!(
                "Role with ID {} not found",
                input.role_id
            )));
        }

        let existing_role = existing.unwrap();

        // Check if new key already exists (if updating key)
        if let Some(ref new_key) = input.key {
            if new_key != &existing_role.key {
                let duplicate = sqlx::query!("SELECT id FROM roles WHERE key = ?", new_key)
                    .fetch_optional(pool)
                    .await?;

                if duplicate.is_some() {
                    return Err(crate::CommandError::Validation(format!(
                        "Role key '{}' already exists",
                        new_key
                    )));
                }
            }
        }

        // Update role
        let updated_key = input.key.as_ref().unwrap_or(&existing_role.key);
        
        sqlx::query!(
            "UPDATE roles SET key = ? WHERE id = ?",
            updated_key,
            input.role_id
        )
        .execute(pool)
        .await?;

        // Get updated role
        let updated = sqlx::query!("SELECT id, key, created_at FROM roles WHERE id = ?", input.role_id)
            .fetch_one(pool)
            .await?;

        let role = Role {
            id: Some(updated.id),
            key: updated.key.clone(),
            created_at: updated.created_at,
        };

        let updated_at = chrono::Utc::now().to_rfc3339();

        Ok(UpdateRoleResult {
            role_id: input.role_id,
            key: updated.key.clone(),
            display_name: role.display_name(),
            updated_at,
            message: format!("Role '{}' updated successfully", updated.key),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_no_fields() {
        let command = UpdateRoleCommand;
        let input = UpdateRoleInput {
            role_id: 1,
            key: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_empty_key() {
        let command = UpdateRoleCommand;
        let input = UpdateRoleInput {
            role_id: 1,
            key: Some("".to_string()),
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_key_too_long() {
        let command = UpdateRoleCommand;
        let input = UpdateRoleInput {
            role_id: 1,
            key: Some("a".repeat(101)),
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_valid_input() {
        let command = UpdateRoleCommand;
        let input = UpdateRoleInput {
            role_id: 1,
            key: Some("role.narrator".to_string()),
        };

        let result = command.validate(&input);
        assert!(result.is_ok());
    }
}
