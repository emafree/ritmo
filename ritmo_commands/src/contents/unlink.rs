//! Command to unlink a content from a book

use crate::{Command, CommandResult, UnlinkContentResult};
use async_trait::async_trait;
use ritmo_core::service::unlink_content_from_book;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to unlink a content from a book
///
/// This command removes the association between a content and a book.
/// The content and book themselves are not deleted, only their link.
#[derive(Debug, Clone)]
pub struct UnlinkContentCommand;

/// Input parameters for unlinking content from book
#[derive(Debug, Clone)]
pub struct UnlinkContentInput {
    /// Content ID (required)
    pub content_id: i64,

    /// Book ID (required)
    pub book_id: i64,
}

#[async_trait]
impl Command for UnlinkContentCommand {
    type Input = UnlinkContentInput;
    type Output = UnlinkContentResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Execute unlink
        unlink_content_from_book(pool, input.content_id, input.book_id).await?;

        // Return structured result
        Ok(UnlinkContentResult {
            content_id: input.content_id,
            book_id: input.book_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_construction() {
        let input = UnlinkContentInput {
            content_id: 1,
            book_id: 2,
        };
        assert_eq!(input.content_id, 1);
        assert_eq!(input.book_id, 2);
    }
}
