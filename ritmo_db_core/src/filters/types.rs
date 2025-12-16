//! Filter types and data structures
//!
//! This module contains all filter-related types used for querying books and contents.

use serde::{Deserialize, Serialize};

/// Filtri per la ricerca di libri
#[derive(Debug, Clone, Default)]
pub struct BookFilters {
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub series: Option<String>,
    pub format: Option<String>,
    pub year: Option<i32>,
    pub isbn: Option<String>,
    pub search: Option<String>,
    pub acquired_after: Option<i64>, // Timestamp UNIX: libri acquisiti dopo questa data
    pub acquired_before: Option<i64>, // Timestamp UNIX: libri acquisiti prima di questa data
    pub sort: BookSortField,
    pub limit: Option<i64>,
    pub offset: i64,
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
#[derive(Debug, Clone, Default)]
pub struct ContentFilters {
    pub author: Option<String>,
    pub content_type: Option<String>,
    pub year: Option<i32>,
    pub search: Option<String>,
    pub sort: ContentSortField,
    pub limit: Option<i64>,
    pub offset: i64,
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
