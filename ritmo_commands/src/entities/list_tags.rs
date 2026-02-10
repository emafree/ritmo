//! Command to list tags

use crate::{Command, CommandResult, ListTagsResult, TagSummary};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to list all tags
///
/// This command queries the database for all tags and returns
/// structured information about each tag.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{ListTagsCommand, ListTagsInput};
/// use ritmo_commands::Command;
///
/// let command = ListTagsCommand;
/// let input = ListTagsInput;
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Found {} tags", result.total_count);
/// ```
#[derive(Debug, Clone)]
pub struct ListTagsCommand;

/// Input parameters for listing tags
#[derive(Debug, Clone)]
pub struct ListTagsInput;

impl Default for ListTagsInput {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Command for ListTagsCommand {
    type Input = ListTagsInput;
    type Output = ListTagsResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        _input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Query all tags
        let tags = sqlx::query!("SELECT id, name, created_at FROM tags ORDER BY name")
            .fetch_all(pool)
            .await?;

        // Convert to summary format
        let tag_summaries: Vec<TagSummary> = tags
            .into_iter()
            .map(|tag| TagSummary {
                id: tag.id.unwrap_or(0),
                name: tag.name,
                created_at: Some(tag.created_at),
            })
            .collect();

        let total_count = tag_summaries.len();

        Ok(ListTagsResult {
            tags: tag_summaries,
            total_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_input() {
        let _input = ListTagsInput::default();
    }
}
