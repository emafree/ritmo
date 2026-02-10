//! Command to update a publisher

use crate::{Command, CommandResult, UpdatePublisherResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to update an existing publisher
///
/// This command updates a publisher's metadata.
/// At least one field must be provided for update.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{UpdatePublisherCommand, UpdatePublisherInput};
/// use ritmo_commands::Command;
///
/// let command = UpdatePublisherCommand;
/// let input = UpdatePublisherInput {
///     publisher_id: 1,
///     name: Some("Penguin Random House".to_string()),
///     country: Some("USA".to_string()),
///     website: None,
///     notes: None,
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Publisher updated: {}", result.name);
/// ```
#[derive(Debug, Clone)]
pub struct UpdatePublisherCommand;

/// Input parameters for updating a publisher
#[derive(Debug, Clone)]
pub struct UpdatePublisherInput {
    /// Publisher ID to update (required)
    pub publisher_id: i64,

    /// New publisher name (optional)
    pub name: Option<String>,

    /// New country (optional)
    pub country: Option<String>,

    /// New website (optional)
    pub website: Option<String>,

    /// New notes (optional)
    pub notes: Option<String>,
}

#[async_trait]
impl Command for UpdatePublisherCommand {
    type Input = UpdatePublisherInput;
    type Output = UpdatePublisherResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate at least one field is being updated
        if input.name.is_none() && input.country.is_none() && input.website.is_none() && input.notes.is_none() {
            return Err(crate::CommandError::Validation(
                "At least one field must be provided for update".to_string()
            ));
        }

        // Validate publisher name if provided
        if let Some(ref name) = input.name {
            if name.trim().is_empty() {
                return Err(crate::CommandError::Validation(
                    "Publisher name cannot be empty".to_string()
                ));
            }

            if name.len() > 200 {
                return Err(crate::CommandError::Validation(
                    "Publisher name too long (max 200 characters)".to_string()
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

        // Check if publisher exists
        let existing = sqlx::query!("SELECT id, name FROM publishers WHERE id = ?", input.publisher_id)
            .fetch_optional(pool)
            .await?;

        if existing.is_none() {
            return Err(crate::CommandError::NotFound(format!(
                "Publisher with ID {} not found",
                input.publisher_id
            )));
        }

        // If updating name, check for duplicates
        if let Some(ref new_name) = input.name {
            let duplicate = sqlx::query!(
                "SELECT id FROM publishers WHERE name = ? AND id != ?",
                new_name,
                input.publisher_id
            )
            .fetch_optional(pool)
            .await?;

            if duplicate.is_some() {
                return Err(crate::CommandError::Validation(format!(
                    "Publisher '{}' already exists",
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

        if let Some(ref country) = input.country {
            update_fields.push("country = ?");
            values.push(country.clone());
        }

        if let Some(ref website) = input.website {
            update_fields.push("website = ?");
            values.push(website.clone());
        }

        if let Some(ref notes) = input.notes {
            update_fields.push("notes = ?");
            values.push(notes.clone());
        }

        update_fields.push("updated_at = strftime('%s', 'now')");

        let query = format!(
            "UPDATE publishers SET {} WHERE id = ?",
            update_fields.join(", ")
        );

        // Execute update
        let mut query_builder = sqlx::query(&query);
        for value in &values {
            query_builder = query_builder.bind(value);
        }
        query_builder = query_builder.bind(input.publisher_id);

        query_builder.execute(pool).await?;

        // Get updated publisher
        let updated = sqlx::query!("SELECT name, updated_at FROM publishers WHERE id = ?", input.publisher_id)
            .fetch_one(pool)
            .await?;

        let updated_at = chrono::DateTime::from_timestamp(updated.updated_at, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        Ok(UpdatePublisherResult {
            publisher_id: input.publisher_id,
            name: updated.name,
            updated_at,
            message: format!("Publisher updated successfully"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_no_fields() {
        let command = UpdatePublisherCommand;
        let input = UpdatePublisherInput {
            publisher_id: 1,
            name: None,
            country: None,
            website: None,
            notes: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_empty_name() {
        let command = UpdatePublisherCommand;
        let input = UpdatePublisherInput {
            publisher_id: 1,
            name: Some("".to_string()),
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
        let command = UpdatePublisherCommand;
        let input = UpdatePublisherInput {
            publisher_id: 1,
            name: Some("Penguin Random House".to_string()),
            country: None,
            website: None,
            notes: None,
        };

        let result = command.validate(&input);
        assert!(result.is_ok());
    }
}
