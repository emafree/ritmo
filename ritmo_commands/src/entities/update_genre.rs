//! Command to update a genre

use crate::{Command, CommandResult, UpdateGenreResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to update an existing genre
///
/// This command updates a genre's name and/or description.
/// Validates that the genre exists and the new name is unique.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{UpdateGenreCommand, UpdateGenreInput};
/// use ritmo_commands::Command;
///
/// let command = UpdateGenreCommand;
/// let input = UpdateGenreInput {
///     id: 1,
///     name: Some("Sci-Fi".to_string()),
///     description: Some("Updated description".to_string()),
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Genre updated: {}", result.message);
/// ```
#[derive(Debug, Clone)]
pub struct UpdateGenreCommand;

/// Input parameters for updating a genre
#[derive(Debug, Clone)]
pub struct UpdateGenreInput {
    /// Genre ID (required)
    pub id: i64,

    /// New genre name (optional)
    pub name: Option<String>,

    /// New description (optional)
    pub description: Option<String>,
}

#[async_trait]
impl Command for UpdateGenreCommand {
    type Input = UpdateGenreInput;
    type Output = UpdateGenreResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate at least one field is being updated
        if input.name.is_none() && input.description.is_none() {
            return Err(crate::CommandError::Validation(
                "At least one field must be provided for update".to_string()
            ));
        }

        // Validate genre name if provided
        if let Some(ref name) = input.name {
            if name.trim().is_empty() {
                return Err(crate::CommandError::Validation(
                    "Genre name cannot be empty".to_string()
                ));
            }

            if name.len() > 100 {
                return Err(crate::CommandError::Validation(
                    "Genre name too long (max 100 characters)".to_string()
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

        // Check if genre exists
        let existing = sqlx::query!("SELECT id, name, description FROM genres WHERE id = ?", input.id)
            .fetch_optional(pool)
            .await?;

        let existing = existing.ok_or_else(|| {
            crate::CommandError::NotFound(format!("Genre with ID {} not found", input.id))
        })?;

        // Determine final values (use new if provided, otherwise keep existing)
        let final_name = input.name.clone().unwrap_or(existing.name);
        let final_description = input.description.clone().or(existing.description);

        // Check if new name conflicts with another genre
        if let Some(ref new_name) = input.name {
            let conflict = sqlx::query!(
                "SELECT id FROM genres WHERE name = ? AND id != ?",
                new_name,
                input.id
            )
            .fetch_optional(pool)
            .await?;

            if conflict.is_some() {
                return Err(crate::CommandError::Validation(format!(
                    "Genre name '{}' already exists",
                    new_name
                )));
            }
        }

        // Update the genre
        let now = chrono::Utc::now().timestamp();
        sqlx::query!(
            "UPDATE genres SET name = ?, description = ?, updated_at = ? WHERE id = ?",
            final_name,
            final_description,
            now,
            input.id
        )
        .execute(pool)
        .await?;

        Ok(UpdateGenreResult {
            genre_id: input.id,
            name: final_name,
            message: format!("Genre with ID {} updated successfully", input.id),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_no_fields() {
        let command = UpdateGenreCommand;
        let input = UpdateGenreInput {
            id: 1,
            name: None,
            description: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_empty_name() {
        let command = UpdateGenreCommand;
        let input = UpdateGenreInput {
            id: 1,
            name: Some("".to_string()),
            description: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_name_too_long() {
        let command = UpdateGenreCommand;
        let input = UpdateGenreInput {
            id: 1,
            name: Some("a".repeat(101)),
            description: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_valid_input() {
        let command = UpdateGenreCommand;
        let input = UpdateGenreInput {
            id: 1,
            name: Some("Sci-Fi".to_string()),
            description: Some("Updated".to_string()),
        };

        let result = command.validate(&input);
        assert!(result.is_ok());
    }
}
