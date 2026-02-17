//! Command to modify the link between a content and a book
//!
//! This command allows changing the book that a content is linked to.
//! Since every content MUST have a link to a book, this command modifies
//! the existing link rather than just adding or removing links.

use crate::{Command, CommandResult, ModifyLinkContentResult};
use async_trait::async_trait;
use ritmo_core::service::{link_content_to_book, unlink_content_from_book};
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to modify the link between a content and a book
///
/// This command handles changing which book a content is linked to.
/// It can:
/// - Link a content to a new book (when old_book_id is None)
/// - Change the link from one book to another
/// - The content must always remain linked to exactly one book
#[derive(Debug, Clone)]
pub struct ModifyLinkContentCommand;

/// Input parameters for modifying content-book link
#[derive(Debug, Clone)]
pub struct ModifyLinkContentInput {
    /// Content ID (required)
    pub content_id: i64,

    /// New book ID to link to (required)
    pub new_book_id: i64,

    /// Old book ID to unlink from (optional - if None, just creates a new link)
    pub old_book_id: Option<i64>,
}

#[async_trait]
impl Command for ModifyLinkContentCommand {
    type Input = ModifyLinkContentInput;
    type Output = ModifyLinkContentResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // If there's an old book ID, unlink from it first
        if let Some(old_book_id) = input.old_book_id {
            // Only unlink if it's different from the new book
            if old_book_id != input.new_book_id {
                unlink_content_from_book(pool, input.content_id, old_book_id).await?;
            } else {
                // Same book - nothing to do
                return Ok(ModifyLinkContentResult {
                    content_id: input.content_id,
                    new_book_id: input.new_book_id,
                    old_book_id: Some(old_book_id),
                });
            }
        }

        // Link to the new book
        link_content_to_book(pool, input.content_id, input.new_book_id).await?;

        // Return structured result
        Ok(ModifyLinkContentResult {
            content_id: input.content_id,
            new_book_id: input.new_book_id,
            old_book_id: input.old_book_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_construction() {
        let input = ModifyLinkContentInput {
            content_id: 1,
            new_book_id: 2,
            old_book_id: Some(3),
        };
        assert_eq!(input.content_id, 1);
        assert_eq!(input.new_book_id, 2);
        assert_eq!(input.old_book_id, Some(3));
    }

    #[test]
    fn test_input_construction_no_old_book() {
        let input = ModifyLinkContentInput {
            content_id: 1,
            new_book_id: 2,
            old_book_id: None,
        };
        assert_eq!(input.content_id, 1);
        assert_eq!(input.new_book_id, 2);
        assert_eq!(input.old_book_id, None);
    }
}
