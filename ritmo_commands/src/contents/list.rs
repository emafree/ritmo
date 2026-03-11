//! Command to list contents with optional filters

use crate::{Command, CommandResult, ContentSummary, ListContentsResult};
use async_trait::async_trait;
use ritmo_db_core::{execute_contents_query, ContentFilters, LibraryConfig};
use sqlx::SqlitePool;

/// Command to list contents
///
/// This command queries the database for contents matching optional filters
/// and returns structured information about each content.
#[derive(Debug, Clone)]
pub struct ListContentsCommand;

/// Input parameters for listing contents
#[derive(Debug, Clone)]
pub struct ListContentsInput {
    /// Filters to apply (default: no filters, list all)
    pub filters: ContentFilters,
}

impl Default for ListContentsInput {
    fn default() -> Self {
        Self {
            filters: ContentFilters::default(),
        }
    }
}

#[async_trait]
impl Command for ListContentsCommand {
    type Input = ListContentsInput;
    type Output = ListContentsResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Query database with filters
        let contents = execute_contents_query(pool, &input.filters).await?;

        // Convert to summary format
        let content_summaries: Vec<ContentSummary> = contents
            .into_iter()
            .map(|content| ContentSummary {
                id: content.id,
                title: content.name,
                // Note: Authors are not included in ContentResult from execute_contents_query
                // This would require a separate query or an enhanced result type
                authors: vec![],
                content_type: content.type_key,
                year: content.publication_date.map(|ts| {
                    use chrono::{DateTime, Datelike, Utc};
                    DateTime::<Utc>::from_timestamp(ts, 0)
                        .map(|dt| dt.year())
                        .unwrap_or(1970)
                }),
            })
            .collect();

        let total_count = content_summaries.len();

        Ok(ListContentsResult {
            contents: content_summaries,
            total_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_input() {
        let input = ListContentsInput::default();
        assert!(input.filters.authors.is_empty());
        assert!(input.filters.content_types.is_empty());
    }
}
