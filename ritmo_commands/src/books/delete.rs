//! Command to delete a single book

use crate::{Command, CommandResult, DeleteBookResult};
use async_trait::async_trait;
use ritmo_core::service::{delete_book, DeleteOptions};
use ritmo_db_core::LibraryConfig;
use ritmo_errors::reporter::SilentReporter;
use sqlx::SqlitePool;

/// Command to delete a book
///
/// This command deletes a single book from the database and optionally
/// removes the physical file from storage.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::books::{DeleteBookCommand, DeleteBookInput};
/// use ritmo_commands::Command;
///
/// let command = DeleteBookCommand;
/// let input = DeleteBookInput {
///     book_id: 1,
///     delete_file: false,
///     force: false,
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Deleted book ID: {}", result.book_id);
/// ```
#[derive(Debug, Clone)]
pub struct DeleteBookCommand;

/// Input parameters for deleting a book
#[derive(Debug, Clone)]
pub struct DeleteBookInput {
    /// Book ID to delete (required)
    pub book_id: i64,

    /// Whether to delete the physical file (default: false)
    pub delete_file: bool,

    /// Force deletion even if there are warnings (default: false)
    pub force: bool,
}

impl Default for DeleteBookInput {
    fn default() -> Self {
        Self {
            book_id: 0,
            delete_file: false,
            force: false,
        }
    }
}

#[async_trait]
impl Command for DeleteBookCommand {
    type Input = DeleteBookInput;
    type Output = DeleteBookResult;

    async fn execute(
        &self,
        config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Prepare delete options
        let options = DeleteOptions {
            delete_file: input.delete_file,
            force: input.force,
        };

        // Use silent reporter (CLI will handle output)
        let mut reporter = SilentReporter;

        // Execute deletion
        delete_book(config, pool, input.book_id, &options, &mut reporter).await?;

        // Return structured result
        Ok(DeleteBookResult {
            book_id: input.book_id,
            file_deleted: input.delete_file,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_input() {
        let input = DeleteBookInput::default();
        assert_eq!(input.book_id, 0);
        assert!(!input.delete_file);
        assert!(!input.force);
    }
}
