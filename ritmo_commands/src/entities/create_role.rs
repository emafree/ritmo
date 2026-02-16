//! Command to create a new role

use crate::{Command, CommandResult, CreateRoleResult};
use async_trait::async_trait;
use ritmo_db::models::Role;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to create a new role
///
/// This command creates a new role with a key for i18n support.
/// Validates that the role key is unique and non-empty.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{CreateRoleCommand, CreateRoleInput};
/// use ritmo_commands::Command;
///
/// let command = CreateRoleCommand;
/// let input = CreateRoleInput {
///     key: "role.narrator".to_string(),
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Role created with ID: {}", result.role_id);
/// ```
#[derive(Debug, Clone)]
pub struct CreateRoleCommand;

/// Input parameters for creating a role
#[derive(Debug, Clone)]
pub struct CreateRoleInput {
    /// Role key (required, must be unique, e.g., "role.author")
    pub key: String,
}

#[async_trait]
impl Command for CreateRoleCommand {
    type Input = CreateRoleInput;
    type Output = CreateRoleResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate role key is not empty
        if input.key.trim().is_empty() {
            return Err(crate::CommandError::Validation(
                "Role key cannot be empty".to_string()
            ));
        }

        // Validate role key length
        if input.key.len() > 100 {
            return Err(crate::CommandError::Validation(
                "Role key too long (max 100 characters)".to_string()
            ));
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

        // Check if role already exists
        let existing = sqlx::query!("SELECT id FROM roles WHERE key = ?", input.key)
            .fetch_optional(pool)
            .await?;

        if existing.is_some() {
            return Err(crate::CommandError::Validation(format!(
                "Role '{}' already exists",
                input.key
            )));
        }

        // Insert new role
        let result = sqlx::query!(
            "INSERT INTO roles (key, created_at) VALUES (?, strftime('%s', 'now')) RETURNING id, created_at",
            input.key
        )
        .fetch_one(pool)
        .await?;

        let role_id = result.id;
        let created_at = chrono::DateTime::from_timestamp(result.created_at, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        // Get display name
        let role = Role {
            id: Some(role_id),
            key: input.key.clone(),
            created_at: result.created_at,
        };

        Ok(CreateRoleResult {
            role_id,
            key: input.key.clone(),
            display_name: role.display_name(),
            created_at,
            message: format!("Role '{}' created successfully", input.key),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_empty_key() {
        let command = CreateRoleCommand;
        let input = CreateRoleInput {
            key: "".to_string(),
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_key_too_long() {
        let command = CreateRoleCommand;
        let input = CreateRoleInput {
            key: "a".repeat(101),
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_valid_input() {
        let command = CreateRoleCommand;
        let input = CreateRoleInput {
            key: "role.narrator".to_string(),
        };

        let result = command.validate(&input);
        assert!(result.is_ok());
    }
}
