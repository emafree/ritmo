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
    /// Must contain at least one author
    pub people: Option<Vec<(String, String)>>,

    /// Content type (e.g., "type.novel", "type.short_story")
    pub content_type: Option<String>,

    /// Genre (e.g., "Science Fiction", "Mystery")
    pub genre: Option<String>,

    /// Publication year
    pub year: Option<i32>,

    /// Additional notes
    pub notes: Option<String>,

    /// Tags
    pub tags: Option<Vec<String>>,

    /// Languages: Vec<(name, iso2, iso3, role)>
    /// Example: vec![("Italian".to_string(), "it".to_string(), "ita".to_string(), "language_role.original".to_string())]
    pub languages: Option<Vec<(String, String, String, String)>>,

    /// Book ID to associate with (required)
    pub book_id: i64,
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

        // Validate that at least one author is present
        let has_author = input.people.as_ref().map_or(false, |people| {
            people.iter().any(|(_, role)| {
                role.to_lowercase().contains("author") || role.to_lowercase().contains("autore")
            })
        });

        if !has_author {
            return Err(crate::CommandError::Validation(
                "At least one author must be specified in --people".to_string()
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
            pages: None, // Pages option removed as per requirement
            tags: input.tags,
            languages: input.languages,
            book_id: Some(input.book_id),
        };

        // Execute creation
        let content_id = create_content(pool, metadata).await?;

        // Return structured result
        Ok(AddContentResult {
            content_id,
            title: input.title,
            book_id: Some(input.book_id),
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
            people: Some(vec![("John Doe".to_string(), "author".to_string())]),
            content_type: None,
            genre: None,
            year: None,
            notes: None,
            tags: None,
            languages: None,
            book_id: 1,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_missing_author() {
        let command = AddContentCommand;
        let input = AddContentInput {
            title: "Test Content".to_string(),
            original_title: None,
            people: Some(vec![("John Doe".to_string(), "editor".to_string())]),
            content_type: None,
            genre: None,
            year: None,
            notes: None,
            tags: None,
            languages: None,
            book_id: 1,
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::CommandError::Validation(_)));
    }

    #[test]
    fn test_validation_with_author() {
        let command = AddContentCommand;
        let input = AddContentInput {
            title: "Test Content".to_string(),
            original_title: None,
            people: Some(vec![("John Doe".to_string(), "author".to_string())]),
            content_type: None,
            genre: None,
            year: None,
            notes: None,
            tags: None,
            languages: None,
            book_id: 1,
        };

        let result = command.validate(&input);
        assert!(result.is_ok());
    }
}
