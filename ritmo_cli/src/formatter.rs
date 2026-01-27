use ritmo_db_core::{BookResult, ContentResult};
use serde_json;

/// Formato di output
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Table,
    Json,
    Simple,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => Self::Json,
            "simple" => Self::Simple,
            _ => Self::Table,
        }
    }
}

/// Formatta i risultati dei libri
pub fn format_books(books: &[BookResult], format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => format_books_json(books),
        OutputFormat::Table => format_books_table(books),
        OutputFormat::Simple => format_books_simple(books),
    }
}

/// Formatta i risultati dei contenuti
pub fn format_contents(contents: &[ContentResult], format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => format_contents_json(contents),
        OutputFormat::Table => format_contents_table(contents),
        OutputFormat::Simple => format_contents_simple(contents),
    }
}

fn format_books_json(books: &[BookResult]) -> String {
    serde_json::to_string_pretty(books).unwrap_or_else(|e| format!("Errore JSON: {}", e))
}

fn format_books_table(books: &[BookResult]) -> String {
    if books.is_empty() {
        return "Nessun libro trovato.".to_string();
    }

    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "{:<5} {:<40} {:<20} {:<15} {:<10}\n",
        "ID", "Titolo", "Editore", "Formato", "Anno"
    ));
    output.push_str(&"-".repeat(95));
    output.push('\n');

    // Rows
    for book in books {
        let title = truncate(&book.name, 38);
        let publisher = truncate(&book.publisher_name.clone().unwrap_or_default(), 18);
        let format = book.format_key.clone().unwrap_or_default();
        let year = book
            .formatted_publication_date()
            .and_then(|d| d.split('-').next().map(String::from))
            .unwrap_or_default();

        output.push_str(&format!(
            "{:<5} {:<40} {:<20} {:<15} {:<10}\n",
            book.id, title, publisher, format, year
        ));
    }

    output.push_str(&format!("\nTotale: {} libri", books.len()));
    output
}

fn format_books_simple(books: &[BookResult]) -> String {
    if books.is_empty() {
        return "Nessun libro trovato.".to_string();
    }

    let mut output = String::new();

    for book in books {
        output.push_str(&format!("• {} ", book.name));

        if let Some(publisher) = &book.publisher_name {
            output.push_str(&format!("({}) ", publisher));
        }

        if let Some(year) = book.formatted_publication_date() {
            output.push_str(&format!("[{}] ", year.split('-').next().unwrap_or("")));
        }

        if let Some(format) = &book.format_key {
            output.push_str(&format!("- {} ", format));
        }

        output.push('\n');
    }

    output.push_str(&format!("\nTotale: {} libri", books.len()));
    output
}

fn format_contents_json(contents: &[ContentResult]) -> String {
    serde_json::to_string_pretty(contents).unwrap_or_else(|e| format!("Errore JSON: {}", e))
}

fn format_contents_table(contents: &[ContentResult]) -> String {
    if contents.is_empty() {
        return "Nessun contenuto trovato.".to_string();
    }

    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "{:<5} {:<45} {:<20} {:<10}\n",
        "ID", "Titolo", "Tipo", "Anno"
    ));
    output.push_str(&"-".repeat(85));
    output.push('\n');

    // Rows
    for content in contents {
        let title = truncate(&content.name, 43);
        let content_type = truncate(&content.type_key.clone().unwrap_or_default(), 18);
        let year = content
            .formatted_publication_date()
            .and_then(|d| d.split('-').next().map(String::from))
            .unwrap_or_default();

        output.push_str(&format!(
            "{:<5} {:<45} {:<20} {:<10}\n",
            content.id, title, content_type, year
        ));
    }

    output.push_str(&format!("\nTotale: {} contenuti", contents.len()));
    output
}

fn format_contents_simple(contents: &[ContentResult]) -> String {
    if contents.is_empty() {
        return "Nessun contenuto trovato.".to_string();
    }

    let mut output = String::new();

    for content in contents {
        output.push_str(&format!("• {} ", content.name));

        if let Some(content_type) = &content.type_key {
            output.push_str(&format!("[{}] ", content_type));
        }

        if let Some(year) = content.formatted_publication_date() {
            output.push_str(&format!("({}) ", year.split('-').next().unwrap_or("")));
        }

        output.push('\n');
    }

    output.push_str(&format!("\nTotale: {} contenuti", contents.len()));
    output
}

/// Tronca una stringa alla lunghezza specificata aggiungendo "..."
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Short", 10), "Short");
        assert_eq!(truncate("Very long string here", 10), "Very lo...");
    }

    #[test]
    fn test_output_format_from_str() {
        assert!(matches!(OutputFormat::from_str("json"), OutputFormat::Json));
        assert!(matches!(
            OutputFormat::from_str("table"),
            OutputFormat::Table
        ));
        assert!(matches!(
            OutputFormat::from_str("simple"),
            OutputFormat::Simple
        ));
    }
}
