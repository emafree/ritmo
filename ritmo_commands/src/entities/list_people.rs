//! Command to list people

use crate::{Command, CommandResult, ListPeopleResult, PersonSummary};
use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to list all people
///
/// This command queries the database for all people (authors, translators, etc.)
/// and returns structured information about each person.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{ListPeopleCommand, ListPeopleInput};
/// use ritmo_commands::Command;
///
/// let command = ListPeopleCommand;
/// let input = ListPeopleInput;
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Found {} people", result.total_count);
/// ```
#[derive(Debug, Clone)]
pub struct ListPeopleCommand;

/// Input parameters for listing people
#[derive(Debug, Clone)]
pub struct ListPeopleInput;

impl Default for ListPeopleInput {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Command for ListPeopleCommand {
    type Input = ListPeopleInput;
    type Output = ListPeopleResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        _input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Query all people
        let people = sqlx::query!(
            "SELECT id, name, display_name, verified, confidence FROM people ORDER BY name"
        )
        .fetch_all(pool)
        .await?;

        // Convert to summary format
        let people_summaries: Vec<PersonSummary> = people
            .into_iter()
            .map(|person| PersonSummary {
                id: person.id,
                name: person.name,
                display_name: person.display_name,
                verified: person.verified == 1,
                confidence: person.confidence,
            })
            .collect();

        let total_count = people_summaries.len();

        Ok(ListPeopleResult {
            people: people_summaries,
            total_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_input() {
        let _input = ListPeopleInput::default();
    }
}
