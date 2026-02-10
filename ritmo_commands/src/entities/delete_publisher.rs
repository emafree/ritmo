//! Command to delete a publisher

use crate::{Command, CommandResult, DeletePublisherResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to delete a publisher
///
/// This command deletes a publisher and returns information about
/// affected books. Cascade deletion will set publisher_id to NULL
/// in books automatically (if foreign key constraints are configured).
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{DeletePublisherCommand, DeletePublisherInput};
/// use ritmo_commands::Command;
///
/// let command = DeletePublisherCommand;
/// let input = DeletePublisherInput {
///     publisher_id: 1,
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Publisher deleted. Books affected: {}", result.books_affected);
/// ```
#[derive(Debug, Clone)]
pub struct DeletePublisherCommand;

/// Input parameters for deleting a publisher
#[derive(Debug, Clone)]
pub struct DeletePublisherInput {
    /// Publisher ID to delete (required)
    pub publisher_id: i64,
}

#[async_trait]
impl Command for DeletePublisherCommand {
    type Input = DeletePublisherInput;
    type Output = DeletePublisherResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Check if publisher exists
        let existing = sqlx::query!("SELECT id, name FROM publishers WHERE id = ?", input.publisher_id)
            .fetch_optional(pool)
            .await?;

        let publisher = match existing {
            Some(p) => p,
            None => {
                return Err(crate::CommandError::NotFound(format!(
                    "Publisher with ID {} not found",
                    input.publisher_id
                )))
            }
        };

        // Count affected books before deletion
        let affected_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM books WHERE publisher_id = ?",
            input.publisher_id
        )
        .fetch_one(pool)
        .await?;

        let books_affected = affected_count.count;

        // Delete publisher (books.publisher_id will be set to NULL by cascade)
        sqlx::query!("DELETE FROM publishers WHERE id = ?", input.publisher_id)
            .execute(pool)
            .await?;

        let deleted_at = chrono::Utc::now().to_rfc3339();

        // Add warning if books were affected
        let warning = if books_affected > 0 {
            Some(format!(
                "Publisher was removed from {} book(s)",
                books_affected
            ))
        } else {
            None
        };

        Ok(DeletePublisherResult {
            publisher_id: input.publisher_id,
            name: publisher.name,
            deleted_at,
            books_affected,
            warning,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_construction() {
        let input = DeletePublisherInput { publisher_id: 1 };

        assert_eq!(input.publisher_id, 1);
    }
}
