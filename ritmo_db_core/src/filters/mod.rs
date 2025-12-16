//! Filter system for querying books and contents
//!
//! This module provides a complete filter system for querying the database.
//! It includes:
//! - Filter types and data structures (`types`)
//! - SQL query building logic (`builder`)
//! - Query execution against the database (`executor`)
//!
//! # Architecture
//!
//! The filter system is organized into isolated, testable modules:
//!
//! ```text
//! filters/
//! ├── mod.rs        <- Public API (this file)
//! ├── types.rs      <- BookFilters, ContentFilters, BookResult, ContentResult
//! ├── builder.rs    <- SQL query construction
//! └── executor.rs   <- Query execution
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use ritmo_db_core::filters::{BookFilters, execute_books_query};
//! use ritmo_db_core::LibraryConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = LibraryConfig::new("/path/to/library");
//! let pool = config.create_pool().await?;
//!
//! // Use builder pattern for filters
//! let filters = BookFilters::default()
//!     .with_author("Calvino")
//!     .with_format("epub");
//!
//! let books = execute_books_query(&pool, &filters).await?;
//! println!("Found {} books", books.len());
//! # Ok(())
//! # }
//! ```

pub mod builder;
pub mod executor;
pub mod types;
pub mod validator;

// Re-export types for convenient access
pub use builder::{build_books_query, build_contents_query};
pub use executor::{execute_books_query, execute_contents_query};
pub use types::{
    BookFilters, BookResult, BookSortField, ContentFilters, ContentResult, ContentSortField,
};
pub use validator::{validate_book_filters, validate_content_filters, ValidationError};
