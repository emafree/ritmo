//! Command to update a single book's metadata

use crate::{Command, CommandResult, UpdateBookResult};
use async_trait::async_trait;
use ritmo_core::service::{update_book, BookUpdateMetadata};
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to update a book
///
/// This command updates a single book's metadata in the database.
/// It uses the book ID to identify which book to update.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::books::{UpdateBookCommand, UpdateBookInput};
/// use ritmo_commands::Command;
///
/// let command = UpdateBookCommand;
/// let input = UpdateBookInput {
///     book_id: 1,
///     title: Some("New Title".to_string()),
///     publisher: Some("New Publisher".to_string()),
///     year: Some(2025),
///     ..Default::default()
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Updated book ID: {}", result.book_id);
/// ```
#[derive(Debug, Clone)]
pub struct UpdateBookCommand;

/// Input parameters for updating a book
///
/// All fields are optional - only specified fields will be updated.
/// Use None to leave a field unchanged.
#[derive(Debug, Clone, Default)]
pub struct UpdateBookInput {
    /// Book ID (required)
    pub book_id: i64,

    /// New title (optional)
    pub title: Option<String>,

    /// New original title (optional)
    pub original_title: Option<String>,

    /// New people: Vec<(name, role)> (optional)
    pub people: Option<Vec<(String, String)>>,

    /// New publisher (optional)
    pub publisher: Option<String>,

    /// New publication year (optional)
    pub year: Option<i32>,

    /// New ISBN (optional)
    pub isbn: Option<String>,

    /// New format (optional)
    pub format: Option<String>,

    /// New series (optional)
    pub series: Option<String>,

    /// New series index (optional)
    pub series_index: Option<i64>,

    /// New page count (optional)
    pub pages: Option<i64>,

    /// New notes (optional)
    pub notes: Option<String>,

    /// New tags (optional)
    pub tags: Option<Vec<String>>,
}

#[async_trait]
impl Command for UpdateBookCommand {
    type Input = UpdateBookInput;
    type Output = UpdateBookResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Prepare metadata for update service
        let metadata = BookUpdateMetadata {
            title: input.title,
            original_title: input.original_title,
            people: input.people,
            publisher: input.publisher,
            year: input.year,
            isbn: input.isbn,
            format: input.format,
            series: input.series,
            series_index: input.series_index,
            pages: input.pages,
            notes: input.notes,
            tags: input.tags,
        };

        // Execute update
        update_book(pool, input.book_id, metadata).await?;

        // Return structured result
        Ok(UpdateBookResult {
            book_id: input.book_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_input() {
        let input = UpdateBookInput::default();
        assert_eq!(input.book_id, 0);
        assert!(input.title.is_none());
        assert!(input.publisher.is_none());
    }
}
