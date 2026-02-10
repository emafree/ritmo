//! Command to delete a tag

use crate::{Command, CommandResult, DeleteTagResult};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to delete a tag
///
/// This command deletes a tag and returns information about
/// affected books. Cascade deletion will remove tag associations
/// from books automatically.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{DeleteTagCommand, DeleteTagInput};
/// use ritmo_commands::Command;
///
/// let command = DeleteTagCommand;
/// let input = DeleteTagInput {
///     tag_id: 1,
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Tag deleted. Books affected: {}", result.books_affected);
/// ```
#[derive(Debug, Clone)]
pub struct DeleteTagCommand;

/// Input parameters for deleting a tag
#[derive(Debug, Clone)]
pub struct DeleteTagInput {
    /// Tag ID to delete (required)
    pub tag_id: i64,
}

#[async_trait]
impl Command for DeleteTagCommand {
    type Input = DeleteTagInput;
    type Output = DeleteTagResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Check if tag exists
        let existing = sqlx::query!("SELECT id, name FROM tags WHERE id = ?", input.tag_id)
            .fetch_optional(pool)
            .await?;

        let tag = match existing {
            Some(t) => t,
            None => {
                return Err(crate::CommandError::NotFound(format!(
                    "Tag with ID {} not found",
                    input.tag_id
                )))
            }
        };

        // Count affected books before deletion
        let affected_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM x_books_tags WHERE tag_id = ?",
            input.tag_id
        )
        .fetch_one(pool)
        .await?;

        let books_affected = affected_count.count;

        // Delete tag (cascade will handle x_books_tags)
        sqlx::query!("DELETE FROM tags WHERE id = ?", input.tag_id)
            .execute(pool)
            .await?;

        let deleted_at = chrono::Utc::now().to_rfc3339();

        // Add warning if books were affected
        let warning = if books_affected > 0 {
            Some(format!(
                "Tag was removed from {} book(s)",
                books_affected
            ))
        } else {
            None
        };

        Ok(DeleteTagResult {
            tag_id: input.tag_id,
            name: tag.name,
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
        let input = DeleteTagInput { tag_id: 1 };

        assert_eq!(input.tag_id, 1);
    }
}
