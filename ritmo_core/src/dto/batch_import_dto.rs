use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root structure for batch import JSON
/// Array of ImportObject
pub type BatchImportInput = Vec<ImportObject>;

/// A single import object representing one physical book file to import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportObject {
    /// Path to the book file (absolute or relative)
    pub file_path: String,

    /// Book-level metadata (physical edition)
    pub book: BookInput,

    /// Contents contained in this book (literary works)
    #[serde(default)]
    pub contents: Vec<ContentInput>,

    /// Confidence scores for extracted fields (Level 3 output, ignored on import)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<HashMap<String, f32>>,
}

/// Book-level metadata (physical edition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookInput {
    /// Book title (required)
    pub title: String,

    /// Original title if different
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_title: Option<String>,

    /// Book-level contributors (editors, preface writers, etc.)
    #[serde(default)]
    pub people: Vec<PersonInput>,

    /// Publisher name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    /// Publication year of this edition
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,

    /// ISBN identifier
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isbn: Option<String>,

    /// File format (auto-detected if omitted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// Series name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,

    /// Position in series
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_index: Option<i64>,

    /// Page count
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pages: Option<i64>,

    /// Free-text notes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Content-level metadata (literary work)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentInput {
    /// Content title (required)
    pub title: String,

    /// Original title if different
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_title: Option<String>,

    /// Content creators (authors, translators, etc.)
    #[serde(default)]
    pub people: Vec<PersonInput>,

    /// Content type (i18n key: "type.novel", "type.short_story", etc.)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "type")]
    pub content_type: Option<String>,

    /// Original publication year of this work
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,

    /// Languages
    #[serde(default)]
    pub languages: Vec<LanguageInput>,
}

/// Person object (contributor)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonInput {
    /// Person name (required)
    pub name: String,

    /// Role i18n key (required): "role.author", "role.translator", etc.
    pub role: String,
}

/// Language object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInput {
    /// ISO 639-1 language code (required): "en", "it", "fr", etc.
    pub code: String,

    /// Role i18n key (required): "language_role.original", "language_role.actual", etc.
    pub role: String,
}
