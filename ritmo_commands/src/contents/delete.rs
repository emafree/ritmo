//! Command to delete a single content

use crate::{Command, CommandResult, DeleteContentResult};
use async_trait::async_trait;
use ritmo_core::service::delete_content;
use ritmo_db_core::LibraryConfig;
use ritmo_errors::reporter::SilentReporter;
use sqlx::SqlitePool;

/// Command to delete a content
///
/// This command deletes a single content from the database.
/// Note: Contents don't have physical files, only database records.
#[derive(Debug, Clone)]
pub struct DeleteContentCommand;

/// Input parameters for deleting a content
#[derive(Debug, Clone)]
pub struct DeleteContentInput {
    /// Content ID to delete (required)
    pub content_id: i64,
}

impl Default for DeleteContentInput {
    fn default() -> Self {
        Self { content_id: 0 }
    }
}

#[async_trait]
impl Command for DeleteContentCommand {
    type Input = DeleteContentInput;
    type Output = DeleteContentResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Use silent reporter (CLI will handle output)
        let mut reporter = SilentReporter;

        // Execute deletion
        delete_content(pool, input.content_id, &mut reporter).await?;

        // Return structured result
        Ok(DeleteContentResult {
            content_id: input.content_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_input() {
        let input = DeleteContentInput::default();
        assert_eq!(input.content_id, 0);
    }
}
