//! Command to list publishers

use crate::{Command, CommandResult, ListPublishersResult, PublisherSummary};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to list all publishers
///
/// This command queries the database for all publishers and returns
/// structured information about each publisher.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{ListPublishersCommand, ListPublishersInput};
/// use ritmo_commands::Command;
///
/// let command = ListPublishersCommand;
/// let input = ListPublishersInput;
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Found {} publishers", result.total_count);
/// ```
#[derive(Debug, Clone)]
pub struct ListPublishersCommand;

/// Input parameters for listing publishers
#[derive(Debug, Clone)]
pub struct ListPublishersInput;

impl Default for ListPublishersInput {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Command for ListPublishersCommand {
    type Input = ListPublishersInput;
    type Output = ListPublishersResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        _input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Query all publishers
        let publishers = sqlx::query!(
            "SELECT id, name, country, website FROM publishers ORDER BY name"
        )
        .fetch_all(pool)
        .await?;

        // Convert to summary format
        let publisher_summaries: Vec<PublisherSummary> = publishers
            .into_iter()
            .map(|pub_| PublisherSummary {
                id: pub_.id,
                name: pub_.name,
                country: pub_.country,
                website: pub_.website,
            })
            .collect();

        let total_count = publisher_summaries.len();

        Ok(ListPublishersResult {
            publishers: publisher_summaries,
            total_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_input() {
        let _input = ListPublishersInput::default();
    }
}
