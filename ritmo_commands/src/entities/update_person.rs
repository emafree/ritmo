//! Command to update a person

use crate::{Command, CommandResult, UpdatePersonResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to update an existing person
///
/// This command updates a person's metadata.
/// At least one field must be provided for update.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{UpdatePersonCommand, UpdatePersonInput};
/// use ritmo_commands::Command;
///
/// let command = UpdatePersonCommand;
/// let input = UpdatePersonInput {
///     person_id: 1,
///     name: Some("Stephen Edwin King".to_string()),
///     display_name: Some("Stephen King".to_string()),
///     given_name: Some("Stephen".to_string()),
///     surname: Some("King".to_string()),
///     middle_names: Some("Edwin".to_string()),
///     title: None,
///     suffix: None,
///     nationality: Some("American".to_string()),
///     birth_date: None,
///     death_date: None,
///     biography: None,
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Person updated: {}", result.name);
/// ```
#[derive(Debug, Clone)]
pub struct UpdatePersonCommand;

/// Input parameters for updating a person
#[derive(Debug, Clone)]
pub struct UpdatePersonInput {
    /// Person ID to update (required)
    pub person_id: i64,

    /// New person name (optional)
    pub name: Option<String>,

    /// New display name (optional)
    pub display_name: Option<String>,

    /// New given name (optional)
    pub given_name: Option<String>,

    /// New surname (optional)
    pub surname: Option<String>,

    /// New middle names (optional)
    pub middle_names: Option<String>,

    /// New title (optional)
    pub title: Option<String>,

    /// New suffix (optional)
    pub suffix: Option<String>,

    /// New nationality (optional)
    pub nationality: Option<String>,

    /// New birth date (optional)
    pub birth_date: Option<i64>,

    /// New death date (optional)
    pub death_date: Option<i64>,

    /// New biography (optional)
    pub biography: Option<String>,
}

#[async_trait]
impl Command for UpdatePersonCommand {
    type Input = UpdatePersonInput;
    type Output = UpdatePersonResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate at least one field is being updated
        if input.name.is_none()
            && input.display_name.is_none()
            && input.given_name.is_none()
            && input.surname.is_none()
            && input.middle_names.is_none()
            && input.title.is_none()
            && input.suffix.is_none()
            && input.nationality.is_none()
            && input.birth_date.is_none()
            && input.death_date.is_none()
            && input.biography.is_none()
        {
            return Err(crate::CommandError::Validation(
                "At least one field must be provided for update".to_string()
            ));
        }

        // Validate person name if provided
        if let Some(ref name) = input.name {
            if name.trim().is_empty() {
                return Err(crate::CommandError::Validation(
                    "Person name cannot be empty".to_string()
                ));
            }

            if name.len() > 200 {
                return Err(crate::CommandError::Validation(
                    "Person name too long (max 200 characters)".to_string()
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

        // Check if person exists and get current data
        let existing = sqlx::query!(
            "SELECT id, name, display_name, birth_date, death_date FROM people WHERE id = ?",
            input.person_id
        )
        .fetch_optional(pool)
        .await?;

        if existing.is_none() {
            return Err(crate::CommandError::NotFound(format!(
                "Person with ID {} not found",
                input.person_id
            )));
        }

        let current = existing.unwrap();

        // Validate birth_date and death_date relationship
        let new_birth = input.birth_date.or(current.birth_date);
        let new_death = input.death_date.or(current.death_date);
        
        if let (Some(birth), Some(death)) = (new_birth, new_death) {
            if death < birth {
                return Err(crate::CommandError::Validation(
                    "Death date cannot be before birth date".to_string()
                ));
            }
        }

        // Build update query dynamically
        let mut update_parts = Vec::new();

        if input.name.is_some() {
            update_parts.push("name = ?1");
        }
        if input.display_name.is_some() {
            update_parts.push("display_name = ?2");
        }
        if input.given_name.is_some() {
            update_parts.push("given_name = ?3");
        }
        if input.surname.is_some() {
            update_parts.push("surname = ?4");
        }
        if input.middle_names.is_some() {
            update_parts.push("middle_names = ?5");
        }
        if input.title.is_some() {
            update_parts.push("title = ?6");
        }
        if input.suffix.is_some() {
            update_parts.push("suffix = ?7");
        }
        if input.nationality.is_some() {
            update_parts.push("nationality = ?8");
        }
        if input.birth_date.is_some() {
            update_parts.push("birth_date = ?9");
        }
        if input.death_date.is_some() {
            update_parts.push("death_date = ?10");
        }
        if input.biography.is_some() {
            update_parts.push("biography = ?11");
        }
        update_parts.push("updated_at = strftime('%s', 'now')");

        let query = format!(
            "UPDATE people SET {} WHERE id = ?12",
            update_parts.join(", ")
        );

        // Execute update
        sqlx::query(&query)
            .bind(input.name.as_ref())
            .bind(input.display_name.as_ref())
            .bind(input.given_name.as_ref())
            .bind(input.surname.as_ref())
            .bind(input.middle_names.as_ref())
            .bind(input.title.as_ref())
            .bind(input.suffix.as_ref())
            .bind(input.nationality.as_ref())
            .bind(input.birth_date)
            .bind(input.death_date)
            .bind(input.biography.as_ref())
            .bind(input.person_id)
            .execute(pool)
            .await?;

        // Get updated person
        let updated = sqlx::query!(
            "SELECT name, display_name, updated_at FROM people WHERE id = ?",
            input.person_id
        )
        .fetch_one(pool)
        .await?;

        let updated_at = chrono::DateTime::from_timestamp(updated.updated_at, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        Ok(UpdatePersonResult {
            person_id: input.person_id,
            name: updated.name,
            display_name: updated.display_name,
            updated_at,
            message: format!("Person updated successfully"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_no_fields() {
        let command = UpdatePersonCommand;
        let input = UpdatePersonInput {
            person_id: 1,
            name: None,
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
    fn test_validation_empty_name() {
        let command = UpdatePersonCommand;
        let input = UpdatePersonInput {
            person_id: 1,
            name: Some("".to_string()),
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
    fn test_validation_valid_input() {
        let command = UpdatePersonCommand;
        let input = UpdatePersonInput {
            person_id: 1,
            name: Some("Stephen Edwin King".to_string()),
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
        assert!(result.is_ok());
    }
}
