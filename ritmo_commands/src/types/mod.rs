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

// ============================================================================
// Content Results
// ============================================================================

/// Result of adding a content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddContentResult {
    pub content_id: i64,
    pub title: String,
    pub book_id: Option<i64>,
}

/// Result of listing contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListContentsResult {
    pub contents: Vec<ContentSummary>,
    pub total_count: usize,
}

/// Summary information for a single content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSummary {
    pub id: i64,
    pub title: String,
    pub authors: Vec<String>,
    pub content_type: Option<String>,
    pub year: Option<i32>,
    pub pages: Option<i64>,
}

/// Result of updating a content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateContentResult {
    pub content_id: i64,
}

/// Result of deleting a content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteContentResult {
    pub content_id: i64,
}

/// Result of linking content to book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkContentResult {
    pub content_id: i64,
    pub book_id: i64,
}

/// Result of unlinking content from book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlinkContentResult {
    pub content_id: i64,
    pub book_id: i64,
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

// ============================================================================
// Entity Results (Tags, Publishers, Series, People)
// ============================================================================

/// Result of listing tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTagsResult {
    pub tags: Vec<TagSummary>,
    pub total_count: usize,
}

/// Summary information for a single tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSummary {
    pub id: i64,
    pub name: String,
    pub created_at: Option<i64>,
}

/// Result of listing publishers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPublishersResult {
    pub publishers: Vec<PublisherSummary>,
    pub total_count: usize,
}

/// Summary information for a single publisher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherSummary {
    pub id: i64,
    pub name: String,
    pub country: Option<String>,
    pub website: Option<String>,
}

/// Result of listing series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSeriesResult {
    pub series: Vec<SeriesSummary>,
    pub total_count: usize,
}

/// Summary information for a single series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesSummary {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub total_books: Option<i64>,
    pub completed: bool,
}

/// Result of listing people
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPeopleResult {
    pub people: Vec<PersonSummary>,
    pub total_count: usize,
}

/// Summary information for a single person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonSummary {
    pub id: i64,
    pub name: String,
    pub display_name: Option<String>,
    pub verified: bool,
    pub confidence: f64,
}

// ============================================================================
// Tag CRUD Results
// ============================================================================

/// Result of creating a tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTagResult {
    pub tag_id: i64,
    pub name: String,
    pub created_at: String,
    pub message: String,
}

/// Result of updating a tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTagResult {
    pub tag_id: i64,
    pub name: String,
    pub updated_at: String,
    pub message: String,
}

/// Result of deleting a tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTagResult {
    pub tag_id: i64,
    pub name: String,
    pub deleted_at: String,
    pub books_affected: i64,
    pub warning: Option<String>,
}

// ============================================================================
// Publisher CRUD Results
// ============================================================================

/// Result of creating a publisher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePublisherResult {
    pub publisher_id: i64,
    pub name: String,
    pub created_at: String,
    pub message: String,
}

/// Result of updating a publisher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePublisherResult {
    pub publisher_id: i64,
    pub name: String,
    pub updated_at: String,
    pub message: String,
}

/// Result of deleting a publisher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePublisherResult {
    pub publisher_id: i64,
    pub name: String,
    pub deleted_at: String,
    pub books_affected: i64,
    pub warning: Option<String>,
}

// ============================================================================
// Series CRUD Results
// ============================================================================

/// Result of creating a series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSeriesResult {
    pub series_id: i64,
    pub name: String,
    pub created_at: String,
    pub message: String,
}

/// Result of updating a series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSeriesResult {
    pub series_id: i64,
    pub name: String,
    pub updated_at: String,
    pub message: String,
}

/// Result of deleting a series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSeriesResult {
    pub series_id: i64,
    pub name: String,
    pub deleted_at: String,
    pub books_affected: i64,
    pub warning: Option<String>,
}

// ============================================================================
// Person CRUD Results
// ============================================================================

/// Result of creating a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePersonResult {
    pub person_id: i64,
    pub name: String,
    pub display_name: Option<String>,
    pub created_at: String,
    pub message: String,
}

/// Result of updating a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePersonResult {
    pub person_id: i64,
    pub name: String,
    pub display_name: Option<String>,
    pub updated_at: String,
    pub message: String,
}

/// Result of deleting a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePersonResult {
    pub person_id: i64,
    pub name: String,
    pub deleted_at: String,
    pub books_affected: i64,
    pub contents_affected: i64,
    pub warning: Option<String>,
}

// ============================================================================
// Role CRUD Results
// ============================================================================

/// Result of listing roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRolesResult {
    pub roles: Vec<RoleSummary>,
    pub total_count: usize,
}

/// Summary information for a single role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleSummary {
    pub id: i64,
    pub key: String,
    pub display_name: String,
    pub created_at: Option<i64>,
}

/// Result of creating a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoleResult {
    pub role_id: i64,
    pub key: String,
    pub display_name: String,
    pub created_at: String,
    pub message: String,
}

/// Result of updating a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleResult {
    pub role_id: i64,
    pub key: String,
    pub display_name: String,
    pub updated_at: String,
    pub message: String,
}

/// Result of deleting a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRoleResult {
    pub role_id: i64,
    pub key: String,
    pub display_name: String,
    pub deleted_at: String,
    pub books_affected: i64,
    pub contents_affected: i64,
    pub warning: Option<String>,
}
