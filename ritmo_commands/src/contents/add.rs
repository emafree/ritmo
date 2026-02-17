//! Command to add a new content

use crate::{Command, CommandResult, AddContentResult};
use async_trait::async_trait;
use ritmo_core::service::{create_content, ContentCreateMetadata};
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to create a new content
///
/// This command creates a new content with metadata and optionally
/// associates it with a book.
#[derive(Debug, Clone)]
pub struct AddContentCommand;

/// Input parameters for creating a content
#[derive(Debug, Clone)]
pub struct AddContentInput {
    /// Content title (required)
    pub title: String,

    /// Original title (if translated)
    pub original_title: Option<String>,

    /// People associated with the content: Vec<(name, role)>
    pub people: Option<Vec<(String, String)>>,

    /// Content type (e.g., "type.novel", "type.short_story")
    pub content_type: Option<String>,

    /// Genre (e.g., "Science Fiction", "Mystery")
    pub genre: Option<String>,

    /// Publication year
    pub year: Option<i32>,

    /// Additional notes
    pub notes: Option<String>,

    /// Number of pages
    pub pages: Option<i64>,

    /// Tags
    pub tags: Option<Vec<String>>,

    /// Languages: Vec<(name, iso2, iso3, role)>
    /// Example: vec![("Italian".to_string(), "it".to_string(), "ita".to_string(), "language_role.original".to_string())]
    pub languages: Option<Vec<(String, String, String, String)>>,

    /// Optional book ID to associate with
    pub book_id: Option<i64>,
}

#[async_trait]
impl Command for AddContentCommand {
    type Input = AddContentInput;
    type Output = AddContentResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate title
        if input.title.trim().is_empty() {
            return Err(crate::CommandError::Validation(
                "Title cannot be empty".to_string()
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

        // Prepare metadata for create service
        let metadata = ContentCreateMetadata {
            title: input.title.clone(),
            original_title: input.original_title,
            people: input.people,
            content_type: input.content_type,
            genre: input.genre,
            year: input.year,
            notes: input.notes,
            pages: input.pages,
            tags: input.tags,
            languages: input.languages,
            book_id: input.book_id,
        };

        // Execute creation
        let content_id = create_content(pool, metadata).await?;

        // Return structured result
        Ok(AddContentResult {
            content_id,
            title: input.title,
            book_id: input.book_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_empty_title() {
        let command = AddContentCommand;
        let input = AddContentInput {
            title: "".to_string(),
            original_title: None,
            people: None,
            content_type: None,
            year: None,
            notes: None,
            pages: None,
            tags: None,
            languages: None,
            book_id: None,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }
}
