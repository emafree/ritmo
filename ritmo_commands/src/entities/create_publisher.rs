//! Command to create a new publisher

use crate::{Command, CommandResult, CreatePublisherResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to create a new publisher
///
/// This command creates a new publisher with optional metadata.
/// Validates that the publisher name is unique and non-empty.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{CreatePublisherCommand, CreatePublisherInput};
/// use ritmo_commands::Command;
///
/// let command = CreatePublisherCommand;
/// let input = CreatePublisherInput {
///     name: "Penguin Books".to_string(),
///     country: Some("UK".to_string()),
///     website: Some("https://penguin.com".to_string()),
///     notes: None,
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Publisher created with ID: {}", result.publisher_id);
/// ```
#[derive(Debug, Clone)]
pub struct CreatePublisherCommand;

/// Input parameters for creating a publisher
#[derive(Debug, Clone)]
pub struct CreatePublisherInput {
    /// Publisher name (required, must be unique)
    pub name: String,

    /// Country (optional)
    pub country: Option<String>,

    /// Website (optional)
    pub website: Option<String>,

    /// Notes (optional)
    pub notes: Option<String>,
}

#[async_trait]
impl Command for CreatePublisherCommand {
    type Input = CreatePublisherInput;
    type Output = CreatePublisherResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate publisher name is not empty
        if input.name.trim().is_empty() {
            return Err(crate::CommandError::Validation(
                "Publisher name cannot be empty".to_string()
            ));
        }

        // Validate publisher name length
        if input.name.len() > 200 {
            return Err(crate::CommandError::Validation(
                "Publisher name too long (max 200 characters)".to_string()
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

        // Check if publisher already exists
        let existing = sqlx::query!("SELECT id FROM publishers WHERE name = ?", input.name)
            .fetch_optional(pool)
            .await?;

        if existing.is_some() {
            return Err(crate::CommandError::Validation(format!(
                "Publisher '{}' already exists",
                input.name
            )));
        }

        // Insert new publisher
        let result = sqlx::query!(
            "INSERT INTO publishers (name, country, website, notes) VALUES (?, ?, ?, ?) RETURNING id, created_at",
            input.name,
            input.country,
            input.website,
            input.notes
        )
        .fetch_one(pool)
        .await?;

        let publisher_id = result.id;
        let created_at = chrono::DateTime::from_timestamp(result.created_at, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        Ok(CreatePublisherResult {
            publisher_id,
            name: input.name.clone(),
            created_at,
            message: format!("Publisher '{}' created successfully", input.name),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_empty_name() {
        let command = CreatePublisherCommand;
        let input = CreatePublisherInput {
            name: "".to_string(),
            country: None,
            website: None,
            notes: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_name_too_long() {
        let command = CreatePublisherCommand;
        let input = CreatePublisherInput {
            name: "a".repeat(201),
            country: None,
            website: None,
            notes: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_valid_input() {
        let command = CreatePublisherCommand;
        let input = CreatePublisherInput {
            name: "Penguin Books".to_string(),
            country: Some("UK".to_string()),
            website: None,
            notes: None,
        };

        let result = command.validate(&input);
        assert!(result.is_ok());
    }
}
