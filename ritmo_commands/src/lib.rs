//! Command Layer for Ritmo
//!
//! This crate provides a clean separation between business logic and presentation.
//! Commands encapsulate operations with structured inputs/outputs that can be used
//! by both CLI and GUI frontends.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐         ┌─────────────┐
//! │   CLI/GUI   │ ──────▶ │   Command   │ ──────▶ Services
//! │ Presentation│         │   (Logic)   │         (ritmo_core)
//! └─────────────┘         └─────────────┘
//! ```
//!
//! # Example
//!
//! ```ignore
//! use ritmo_commands::books::{AddBookCommand, AddBookInput};
//! use ritmo_commands::Command;
//!
//! let command = AddBookCommand;
//! let input = AddBookInput {
//!     file_path: PathBuf::from("book.epub"),
//!     title: "My Book".to_string(),
//!     // ...
//! };
//!
//! let result = command.execute(&config, &pool, input).await?;
//! println!("Book added with ID: {}", result.book_id);
//! ```

use async_trait::async_trait;
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;
use std::fmt::Debug;

pub mod books;
pub mod contents;
pub mod types;

pub use types::*;

/// Base trait for all commands
///
/// Commands are pure business logic with no UI/presentation concerns.
/// They accept typed inputs, execute operations, and return structured results.
///
/// # Type Parameters
///
/// - `Input`: The input parameters for this command
/// - `Output`: The structured result returned by this command
///
/// # Implementation Requirements
///
/// Commands should:
/// - Be stateless (no mutable state)
/// - Not depend on UI frameworks
/// - Return structured, serializable results
/// - Handle all errors gracefully
/// - Provide clear validation
#[async_trait]
pub trait Command: Send + Sync + Debug {
    /// Input parameters for this command
    type Input: Send + Sync;

    /// Output result of this command
    type Output: Send + Sync;

    /// Execute the command
    ///
    /// # Arguments
    ///
    /// * `config` - Library configuration
    /// * `pool` - Database connection pool
    /// * `input` - Command-specific input parameters
    ///
    /// # Returns
    ///
    /// * `Ok(Output)` - Structured result on success
    /// * `Err(CommandError)` - Error details on failure
    async fn execute(
        &self,
        config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> Result<Self::Output, CommandError>;

    /// Validate input before execution (optional)
    ///
    /// Override this method to provide input validation.
    /// Called automatically before execute() if implemented.
    fn validate(&self, _input: &Self::Input) -> Result<(), CommandError> {
        Ok(())
    }
}

/// Generic error type for commands
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Ritmo error: {0}")]
    Ritmo(#[from] ritmo_errors::RitmoErr),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

/// Convenience type alias for command results
pub type CommandResult<T> = Result<T, CommandError>;
