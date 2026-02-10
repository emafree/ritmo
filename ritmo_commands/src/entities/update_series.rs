//! Command to update a series

use crate::{Command, CommandResult, UpdateSeriesResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to update an existing series
///
/// This command updates a series's metadata.
/// At least one field must be provided for update.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{UpdateSeriesCommand, UpdateSeriesInput};
/// use ritmo_commands::Command;
///
/// let command = UpdateSeriesCommand;
/// let input = UpdateSeriesInput {
///     series_id: 1,
///     name: Some("Harry Potter Series".to_string()),
///     description: None,
///     total_books: Some(7),
///     completed: Some(true),
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Series updated: {}", result.name);
/// ```
#[derive(Debug, Clone)]
pub struct UpdateSeriesCommand;

/// Input parameters for updating a series
#[derive(Debug, Clone)]
pub struct UpdateSeriesInput {
    /// Series ID to update (required)
    pub series_id: i64,

    /// New series name (optional)
    pub name: Option<String>,

    /// New description (optional)
    pub description: Option<String>,

    /// New total books count (optional)
    pub total_books: Option<i64>,

    /// New completed status (optional)
    pub completed: Option<bool>,
}

#[async_trait]
impl Command for UpdateSeriesCommand {
    type Input = UpdateSeriesInput;
    type Output = UpdateSeriesResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate at least one field is being updated
        if input.name.is_none() && input.description.is_none() && input.total_books.is_none() && input.completed.is_none() {
            return Err(crate::CommandError::Validation(
                "At least one field must be provided for update".to_string()
            ));
        }

        // Validate series name if provided
        if let Some(ref name) = input.name {
            if name.trim().is_empty() {
                return Err(crate::CommandError::Validation(
                    "Series name cannot be empty".to_string()
                ));
            }

            if name.len() > 200 {
                return Err(crate::CommandError::Validation(
                    "Series name too long (max 200 characters)".to_string()
                ));
            }
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

        // Check if series exists
        let existing = sqlx::query!("SELECT id, name FROM series WHERE id = ?", input.series_id)
            .fetch_optional(pool)
            .await?;

        if existing.is_none() {
            return Err(crate::CommandError::NotFound(format!(
                "Series with ID {} not found",
                input.series_id
            )));
        }

        // If updating name, check for duplicates
        if let Some(ref new_name) = input.name {
            let duplicate = sqlx::query!(
                "SELECT id FROM series WHERE name = ? AND id != ?",
                new_name,
                input.series_id
            )
            .fetch_optional(pool)
            .await?;

            if duplicate.is_some() {
                return Err(crate::CommandError::Validation(format!(
                    "Series '{}' already exists",
                    new_name
                )));
            }
        }

        // Build update query dynamically
        let mut update_parts = Vec::new();

        if input.name.is_some() {
            update_parts.push("name = ?1");
        }
        if input.description.is_some() {
            update_parts.push("description = ?2");
        }
        if input.total_books.is_some() {
            update_parts.push("total_books = ?3");
        }
        if input.completed.is_some() {
            update_parts.push("completed = ?4");
        }
        update_parts.push("updated_at = strftime('%s', 'now')");

        let query = format!(
            "UPDATE series SET {} WHERE id = ?5",
            update_parts.join(", ")
        );

        // Prepare values
        let completed_int = input.completed.map(|c| if c { 1 } else { 0 });

        // Execute update
        sqlx::query(&query)
            .bind(input.name.as_ref())
            .bind(input.description.as_ref())
            .bind(input.total_books)
            .bind(completed_int)
            .bind(input.series_id)
            .execute(pool)
            .await?;

        // Get updated series
        let updated = sqlx::query!("SELECT name, updated_at FROM series WHERE id = ?", input.series_id)
            .fetch_one(pool)
            .await?;

        let updated_at = chrono::DateTime::from_timestamp(updated.updated_at, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        Ok(UpdateSeriesResult {
            series_id: input.series_id,
            name: updated.name,
            updated_at,
            message: format!("Series updated successfully"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_no_fields() {
        let command = UpdateSeriesCommand;
        let input = UpdateSeriesInput {
            series_id: 1,
            name: None,
            description: None,
            total_books: None,
            completed: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_empty_name() {
        let command = UpdateSeriesCommand;
        let input = UpdateSeriesInput {
            series_id: 1,
            name: Some("".to_string()),
            description: None,
            total_books: None,
            completed: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_invalid_total_books() {
        let command = UpdateSeriesCommand;
        let input = UpdateSeriesInput {
            series_id: 1,
            name: None,
            description: None,
            total_books: Some(0),
            completed: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_valid_input() {
        let command = UpdateSeriesCommand;
        let input = UpdateSeriesInput {
            series_id: 1,
            name: Some("Harry Potter Series".to_string()),
            description: None,
            total_books: None,
            completed: Some(true),
        };

        let result = command.validate(&input);
        assert!(result.is_ok());
    }
}
