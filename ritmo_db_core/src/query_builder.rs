//! Backward compatibility module - DEPRECATED
//!
//! This module is deprecated. Use `crate::filters` instead.
//!
//! ```rust,ignore
//! // Old way (deprecated):
//! use ritmo_db_core::query_builder::execute_books_query;
//!
//! // New way:
//! use ritmo_db_core::filters::execute_books_query;
//! ```

// Re-export everything from new filters module for backward compatibility
pub use crate::filters::builder::*;
pub use crate::filters::executor::*;
