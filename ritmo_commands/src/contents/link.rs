//! Command to link a content to a book

use crate::{Command, CommandResult, LinkContentResult};
use async_trait::async_trait;
use ritmo_core::service::link_content_to_book;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to link a content to a book
///
/// This command creates an association between a content and a book.
/// A content can be linked to multiple books, and a book can have multiple contents.
#[derive(Debug, Clone)]
pub struct LinkContentCommand;

/// Input parameters for linking content to book
#[derive(Debug, Clone)]
pub struct LinkContentInput {
    /// Content ID (required)
    pub content_id: i64,

    /// Book ID (required)
    pub book_id: i64,
}

#[async_trait]
impl Command for LinkContentCommand {
    type Input = LinkContentInput;
    type Output = LinkContentResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Execute link
        link_content_to_book(pool, input.content_id, input.book_id).await?;

        // Return structured result
        Ok(LinkContentResult {
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
        let input = LinkContentInput {
            content_id: 1,
            book_id: 2,
        };
        assert_eq!(input.content_id, 1);
        assert_eq!(input.book_id, 2);
    }
}
