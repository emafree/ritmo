//! Structured types for command inputs and outputs
//!
//! These types provide a clean API for both CLI and GUI frontends.
//! All types are serializable for potential future use (API, IPC, etc.).

use serde::{Deserialize, Serialize};

/// Result of adding a book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddBookResult {
    pub book_id: i64,
    pub title: String,
    pub file_path: String,
    pub file_size: u64,
    pub warnings: Vec<String>,
}

/// Result of updating a book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBookResult {
    pub book_id: i64,
}

/// Result of deleting a book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteBookResult {
    pub book_id: i64,
    pub file_deleted: bool,
}

/// Result of listing books
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListBooksResult {
    pub books: Vec<BookSummary>,
    pub total_count: usize,
}

/// Summary information for a single book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookSummary {
    pub id: i64,
    pub title: String,
    pub authors: Vec<String>,
    pub publisher: Option<String>,
    pub year: Option<i32>,
    pub format: Option<String>,
    pub file_size: Option<i64>,
}

/// Result of bulk operations (update/delete)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationResult {
    pub affected_count: usize,
    pub success_ids: Vec<i64>,
    pub failed_ids: Vec<(i64, String)>, // (id, error_message)
    pub warnings: Vec<String>,
}

/// Preview information for confirmation dialogs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPreview {
    pub items: Vec<PreviewItem>,
    pub operation_type: OperationType,
    pub warnings: Vec<String>,
}

/// Single item in a preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewItem {
    pub id: i64,
    pub display_text: String,
    pub details: Vec<(String, String)>, // key-value pairs
}

/// Type of operation being previewed
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationType {
    Update,
    Delete,
    Link,
    Unlink,
}
