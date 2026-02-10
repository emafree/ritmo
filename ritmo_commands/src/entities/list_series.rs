//! Command to list series

use crate::{Command, CommandResult, ListSeriesResult, SeriesSummary};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to list all series
///
/// This command queries the database for all series and returns
/// structured information about each series.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{ListSeriesCommand, ListSeriesInput};
/// use ritmo_commands::Command;
///
/// let command = ListSeriesCommand;
/// let input = ListSeriesInput;
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Found {} series", result.total_count);
/// ```
#[derive(Debug, Clone)]
pub struct ListSeriesCommand;

/// Input parameters for listing series
#[derive(Debug, Clone)]
pub struct ListSeriesInput;

impl Default for ListSeriesInput {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Command for ListSeriesCommand {
    type Input = ListSeriesInput;
    type Output = ListSeriesResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        _input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Query all series
        let series_list = sqlx::query!(
            "SELECT id, name, description, total_books, completed FROM series ORDER BY name"
        )
        .fetch_all(pool)
        .await?;

        // Convert to summary format
        let series_summaries: Vec<SeriesSummary> = series_list
            .into_iter()
            .map(|series| SeriesSummary {
                id: series.id,
                name: series.name,
                description: series.description,
                total_books: series.total_books,
                completed: series.completed == 1,
            })
            .collect();

        let total_count = series_summaries.len();

        Ok(ListSeriesResult {
            series: series_summaries,
            total_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_input() {
        let _input = ListSeriesInput::default();
    }
}
