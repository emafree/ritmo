//! Command to update an existing genre

use crate::{Command, CommandResult, UpdateGenreResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to update an existing genre
///
/// This command updates the name and/or description of a genre.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{UpdateGenreCommand, UpdateGenreInput};
/// use ritmo_commands::Command;
///
/// let command = UpdateGenreCommand;
/// let input = UpdateGenreInput {
///     genre_id: 1,
///     name: Some("Sci-Fi".to_string()),
///     description: Some("Science Fiction genre".to_string()),
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Genre updated: {}", result.name);
/// ```
#[derive(Debug, Clone)]
pub struct UpdateGenreCommand;

/// Input parameters for updating a genre
#[derive(Debug, Clone)]
pub struct UpdateGenreInput {
    /// Genre ID to update
    pub genre_id: i64,

    /// New name (optional - if None, keep existing)
    pub name: Option<String>,

    /// New description (optional - if None, keep existing)
    pub description: Option<String>,
}

#[async_trait]
impl Command for UpdateGenreCommand {
    type Input = UpdateGenreInput;
    type Output = UpdateGenreResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate genre_id is positive
        if input.genre_id <= 0 {
            return Err(crate::CommandError::Validation(
                "Genre ID must be positive".to_string()
            ));
        }

        // Validate name if provided
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
        let existing = sqlx::query!("SELECT id, name, description FROM genres WHERE id = ?", input.genre_id)
            .fetch_optional(pool)
            .await?;

        let existing_genre = existing.ok_or_else(|| {
            crate::CommandError::NotFound(format!("Genre with ID {} not found", input.genre_id))
        })?;

        // Determine final values
        let final_name = input.name.unwrap_or(existing_genre.name);
        let final_description = if input.description.is_some() {
            input.description
        } else {
            existing_genre.description
        };

        // Check if new name conflicts with another genre
        if let Some(conflict) = sqlx::query!(
            "SELECT id FROM genres WHERE name = ? AND id != ?",
            final_name,
            input.genre_id
        )
        .fetch_optional(pool)
        .await?
        {
            return Err(crate::CommandError::Validation(format!(
                "Genre name '{}' already used by genre ID {}",
                final_name, conflict.id.unwrap_or(0)
            )));
        }

        // Update the genre
        let now = chrono::Utc::now().timestamp();
        sqlx::query!(
            "UPDATE genres SET name = ?, description = ?, updated_at = ? WHERE id = ?",
            final_name,
            final_description,
            now,
            input.genre_id
        )
        .execute(pool)
        .await?;

        let updated_at = chrono::DateTime::from_timestamp(now, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        Ok(UpdateGenreResult {
            genre_id: input.genre_id,
            name: final_name.clone(),
            updated_at,
            message: format!("Genre '{}' updated successfully", final_name),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_invalid_id() {
        let command = UpdateGenreCommand;
        let input = UpdateGenreInput {
            genre_id: 0,
            name: Some("Test".to_string()),
            description: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_empty_name() {
        let command = UpdateGenreCommand;
        let input = UpdateGenreInput {
            genre_id: 1,
            name: Some("".to_string()),
            description: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_valid_input() {
        let command = UpdateGenreCommand;
        let input = UpdateGenreInput {
            genre_id: 1,
            name: Some("Science Fiction".to_string()),
            description: Some("Updated description".to_string()),
        };

        let result = command.validate(&input);
        assert!(result.is_ok());
    }
}
