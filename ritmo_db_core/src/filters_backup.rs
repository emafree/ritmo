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
}
