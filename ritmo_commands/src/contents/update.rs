//! Command to update a single content's metadata

use crate::{Command, CommandResult, UpdateContentResult};
use async_trait::async_trait;
use ritmo_core::service::{update_content, ContentUpdateMetadata};
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to update a content
///
/// This command updates a single content's metadata in the database.
/// It uses the content ID to identify which content to update.
#[derive(Debug, Clone)]
pub struct UpdateContentCommand;

/// Input parameters for updating a content
///
/// All fields are optional - only specified fields will be updated.
/// Use None to leave a field unchanged.
#[derive(Debug, Clone, Default)]
pub struct UpdateContentInput {
    /// Content ID (required)
    pub content_id: i64,

    /// New title (optional)
    pub title: Option<String>,

    /// New original title (optional)
    pub original_title: Option<String>,

    /// New people: Vec<(name, role)> (optional)
    pub people: Option<Vec<(String, String)>>,

    /// New content type (optional)
    pub content_type: Option<String>,

    /// New publication year (optional)
    pub year: Option<i32>,

    /// New notes (optional)
    pub notes: Option<String>,

    /// New page count (optional)
    pub pages: Option<i64>,

    /// New tags (optional)
    pub tags: Option<Vec<String>>,

    /// New languages: Vec<(name, iso2, iso3, role)> (optional)
    pub languages: Option<Vec<(String, String, String, String)>>,
}

#[async_trait]
impl Command for UpdateContentCommand {
    type Input = UpdateContentInput;
    type Output = UpdateContentResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Prepare metadata for update service
        let metadata = ContentUpdateMetadata {
            title: input.title,
            original_title: input.original_title,
            people: input.people,
            content_type: input.content_type,
            year: input.year,
            notes: input.notes,
            pages: input.pages,
            tags: input.tags,
            languages: input.languages,
        };

        // Execute update
        update_content(pool, input.content_id, metadata).await?;

        // Return structured result
        Ok(UpdateContentResult {
            content_id: input.content_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_input() {
        let input = UpdateContentInput::default();
        assert_eq!(input.content_id, 0);
        assert!(input.title.is_none());
        assert!(input.content_type.is_none());
    }
}
