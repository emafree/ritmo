//! Command to create a new series

use crate::{Command, CommandResult, CreateSeriesResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to create a new series
///
/// This command creates a new series with optional metadata.
/// Validates that the series name is unique and non-empty.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{CreateSeriesCommand, CreateSeriesInput};
/// use ritmo_commands::Command;
///
/// let command = CreateSeriesCommand;
/// let input = CreateSeriesInput {
///     name: "Harry Potter".to_string(),
///     description: Some("Fantasy series about a young wizard".to_string()),
///     total_books: Some(7),
///     completed: true,
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Series created with ID: {}", result.series_id);
/// ```
#[derive(Debug, Clone)]
pub struct CreateSeriesCommand;

/// Input parameters for creating a series
#[derive(Debug, Clone)]
pub struct CreateSeriesInput {
    /// Series name (required, must be unique)
    pub name: String,

    /// Description (optional)
    pub description: Option<String>,

    /// Total number of books in series (optional)
    pub total_books: Option<i64>,

    /// Whether the series is completed (default: false)
    pub completed: bool,
}

#[async_trait]
impl Command for CreateSeriesCommand {
    type Input = CreateSeriesInput;
    type Output = CreateSeriesResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate series name is not empty
        if input.name.trim().is_empty() {
            return Err(crate::CommandError::Validation(
                "Series name cannot be empty".to_string()
            ));
        }

        // Validate series name length
        if input.name.len() > 200 {
            return Err(crate::CommandError::Validation(
                "Series name too long (max 200 characters)".to_string()
            ));
        }

        // Validate total_books if provided
        if let Some(total) = input.total_books {
            if total < 1 {
                return Err(crate::CommandError::Validation(
                    "Total books must be at least 1".to_string()
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

        // Check if series already exists
        let existing = sqlx::query!("SELECT id FROM series WHERE name = ?", input.name)
            .fetch_optional(pool)
            .await?;

        if existing.is_some() {
            return Err(crate::CommandError::Validation(format!(
                "Series '{}' already exists",
                input.name
            )));
        }

        // Convert bool to i64 for SQLite
        let completed_int: i64 = if input.completed { 1 } else { 0 };

        // Insert new series
        let result = sqlx::query!(
            "INSERT INTO series (name, description, total_books, completed) VALUES (?, ?, ?, ?) RETURNING id, created_at",
            input.name,
            input.description,
            input.total_books,
            completed_int
        )
        .fetch_one(pool)
        .await?;

        let series_id = result.id;
        let created_at = chrono::DateTime::from_timestamp(result.created_at, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        Ok(CreateSeriesResult {
            series_id,
            name: input.name.clone(),
            created_at,
            message: format!("Series '{}' created successfully", input.name),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_empty_name() {
        let command = CreateSeriesCommand;
        let input = CreateSeriesInput {
            name: "".to_string(),
            description: None,
            total_books: None,
            completed: false,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_name_too_long() {
        let command = CreateSeriesCommand;
        let input = CreateSeriesInput {
            name: "a".repeat(201),
            description: None,
            total_books: None,
            completed: false,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_invalid_total_books() {
        let command = CreateSeriesCommand;
        let input = CreateSeriesInput {
            name: "Test Series".to_string(),
            description: None,
            total_books: Some(0),
            completed: false,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_valid_input() {
        let command = CreateSeriesCommand;
        let input = CreateSeriesInput {
            name: "Harry Potter".to_string(),
            description: Some("Fantasy series".to_string()),
            total_books: Some(7),
            completed: true,
        };

        let result = command.validate(&input);
        assert!(result.is_ok());
    }
}
