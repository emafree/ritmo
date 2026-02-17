//! Command to create a new genre

use crate::{Command, CommandResult, CreateGenreResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to create a new genre
///
/// This command creates a new genre with optional description.
/// Validates that the genre name is unique and non-empty.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{CreateGenreCommand, CreateGenreInput};
/// use ritmo_commands::Command;
///
/// let command = CreateGenreCommand;
/// let input = CreateGenreInput {
///     name: "Science Fiction".to_string(),
///     description: Some("Science fiction and speculative fiction".to_string()),
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Genre created with ID: {}", result.genre_id);
/// ```
#[derive(Debug, Clone)]
pub struct CreateGenreCommand;

/// Input parameters for creating a genre
#[derive(Debug, Clone)]
pub struct CreateGenreInput {
    /// Genre name (required, must be unique)
    pub name: String,

    /// Optional description
    pub description: Option<String>,
}

#[async_trait]
impl Command for CreateGenreCommand {
    type Input = CreateGenreInput;
    type Output = CreateGenreResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate genre name is not empty
        if input.name.trim().is_empty() {
            return Err(crate::CommandError::Validation(
                "Genre name cannot be empty".to_string()
            ));
        }

        // Validate genre name length
        if input.name.len() > 100 {
            return Err(crate::CommandError::Validation(
                "Genre name too long (max 100 characters)".to_string()
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

        // Check if genre already exists
        let existing = sqlx::query!("SELECT id FROM genres WHERE name = ?", input.name)
            .fetch_optional(pool)
            .await?;

        if existing.is_some() {
            return Err(crate::CommandError::Validation(format!(
                "Genre '{}' already exists",
                input.name
            )));
        }

        // Insert new genre
        let result = sqlx::query!(
            "INSERT INTO genres (name, description) VALUES (?, ?) RETURNING id, created_at",
            input.name,
            input.description
        )
        .fetch_one(pool)
        .await?;

        let genre_id = result.id;
        let created_at = chrono::DateTime::from_timestamp(result.created_at, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        Ok(CreateGenreResult {
            genre_id,
            name: input.name.clone(),
            created_at,
            message: format!("Genre '{}' created successfully", input.name),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_empty_name() {
        let command = CreateGenreCommand;
        let input = CreateGenreInput {
            name: "".to_string(),
            description: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_name_too_long() {
        let command = CreateGenreCommand;
        let input = CreateGenreInput {
            name: "a".repeat(101),
            description: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_valid_input() {
        let command = CreateGenreCommand;
        let input = CreateGenreInput {
            name: "Science Fiction".to_string(),
            description: Some("Science fiction and speculative fiction".to_string()),
        };

        let result = command.validate(&input);
        assert!(result.is_ok());
    }
}
