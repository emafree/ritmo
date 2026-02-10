//! Command to list books with optional filters

use crate::{BookSummary, Command, CommandResult, ListBooksResult};
use async_trait::async_trait;
use ritmo_db_core::{execute_books_query, BookFilters};
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to list books
///
/// This command queries the database for books matching optional filters
/// and returns structured information about each book.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::books::{ListBooksCommand, ListBooksInput};
/// use ritmo_commands::Command;
/// use ritmo_db_core::BookFilters;
///
/// let command = ListBooksCommand;
/// let input = ListBooksInput {
///     filters: BookFilters {
///         author: Some("King".to_string()),
///         format: Some("format.epub".to_string()),
///         ..Default::default()
///     },
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Found {} books", result.books.len());
/// ```
#[derive(Debug, Clone)]
pub struct ListBooksCommand;

/// Input parameters for listing books
#[derive(Debug, Clone)]
pub struct ListBooksInput {
    /// Filters to apply (default: no filters, list all)
    pub filters: BookFilters,
}

impl Default for ListBooksInput {
    fn default() -> Self {
        Self {
            filters: BookFilters::default(),
        }
    }
}

#[async_trait]
impl Command for ListBooksCommand {
    type Input = ListBooksInput;
    type Output = ListBooksResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Query database with filters
        let books = execute_books_query(pool, &input.filters).await?;

        // Convert to summary format
        let book_summaries: Vec<BookSummary> = books
            .into_iter()
            .map(|book| BookSummary {
                id: book.id,
                title: book.name,
                // Note: Authors are not included in BookResult from execute_books_query
                // This would require a separate query or an enhanced result type
                authors: vec![],
                publisher: book.publisher_name,
                year: book.publication_date.map(|ts| {
                    use chrono::{DateTime, Datelike, Utc};
                    DateTime::<Utc>::from_timestamp(ts, 0)
                        .map(|dt| dt.year())
                        .unwrap_or(1970)
                }),
                format: book.format_key,
                // Note: file_link is a path (String), not size (u64)
                // File size would need to be calculated from the file system
                file_size: None,
            })
            .collect();

        let total_count = book_summaries.len();

        Ok(ListBooksResult {
            books: book_summaries,
            total_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_input() {
        let input = ListBooksInput::default();
        assert!(input.filters.authors.is_empty());
        assert!(input.filters.publishers.is_empty());
    }
}
