//! Command to create a new person

use crate::{Command, CommandResult, CreatePersonResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to create a new person
///
/// This command creates a new person (author, translator, etc.) with optional metadata.
/// Validates that the person name is non-empty.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{CreatePersonCommand, CreatePersonInput};
/// use ritmo_commands::Command;
///
/// let command = CreatePersonCommand;
/// let input = CreatePersonInput {
///     name: "Stephen King".to_string(),
///     display_name: Some("Stephen King".to_string()),
///     given_name: Some("Stephen".to_string()),
///     surname: Some("King".to_string()),
///     middle_names: None,
///     title: None,
///     suffix: None,
///     nationality: Some("American".to_string()),
///     birth_date: None,
///     death_date: None,
///     biography: None,
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Person created with ID: {}", result.person_id);
/// ```
#[derive(Debug, Clone)]
pub struct CreatePersonCommand;

/// Input parameters for creating a person
#[derive(Debug, Clone)]
pub struct CreatePersonInput {
    /// Person name (required)
    pub name: String,

    /// Display name (optional, defaults to name if not provided)
    pub display_name: Option<String>,

    /// Given name (first name) (optional)
    pub given_name: Option<String>,

    /// Surname (last name) (optional)
    pub surname: Option<String>,

    /// Middle names (optional)
    pub middle_names: Option<String>,

    /// Title (e.g., "Dr.", "Prof.") (optional)
    pub title: Option<String>,

    /// Suffix (e.g., "Jr.", "III") (optional)
    pub suffix: Option<String>,

    /// Nationality (optional)
    pub nationality: Option<String>,

    /// Birth date as Unix timestamp (optional)
    pub birth_date: Option<i64>,

    /// Death date as Unix timestamp (optional)
    pub death_date: Option<i64>,

    /// Biography (optional)
    pub biography: Option<String>,
}

#[async_trait]
impl Command for CreatePersonCommand {
    type Input = CreatePersonInput;
    type Output = CreatePersonResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate person name is not empty
        if input.name.trim().is_empty() {
            return Err(crate::CommandError::Validation(
                "Person name cannot be empty".to_string()
            ));
        }

        // Validate person name length
        if input.name.len() > 200 {
            return Err(crate::CommandError::Validation(
                "Person name too long (max 200 characters)".to_string()
            ));
        }

        // Validate birth_date and death_date relationship
        if let (Some(birth), Some(death)) = (input.birth_date, input.death_date) {
            if death < birth {
                return Err(crate::CommandError::Validation(
                    "Death date cannot be before birth date".to_string()
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

        // Use display_name if provided, otherwise use name
        let display_name = input.display_name.clone().or_else(|| Some(input.name.clone()));

        // Insert new person
        let result = sqlx::query!(
            r#"INSERT INTO people (
                name, display_name, given_name, surname, middle_names,
                title, suffix, nationality, birth_date, death_date, biography
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, created_at"#,
            input.name,
            display_name,
            input.given_name,
            input.surname,
            input.middle_names,
            input.title,
            input.suffix,
            input.nationality,
            input.birth_date,
            input.death_date,
            input.biography
        )
        .fetch_one(pool)
        .await?;

        let person_id = result.id;
        let created_at = chrono::DateTime::from_timestamp(result.created_at, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        Ok(CreatePersonResult {
            person_id,
            name: input.name.clone(),
            display_name,
            created_at,
            message: format!("Person '{}' created successfully", input.name),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_empty_name() {
        let command = CreatePersonCommand;
        let input = CreatePersonInput {
            name: "".to_string(),
            display_name: None,
            given_name: None,
            surname: None,
            middle_names: None,
            title: None,
            suffix: None,
            nationality: None,
            birth_date: None,
            death_date: None,
            biography: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_name_too_long() {
        let command = CreatePersonCommand;
        let input = CreatePersonInput {
            name: "a".repeat(201),
            display_name: None,
            given_name: None,
            surname: None,
            middle_names: None,
            title: None,
            suffix: None,
            nationality: None,
            birth_date: None,
            death_date: None,
            biography: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_death_before_birth() {
        let command = CreatePersonCommand;
        let input = CreatePersonInput {
            name: "Test Person".to_string(),
            display_name: None,
            given_name: None,
            surname: None,
            middle_names: None,
            title: None,
            suffix: None,
            nationality: None,
            birth_date: Some(100),
            death_date: Some(50),
            biography: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_valid_input() {
        let command = CreatePersonCommand;
        let input = CreatePersonInput {
            name: "Stephen King".to_string(),
            display_name: Some("Stephen King".to_string()),
            given_name: Some("Stephen".to_string()),
            surname: Some("King".to_string()),
            middle_names: None,
            title: None,
            suffix: None,
            nationality: Some("American".to_string()),
            birth_date: None,
            death_date: None,
            biography: None,
        };

        let result = command.validate(&input);
        assert!(result.is_ok());
    }
}
