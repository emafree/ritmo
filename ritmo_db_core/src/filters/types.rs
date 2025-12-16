//! Filter types and data structures
//!
//! This module contains all filter-related types used for querying books and contents.

use serde::{Deserialize, Serialize};

/// Filtri per la ricerca di libri
///
/// Supports both single and multiple values for certain filters.
/// Multiple values use OR logic within the same filter type.
/// Different filter types use AND logic.
///
/// Example:
/// ```ignore
/// BookFilters {
///     authors: vec!["King".to_string(), "Tolkien".to_string()],
///     formats: vec!["epub".to_string()],
///     year: Some(2020),
///     ..Default::default()
/// }
/// // SQL: (author LIKE '%King%' OR author LIKE '%Tolkien%') AND format LIKE '%epub%' AND year = 2020
/// ```
#[derive(Debug, Clone, Default)]
pub struct BookFilters {
    /// Authors (OR logic if multiple)
    pub authors: Vec<String>,
    /// Publishers (OR logic if multiple)
    pub publishers: Vec<String>,
    /// Series (OR logic if multiple)
    pub series_list: Vec<String>,
    /// Formats (OR logic if multiple)
    pub formats: Vec<String>,
    /// Publication year (exact match)
    pub year: Option<i32>,
    /// ISBN search pattern
    pub isbn: Option<String>,
    /// Full-text search
    pub search: Option<String>,
    /// Acquisition date filters
    pub acquired_after: Option<i64>, // Timestamp UNIX: libri acquisiti dopo questa data
    pub acquired_before: Option<i64>, // Timestamp UNIX: libri acquisiti prima di questa data
    /// Sort configuration
    pub sort: BookSortField,
    /// Result pagination
    pub limit: Option<i64>,
    pub offset: i64,
}

impl BookFilters {
    /// Helper to add a single author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.authors.push(author.into());
        self
    }

    /// Helper to add multiple authors
    pub fn with_authors(mut self, authors: Vec<String>) -> Self {
        self.authors = authors;
        self
    }

    /// Helper to add a single publisher
    pub fn with_publisher(mut self, publisher: impl Into<String>) -> Self {
        self.publishers.push(publisher.into());
        self
    }

    /// Helper to add a single format
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.formats.push(format.into());
        self
    }

    /// Helper to add a single series
    pub fn with_series(mut self, series: impl Into<String>) -> Self {
        self.series_list.push(series.into());
        self
    }

    /// Backward compatibility: set author from Option<String>
    pub fn set_author_opt(mut self, author: Option<String>) -> Self {
        if let Some(a) = author {
            self.authors = vec![a];
        }
        self
    }

    /// Backward compatibility: set publisher from Option<String>
    pub fn set_publisher_opt(mut self, publisher: Option<String>) -> Self {
        if let Some(p) = publisher {
            self.publishers = vec![p];
        }
        self
    }

    /// Backward compatibility: set series from Option<String>
    pub fn set_series_opt(mut self, series: Option<String>) -> Self {
        if let Some(s) = series {
            self.series_list = vec![s];
        }
        self
    }

    /// Backward compatibility: set format from Option<String>
    pub fn set_format_opt(mut self, format: Option<String>) -> Self {
        if let Some(f) = format {
            self.formats = vec![f];
        }
        self
    }
}

/// Campi per ordinamento libri
#[derive(Debug, Clone, Default)]
pub enum BookSortField {
    #[default]
    Title,
    Author,
    Year,
    DateAdded,
}

impl BookSortField {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "author" => Self::Author,
            "year" => Self::Year,
            "date_added" => Self::DateAdded,
            _ => Self::Title,
        }
    }

    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::Title => "books.name",
            Self::Author => "people.name",
            Self::Year => "books.publication_date",
            Self::DateAdded => "books.created_at",
        }
    }
}

/// Filtri per la ricerca di contenuti
///
/// Supports multiple values with OR logic for authors and content types.
#[derive(Debug, Clone, Default)]
pub struct ContentFilters {
    /// Authors (OR logic if multiple)
    pub authors: Vec<String>,
    /// Content types (OR logic if multiple)
    pub content_types: Vec<String>,
    /// Publication year (exact match)
    pub year: Option<i32>,
    /// Full-text search
    pub search: Option<String>,
    /// Sort configuration
    pub sort: ContentSortField,
    /// Result pagination
    pub limit: Option<i64>,
    pub offset: i64,
}

impl ContentFilters {
    /// Helper to add a single author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.authors.push(author.into());
        self
    }

    /// Helper to add a single content type
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_types.push(content_type.into());
        self
    }

    /// Backward compatibility: set author from Option<String>
    pub fn set_author_opt(mut self, author: Option<String>) -> Self {
        if let Some(a) = author {
            self.authors = vec![a];
        }
        self
    }

    /// Backward compatibility: set content_type from Option<String>
    pub fn set_content_type_opt(mut self, content_type: Option<String>) -> Self {
        if let Some(ct) = content_type {
            self.content_types = vec![ct];
        }
        self
    }
}

/// Campi per ordinamento contenuti
#[derive(Debug, Clone, Default)]
pub enum ContentSortField {
    #[default]
    Title,
    Author,
    Year,
    Type,
}

impl ContentSortField {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "author" => Self::Author,
            "year" => Self::Year,
            "type" => Self::Type,
            _ => Self::Title,
        }
    }

    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::Title => "contents.name",
            Self::Author => "people.name",
            Self::Year => "contents.publication_date",
            Self::Type => "types.name",
        }
    }
}

