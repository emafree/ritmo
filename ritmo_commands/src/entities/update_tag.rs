//! Command to update a tag

use crate::{Command, CommandResult, UpdateTagResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to update an existing tag
///
/// This command updates a tag's name and/or description.
/// At least one field must be provided for update.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{UpdateTagCommand, UpdateTagInput};
/// use ritmo_commands::Command;
///
/// let command = UpdateTagCommand;
/// let input = UpdateTagInput {
///     tag_id: 1,
///     name: Some("sci-fi".to_string()),
///     description: Some("Science fiction genre".to_string()),
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Tag updated: {}", result.name);
/// ```
#[derive(Debug, Clone)]
pub struct UpdateTagCommand;

/// Input parameters for updating a tag
#[derive(Debug, Clone)]
pub struct UpdateTagInput {
    /// Tag ID to update (required)
    pub tag_id: i64,

    /// New tag name (optional)
    pub name: Option<String>,

    /// New description (optional)
    pub description: Option<String>,
}

#[async_trait]
impl Command for UpdateTagCommand {
    type Input = UpdateTagInput;
    type Output = UpdateTagResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate at least one field is being updated
        if input.name.is_none() && input.description.is_none() {
            return Err(crate::CommandError::Validation(
                "At least one field must be provided for update".to_string()
            ));
        }

        // Validate tag name if provided
        if let Some(ref name) = input.name {
            if name.trim().is_empty() {
                return Err(crate::CommandError::Validation(
                    "Tag name cannot be empty".to_string()
                ));
            }

            if name.len() > 100 {
                return Err(crate::CommandError::Validation(
                    "Tag name too long (max 100 characters)".to_string()
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

        // Check if tag exists
        let existing = sqlx::query!("SELECT id, name FROM tags WHERE id = ?", input.tag_id)
            .fetch_optional(pool)
            .await?;

        if existing.is_none() {
            return Err(crate::CommandError::NotFound(format!(
                "Tag with ID {} not found",
                input.tag_id
            )));
        }

        // If updating name, check for duplicates
        if let Some(ref new_name) = input.name {
            let duplicate = sqlx::query!(
                "SELECT id FROM tags WHERE name = ? AND id != ?",
                new_name,
                input.tag_id
            )
            .fetch_optional(pool)
            .await?;

            if duplicate.is_some() {
                return Err(crate::CommandError::Validation(format!(
                    "Tag '{}' already exists",
                    new_name
                )));
            }
        }

        // Build update query dynamically
        let mut update_fields = Vec::new();
        let mut values: Vec<String> = Vec::new();

        if let Some(ref name) = input.name {
            update_fields.push("name = ?");
            values.push(name.clone());
        }

        if let Some(ref description) = input.description {
            update_fields.push("description = ?");
            values.push(description.clone());
        }

        let query = format!(
            "UPDATE tags SET {} WHERE id = ?",
            update_fields.join(", ")
        );

        // Execute update
        let mut query_builder = sqlx::query(&query);
        for value in &values {
            query_builder = query_builder.bind(value);
        }
        query_builder = query_builder.bind(input.tag_id);

        query_builder.execute(pool).await?;

        // Get updated tag
        let updated = sqlx::query!("SELECT name FROM tags WHERE id = ?", input.tag_id)
            .fetch_one(pool)
            .await?;

        let updated_at = chrono::Utc::now().to_rfc3339();

        Ok(UpdateTagResult {
            tag_id: input.tag_id,
            name: updated.name,
            updated_at,
            message: format!("Tag updated successfully"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_no_fields() {
        let command = UpdateTagCommand;
        let input = UpdateTagInput {
            tag_id: 1,
            name: None,
            description: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_empty_name() {
        let command = UpdateTagCommand;
        let input = UpdateTagInput {
            tag_id: 1,
            name: Some("".to_string()),
            description: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_valid_input() {
        let command = UpdateTagCommand;
        let input = UpdateTagInput {
            tag_id: 1,
            name: Some("sci-fi".to_string()),
            description: None,
        };

        let result = command.validate(&input);
        assert!(result.is_ok());
    }
}
