//! Backward compatibility module - DEPRECATED
//!
//! This module is deprecated. Use `crate::filters::types` instead.
//!
//! ```rust,ignore
//! // Old way (deprecated):
//! use ritmo_db_core::results::BookResult;
//!
//! // New way:
//! use ritmo_db_core::filters::BookResult;
//! ```

// Re-export everything from new filters::types module for backward compatibility
pub use crate::filters::types::{BookResult, ContentResult};
