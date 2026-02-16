//! Command to delete a role

use crate::{Command, CommandResult, DeleteRoleResult};
use async_trait::async_trait;
use ritmo_db::models::Role;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;

/// Command to delete a role
///
/// This command deletes a role from the database.
/// It also removes all associations from books and contents.
///
/// # Example
///
/// ```ignore
/// use ritmo_commands::entities::{DeleteRoleCommand, DeleteRoleInput};
/// use ritmo_commands::Command;
///
/// let command = DeleteRoleCommand;
/// let input = DeleteRoleInput {
///     role_id: 1,
/// };
///
/// let result = command.execute(&config, &pool, input).await?;
/// println!("Role deleted: {}", result.role_id);
/// ```
#[derive(Debug, Clone)]
pub struct DeleteRoleCommand;

/// Input parameters for deleting a role
#[derive(Debug, Clone)]
pub struct DeleteRoleInput {
    /// Role ID to delete
    pub role_id: i64,
}

#[async_trait]
impl Command for DeleteRoleCommand {
    type Input = DeleteRoleInput;
    type Output = DeleteRoleResult;

    async fn execute(
        &self,
        _config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Check if role exists
        let existing = sqlx::query!("SELECT id, key, created_at FROM roles WHERE id = ?", input.role_id)
            .fetch_optional(pool)
            .await?;

        if existing.is_none() {
            return Err(crate::CommandError::NotFound(format!(
                "Role with ID {} not found",
                input.role_id
            )));
        }

        let role_data = existing.unwrap();
        let role = Role {
            id: Some(role_data.id),
            key: role_data.key.clone(),
            created_at: role_data.created_at,
        };

        // Count affected books and contents
        let books_affected = sqlx::query_scalar!(
            "SELECT COUNT(DISTINCT book_id) FROM x_books_people_roles WHERE role_id = ?",
            input.role_id
        )
        .fetch_one(pool)
        .await?;

        let contents_affected = sqlx::query_scalar!(
            "SELECT COUNT(DISTINCT content_id) FROM x_contents_people_roles WHERE role_id = ?",
            input.role_id
        )
        .fetch_one(pool)
        .await?;

        // Delete role associations from books
        sqlx::query!(
            "DELETE FROM x_books_people_roles WHERE role_id = ?",
            input.role_id
        )
        .execute(pool)
        .await?;

        // Delete role associations from contents
        sqlx::query!(
            "DELETE FROM x_contents_people_roles WHERE role_id = ?",
            input.role_id
        )
        .execute(pool)
        .await?;

        // Delete the role itself
        sqlx::query!("DELETE FROM roles WHERE id = ?", input.role_id)
            .execute(pool)
            .await?;

        let deleted_at = chrono::Utc::now().to_rfc3339();

        let warning = if books_affected > 0 || contents_affected > 0 {
            Some(format!(
                "Removed role associations from {} book(s) and {} content(s)",
                books_affected, contents_affected
            ))
        } else {
            None
        };

        Ok(DeleteRoleResult {
            role_id: input.role_id,
            key: role_data.key.clone(),
            display_name: role.display_name(),
            deleted_at,
            books_affected,
            contents_affected,
            warning,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_creation() {
        let input = DeleteRoleInput {
            role_id: 1,
        };
        assert_eq!(input.role_id, 1);
    }
}
