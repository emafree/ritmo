//! Command to create a new tag

use crate::{Command, CommandResult, CreateTagResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to create a new tag
///
/// This command creates a new tag with optional description.
/// Validates that the tag name is unique and non-empty.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{CreateTagCommand, CreateTagInput};
/// use ritmo_commands::Command;
///
/// let command = CreateTagCommand;
/// let input = CreateTagInput {
///     name: "science-fiction".to_string(),
///     description: Some("Science fiction books".to_string()),
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Tag created with ID: {}", result.tag_id);
/// ```
#[derive(Debug, Clone)]
pub struct CreateTagCommand;

/// Input parameters for creating a tag
#[derive(Debug, Clone)]
pub struct CreateTagInput {
    /// Tag name (required, must be unique)
    pub name: String,

    /// Optional description
    pub description: Option<String>,
}

#[async_trait]
impl Command for CreateTagCommand {
    type Input = CreateTagInput;
    type Output = CreateTagResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate tag name is not empty
        if input.name.trim().is_empty() {
            return Err(crate::CommandError::Validation(
                "Tag name cannot be empty".to_string()
            ));
        }

        // Validate tag name length
        if input.name.len() > 100 {
            return Err(crate::CommandError::Validation(
                "Tag name too long (max 100 characters)".to_string()
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

        // Check if tag already exists
        let existing = sqlx::query!("SELECT id FROM tags WHERE name = ?", input.name)
            .fetch_optional(pool)
            .await?;

        if existing.is_some() {
            return Err(crate::CommandError::Validation(format!(
                "Tag '{}' already exists",
                input.name
            )));
        }

        // Insert new tag
        let result = sqlx::query!(
            "INSERT INTO tags (name, description) VALUES (?, ?) RETURNING id, created_at",
            input.name,
            input.description
        )
        .fetch_one(pool)
        .await?;

        let tag_id = result.id;
        let created_at = chrono::DateTime::from_timestamp(result.created_at, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        Ok(CreateTagResult {
            tag_id,
            name: input.name.clone(),
            created_at,
            message: format!("Tag '{}' created successfully", input.name),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_empty_name() {
        let command = CreateTagCommand;
        let input = CreateTagInput {
            name: "".to_string(),
            description: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_name_too_long() {
        let command = CreateTagCommand;
        let input = CreateTagInput {
            name: "a".repeat(101),
            description: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_valid_input() {
        let command = CreateTagCommand;
        let input = CreateTagInput {
            name: "science-fiction".to_string(),
            description: Some("Science fiction books".to_string()),
        };

        let result = command.validate(&input);
        assert!(result.is_ok());
    }
}
