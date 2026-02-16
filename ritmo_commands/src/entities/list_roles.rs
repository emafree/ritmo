//! Command to list roles

use crate::{Command, CommandResult, ListRolesResult, RoleSummary};
use async_trait::async_trait;
use ritmo_db::models::Role;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to list all roles
///
/// This command queries the database for all roles and returns
/// structured information about each role.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{ListRolesCommand, ListRolesInput};
/// use ritmo_commands::Command;
///
/// let command = ListRolesCommand;
/// let input = ListRolesInput;
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Found {} roles", result.total_count);
/// ```
#[derive(Debug, Clone)]
pub struct ListRolesCommand;

/// Input parameters for listing roles
#[derive(Debug, Clone)]
pub struct ListRolesInput;

impl Default for ListRolesInput {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Command for ListRolesCommand {
    type Input = ListRolesInput;
    type Output = ListRolesResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        _input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Query all roles
        let roles = sqlx::query!("SELECT id, key, created_at FROM roles ORDER BY key")
            .fetch_all(pool)
            .await?;

        // Convert to summary format
        let role_summaries: Vec<RoleSummary> = roles
            .into_iter()
            .map(|role| {
                let role_obj = Role {
                    id: role.id,
                    key: role.key.clone(),
                    created_at: role.created_at,
                };
                RoleSummary {
                    id: role.id.unwrap_or(0),
                    key: role.key,
                    display_name: role_obj.display_name(),
                    created_at: Some(role.created_at),
                }
            })
            .collect();

        let total_count = role_summaries.len();

        Ok(ListRolesResult {
            roles: role_summaries,
            total_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_input() {
        let _input = ListRolesInput::default();
    }
}
