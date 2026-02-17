//! Command to delete a genre

use crate::{Command, CommandResult, DeleteGenreResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to delete a genre
///
/// This command deletes a genre from the database. Contents using this genre
/// will have their genre_id set to NULL.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{DeleteGenreCommand, DeleteGenreInput};
/// use ritmo_commands::Command;
///
/// let command = DeleteGenreCommand;
/// let input = DeleteGenreInput { genre_id: 1 };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Deleted genre: {}", result.name);
/// ```
#[derive(Debug, Clone)]
pub struct DeleteGenreCommand;

/// Input parameters for deleting a genre
#[derive(Debug, Clone)]
pub struct DeleteGenreInput {
    /// Genre ID to delete
    pub genre_id: i64,
}

#[async_trait]
impl Command for DeleteGenreCommand {
    type Input = DeleteGenreInput;
    type Output = DeleteGenreResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate genre_id is positive
        if input.genre_id <= 0 {
            return Err(crate::CommandError::Validation(
                "Genre ID must be positive".to_string()
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

        // Check if genre exists
        let existing = sqlx::query!("SELECT id, name FROM genres WHERE id = ?", input.genre_id)
            .fetch_optional(pool)
            .await?;

        let genre = existing.ok_or_else(|| {
            crate::CommandError::NotFound(format!("Genre with ID {} not found", input.genre_id))
        })?;

        // Count affected contents
        let contents_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM contents WHERE genre_id = ?",
            input.genre_id
        )
        .fetch_one(pool)
        .await?;

        let contents_affected = contents_count.count as i64;

        // Create warning if contents will be affected
        let warning = if contents_affected > 0 {
            Some(format!(
                "{} content(s) were using this genre and will have genre_id set to NULL",
                contents_affected
            ))
        } else {
            None
        };

        // Delete the genre (CASCADE will handle setting genre_id to NULL in contents)
        sqlx::query!("DELETE FROM genres WHERE id = ?", input.genre_id)
            .execute(pool)
            .await?;

        let deleted_at = chrono::Utc::now().to_rfc3339();

        Ok(DeleteGenreResult {
            genre_id: input.genre_id,
            name: genre.name,
            deleted_at,
            contents_affected,
            warning,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_invalid_id() {
        let command = DeleteGenreCommand;
        let input = DeleteGenreInput { genre_id: 0 };

        let result = command.validate(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_valid_input() {
        let command = DeleteGenreCommand;
        let input = DeleteGenreInput { genre_id: 1 };

        let result = command.validate(&input);
        assert!(result.is_ok());
    }
}
