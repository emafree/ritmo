//! Command to delete a person

use crate::{Command, CommandResult, DeletePersonResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to delete a person
///
/// This command deletes a person and returns information about
/// affected books and contents. Cascade deletion will remove
/// person associations automatically.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{DeletePersonCommand, DeletePersonInput};
/// use ritmo_commands::Command;
///
/// let command = DeletePersonCommand;
/// let input = DeletePersonInput {
///     person_id: 1,
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Person deleted. Books affected: {}, Contents affected: {}",
///          result.books_affected, result.contents_affected);
/// ```
#[derive(Debug, Clone)]
pub struct DeletePersonCommand;

/// Input parameters for deleting a person
#[derive(Debug, Clone)]
pub struct DeletePersonInput {
    /// Person ID to delete (required)
    pub person_id: i64,
}

#[async_trait]
impl Command for DeletePersonCommand {
    type Input = DeletePersonInput;
    type Output = DeletePersonResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Check if person exists
        let existing = sqlx::query!("SELECT id, name FROM people WHERE id = ?", input.person_id)
            .fetch_optional(pool)
            .await?;

        let person = match existing {
            Some(p) => p,
            None => {
                return Err(crate::CommandError::NotFound(format!(
                    "Person with ID {} not found",
                    input.person_id
                )))
            }
        };

        // Count affected books before deletion
        let books_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM x_books_people_roles WHERE person_id = ?",
            input.person_id
        )
        .fetch_one(pool)
        .await?;

        let books_affected = books_count.count;

        // Count affected contents before deletion
        let contents_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM x_contents_people_roles WHERE person_id = ?",
            input.person_id
        )
        .fetch_one(pool)
        .await?;

        let contents_affected = contents_count.count;

        // Delete person (cascade will handle x_books_people_roles and x_contents_people_roles)
        sqlx::query!("DELETE FROM people WHERE id = ?", input.person_id)
            .execute(pool)
            .await?;

        let deleted_at = chrono::Utc::now().to_rfc3339();

        // Add warning if books or contents were affected
        let warning = if books_affected > 0 || contents_affected > 0 {
            let mut parts = Vec::new();
            if books_affected > 0 {
                parts.push(format!("{} book(s)", books_affected));
            }
            if contents_affected > 0 {
                parts.push(format!("{} content(s)", contents_affected));
            }
            Some(format!("Person was removed from {}", parts.join(" and ")))
        } else {
            None
        };

        Ok(DeletePersonResult {
            person_id: input.person_id,
            name: person.name,
            deleted_at,
            books_affected,
            contents_affected,
            warning,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_construction() {
        let input = DeletePersonInput { person_id: 1 };

        assert_eq!(input.person_id, 1);
    }
}
