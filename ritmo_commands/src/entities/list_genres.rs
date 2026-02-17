//! Command to list genres

use crate::{Command, CommandResult, ListGenresResult, GenreSummary};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to list all genres
///
/// This command queries the database for all genres and returns
/// structured information about each genre.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{ListGenresCommand, ListGenresInput};
/// use ritmo_commands::Command;
///
/// let command = ListGenresCommand;
/// let input = ListGenresInput;
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Found {} genres", result.total_count);
/// ```
#[derive(Debug, Clone)]
pub struct ListGenresCommand;

/// Input parameters for listing genres
#[derive(Debug, Clone)]
pub struct ListGenresInput;

impl Default for ListGenresInput {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Command for ListGenresCommand {
    type Input = ListGenresInput;
    type Output = ListGenresResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        _input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Query all genres
        let genres = sqlx::query!("SELECT id, name, description, created_at FROM genres ORDER BY name")
            .fetch_all(pool)
            .await?;

        // Convert to summary format
        let genre_summaries: Vec<GenreSummary> = genres
            .into_iter()
            .map(|genre| GenreSummary {
                id: genre.id.unwrap_or(0),
                name: genre.name,
                description: genre.description,
                created_at: Some(genre.created_at),
            })
            .collect();

        let total_count = genre_summaries.len();

        Ok(ListGenresResult {
            genres: genre_summaries,
            total_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_input() {
        let _input = ListGenresInput::default();
    }
}
