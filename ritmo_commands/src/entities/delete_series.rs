//! Command to delete a series

use crate::{Command, CommandResult, DeleteSeriesResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to delete a series
///
/// This command deletes a series and returns information about
/// affected books. Cascade deletion will set series_id to NULL
/// in books automatically (if foreign key constraints are configured).
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{DeleteSeriesCommand, DeleteSeriesInput};
/// use ritmo_commands::Command;
///
/// let command = DeleteSeriesCommand;
/// let input = DeleteSeriesInput {
///     series_id: 1,
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Series deleted. Books affected: {}", result.books_affected);
/// ```
#[derive(Debug, Clone)]
pub struct DeleteSeriesCommand;

/// Input parameters for deleting a series
#[derive(Debug, Clone)]
pub struct DeleteSeriesInput {
    /// Series ID to delete (required)
    pub series_id: i64,
}

#[async_trait]
impl Command for DeleteSeriesCommand {
    type Input = DeleteSeriesInput;
    type Output = DeleteSeriesResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Check if series exists
        let existing = sqlx::query!("SELECT id, name FROM series WHERE id = ?", input.series_id)
            .fetch_optional(pool)
            .await?;

        let series = match existing {
            Some(s) => s,
            None => {
                return Err(crate::CommandError::NotFound(format!(
                    "Series with ID {} not found",
                    input.series_id
                )))
            }
        };

        // Count affected books before deletion
        let affected_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM books WHERE series_id = ?",
            input.series_id
        )
        .fetch_one(pool)
        .await?;

        let books_affected = affected_count.count;

        // Delete series (books.series_id will be set to NULL by cascade)
        sqlx::query!("DELETE FROM series WHERE id = ?", input.series_id)
            .execute(pool)
            .await?;

        let deleted_at = chrono::Utc::now().to_rfc3339();

        // Add warning if books were affected
        let warning = if books_affected > 0 {
            Some(format!(
                "Series was removed from {} book(s)",
                books_affected
            ))
        } else {
            None
        };

        Ok(DeleteSeriesResult {
            series_id: input.series_id,
            name: series.name,
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
        let input = DeleteSeriesInput { series_id: 1 };

        assert_eq!(input.series_id, 1);
    }
}