/// Risultato di una query per libri
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookResult {
    pub id: i64,
    pub name: String,
    pub original_title: Option<String>,
    pub publisher_name: Option<String>,
    pub format_name: Option<String>,
    pub series_name: Option<String>,
    pub series_index: Option<i64>,
    pub publication_date: Option<i64>,
    pub isbn: Option<String>,
    pub pages: Option<i64>,
    pub file_link: Option<String>,
    pub created_at: i64,
}

impl BookResult {
    /// Formatta la data di pubblicazione in formato leggibile
    pub fn formatted_publication_date(&self) -> Option<String> {
        self.publication_date.map(|timestamp| {
            use chrono::{DateTime, Utc};
            let dt = DateTime::<Utc>::from_timestamp(timestamp, 0).expect("Invalid timestamp");
            dt.format("%Y-%m-%d").to_string()
        })
    }

    /// Formatta la data di creazione in formato leggibile
    pub fn formatted_created_at(&self) -> String {
        use chrono::{DateTime, Utc};
        let dt = DateTime::<Utc>::from_timestamp(self.created_at, 0).expect("Invalid timestamp");
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Restituisce una rappresentazione breve per listing
    pub fn to_short_string(&self) -> String {
        let mut parts = vec![self.name.clone()];

        if let Some(author) = &self.publisher_name {
            parts.push(format!("({})", author));
        }

        if let Some(year) = self.formatted_publication_date() {
            parts.push(year);
        }

        parts.join(" ")
    }
}

/// Risultato di una query per contenuti
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContentResult {
    pub id: i64,
    pub name: String,
    pub original_title: Option<String>,
    pub type_name: Option<String>,
    pub publication_date: Option<i64>,
    pub pages: Option<i64>,
    pub created_at: i64,
}

impl ContentResult {
    /// Formatta la data di pubblicazione in formato leggibile
    pub fn formatted_publication_date(&self) -> Option<String> {
        self.publication_date.map(|timestamp| {
            use chrono::{DateTime, Utc};
            let dt = DateTime::<Utc>::from_timestamp(timestamp, 0).expect("Invalid timestamp");
            dt.format("%Y-%m-%d").to_string()
        })
    }

    /// Formatta la data di creazione in formato leggibile
    pub fn formatted_created_at(&self) -> String {
        use chrono::{DateTime, Utc};
        let dt = DateTime::<Utc>::from_timestamp(self.created_at, 0).expect("Invalid timestamp");
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Restituisce una rappresentazione breve per listing
    pub fn to_short_string(&self) -> String {
        let mut parts = vec![self.name.clone()];

        if let Some(content_type) = &self.type_name {
            parts.push(format!("[{}]", content_type));
        }

        if let Some(year) = self.formatted_publication_date() {
            parts.push(year);
        }

        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_sort_field() {
        assert!(matches!(
            BookSortField::from_str("title"),
            BookSortField::Title
        ));
        assert!(matches!(
            BookSortField::from_str("author"),
            BookSortField::Author
        ));
        assert!(matches!(
            BookSortField::from_str("year"),
            BookSortField::Year
        ));
        assert!(matches!(
            BookSortField::from_str("date_added"),
            BookSortField::DateAdded
        ));
        assert!(matches!(
            BookSortField::from_str("invalid"),
            BookSortField::Title
        ));
    }

    #[test]
    fn test_content_sort_field() {
        assert!(matches!(
            ContentSortField::from_str("title"),
            ContentSortField::Title
        ));
        assert!(matches!(
            ContentSortField::from_str("author"),
            ContentSortField::Author
        ));
        assert!(matches!(
            ContentSortField::from_str("year"),
            ContentSortField::Year
        ));
        assert!(matches!(
            ContentSortField::from_str("type"),
            ContentSortField::Type
        ));
    }

    #[test]
    fn test_book_result_formatting() {
        let book = BookResult {
            id: 1,
            name: "Il barone rampante".to_string(),
            original_title: Some("Il barone rampante".to_string()),
            publisher_name: Some("Einaudi".to_string()),
            format_name: Some("EPUB".to_string()),
            series_name: Some("I nostri antenati".to_string()),
            series_index: Some(2),
            publication_date: Some(1262304000), // 2010-01-01
            isbn: Some("978-88-06-20000-0".to_string()),
            pages: Some(320),
            file_link: Some("/path/to/book.epub".to_string()),
            created_at: 1609459200, // 2021-01-01
        };

        assert_eq!(
            book.formatted_publication_date(),
            Some("2010-01-01".to_string())
        );
        assert!(book.to_short_string().contains("Il barone rampante"));
        assert!(book.to_short_string().contains("(Einaudi)"));
    }

    #[test]
    fn test_content_result_formatting() {
        let content = ContentResult {
            id: 1,
            name: "Il cavaliere inesistente".to_string(),
            original_title: None,
            type_name: Some("Romanzo".to_string()),
            publication_date: Some(1262304000), // 2010-01-01
            pages: Some(250),
            created_at: 1609459200,
        };

        assert_eq!(
            content.formatted_publication_date(),
            Some("2010-01-01".to_string())
        );
        assert!(content
            .to_short_string()
            .contains("Il cavaliere inesistente"));
        assert!(content.to_short_string().contains("[Romanzo]"));
    }
}
