use serde::{Deserialize, Serialize};

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
            let dt = DateTime::<Utc>::from_timestamp(timestamp, 0)
                .expect("Invalid timestamp");
            dt.format("%Y-%m-%d").to_string()
        })
    }

    /// Formatta la data di creazione in formato leggibile
    pub fn formatted_created_at(&self) -> String {
        use chrono::{DateTime, Utc};
        let dt = DateTime::<Utc>::from_timestamp(self.created_at, 0)
            .expect("Invalid timestamp");
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
            let dt = DateTime::<Utc>::from_timestamp(timestamp, 0)
                .expect("Invalid timestamp");
            dt.format("%Y-%m-%d").to_string()
        })
    }

    /// Formatta la data di creazione in formato leggibile
    pub fn formatted_created_at(&self) -> String {
        use chrono::{DateTime, Utc};
        let dt = DateTime::<Utc>::from_timestamp(self.created_at, 0)
            .expect("Invalid timestamp");
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

        assert_eq!(book.formatted_publication_date(), Some("2010-01-01".to_string()));
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

        assert_eq!(content.formatted_publication_date(), Some("2010-01-01".to_string()));
        assert!(content.to_short_string().contains("Il cavaliere inesistente"));
        assert!(content.to_short_string().contains("[Romanzo]"));
    }
}
