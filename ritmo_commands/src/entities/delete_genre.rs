//! Command to delete a genre

use crate::{Command, CommandResult, DeleteGenreResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to delete a genre
///
/// This command deletes a genre from the database.
/// Validates that the genre exists before deletion.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{DeleteGenreCommand, DeleteGenreInput};
/// use ritmo_commands::Command;
///
/// let command = DeleteGenreCommand;
/// let input = DeleteGenreInput {
///     id: 1,
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Genre deleted: {}", result.message);
/// ```
#[derive(Debug, Clone)]
pub struct DeleteGenreCommand;

/// Input parameters for deleting a genre
#[derive(Debug, Clone)]
pub struct DeleteGenreInput {
    /// Genre ID (required)
    pub id: i64,
}

#[async_trait]
impl Command for DeleteGenreCommand {
    type Input = DeleteGenreInput;
    type Output = DeleteGenreResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Check if genre exists
        let existing = sqlx::query!("SELECT id, name FROM genres WHERE id = ?", input.id)
            .fetch_optional(pool)
            .await?;

        let existing = existing.ok_or_else(|| {
            crate::CommandError::NotFound(format!("Genre with ID {} not found", input.id))
        })?;

        let genre_name = existing.name;

        // Check if genre is in use by any contents
        let usage_count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM contents WHERE genre_id = ?",
            input.id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        if usage_count > 0 {
            return Err(crate::CommandError::Validation(format!(
                "Cannot delete genre '{}': it is used by {} content(s)",
                genre_name, usage_count
            )));
        }

        // Delete the genre
        sqlx::query!("DELETE FROM genres WHERE id = ?", input.id)
            .execute(pool)
            .await?;

        Ok(DeleteGenreResult {
            genre_id: input.id,
            name: genre_name.clone(),
            message: format!("Genre '{}' deleted successfully", genre_name),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_input() {
        let input = DeleteGenreInput { id: 1 };
        assert_eq!(input.id, 1);
    }
}
