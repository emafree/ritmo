//! Command to add a single book to the library

use crate::{AddBookResult, Command, CommandError, CommandResult};
use async_trait::async_trait;
use ritmo_core::service::{import_book, BookImportMetadata};
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;
use std::path::PathBuf;

/// Command to add a single book
///
/// This command imports a book file into the library with associated metadata.
/// It validates the input, imports the file, and returns structured information
/// about the imported book.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::books::{AddBookCommand, AddBookInput};
/// use ritmo_commands::Command;
///
/// let command = AddBookCommand;
/// let input = AddBookInput {
///     file_path: PathBuf::from("book.epub"),
///     title: "My Book".to_string(),
///     original_title: None,
///     people: None,
///     publisher: None,
///     year: Some(2024),
///     isbn: None,
///     format: None,
///     series: None,
///     series_index: None,
///     notes: None,
///     tags: None,
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Book added with ID: {}", result.book_id);
/// ```
#[derive(Debug, Clone)]
pub struct AddBookCommand;

/// Input parameters for adding a book
#[derive(Debug, Clone)]
pub struct AddBookInput {
    /// Path to the book file
    pub file_path: PathBuf,

    /// Book title (required)
    pub title: String,

    /// Original title (if translated)
    pub original_title: Option<String>,

    /// People associated with the book: Vec<(name, role)>
    /// Example: vec![("John Doe".to_string(), "role.author".to_string())]
    pub people: Option<Vec<(String, String)>>,

    /// Publisher name
    pub publisher: Option<String>,

    /// Publication year
    pub year: Option<i32>,

    /// ISBN code
    pub isbn: Option<String>,

    /// Format (e.g., "format.epub")
    pub format: Option<String>,

    /// Series name
    pub series: Option<String>,

    /// Position in series
    pub series_index: Option<i64>,

    /// Additional notes
    pub notes: Option<String>,

    /// Tags
    pub tags: Option<Vec<String>>,
}

#[async_trait]
impl Command for AddBookCommand {
    type Input = AddBookInput;
    type Output = AddBookResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate title first (before file system checks)
        if input.title.trim().is_empty() {
            return Err(CommandError::Validation(
                "Title cannot be empty".to_string()
            ));
        }

        // Validate file exists
        if !input.file_path.exists() {
            return Err(CommandError::NotFound(format!(
                "File not found: {}",
                input.file_path.display()
            )));
        }

        // Validate file is readable
        if !input.file_path.is_file() {
            return Err(CommandError::Validation(format!(
                "Path is not a file: {}",
                input.file_path.display()
            )));
        }

        Ok(())
    }

    async fn execute(
        &self,
        config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Validate input
        self.validate(&input)?;

        // Prepare metadata for import service
        let metadata = BookImportMetadata {
            title: input.title.clone(),
            original_title: input.original_title,
            people: input.people,
            publisher: input.publisher,
            year: input.year,
            isbn: input.isbn,
            format: input.format,
            series: input.series,
            series_index: input.series_index,
            notes: input.notes,
            tags: input.tags,
        };

        // Execute import
        let book_id = import_book(config, pool, &input.file_path, metadata).await?;

        // Get file size
        let file_size = std::fs::metadata(&input.file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Return structured result
        Ok(AddBookResult {
            book_id,
            title: input.title,
            file_path: input.file_path.display().to_string(),
            file_size,
            warnings: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_empty_title() {
        let command = AddBookCommand;
        let input = AddBookInput {
            file_path: PathBuf::from("test.epub"),
            title: "".to_string(),
            original_title: None,
            people: None,
            publisher: None,
            year: None,
            isbn: None,
            format: None,
            series: None,
            series_index: None,
            notes: None,
            tags: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_nonexistent_file() {
        let command = AddBookCommand;
        let input = AddBookInput {
            file_path: PathBuf::from("/nonexistent/file.epub"),
            title: "Test Book".to_string(),
            original_title: None,
            people: None,
            publisher: None,
            year: None,
            isbn: None,
            format: None,
            series: None,
            series_index: None,
            notes: None,
            tags: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandError::NotFound(_)));
    }
}
