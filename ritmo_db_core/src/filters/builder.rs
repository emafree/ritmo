//! Query builder for filters
//!
//! This module contains the logic for building SQL queries from filter structures.
//! Supports OR logic for multiple values within the same filter type.

use super::types::{BookFilters, ContentFilters};

/// Helper function to build OR clauses for multiple values
/// Returns (sql_clause, params) or None if values is empty
fn build_or_clause(
    field_name: &str,
    values: &[String],
    use_like: bool,
) -> Option<(String, Vec<String>)> {
    if values.is_empty() {
        return None;
    }

    let mut or_parts = Vec::new();
    let mut params = Vec::new();

    for value in values {
        if use_like {
            or_parts.push(format!("{} LIKE ?", field_name));
            params.push(format!("%{}%", value));
        } else {
            or_parts.push(format!("{} = ?", field_name));
            params.push(value.clone());
        }
    }

    // If single value, no need for parentheses
    let clause = if or_parts.len() == 1 {
        or_parts[0].clone()
    } else {
        format!("({})", or_parts.join(" OR "))
    };

    Some((clause, params))
}

/// Costruisce la query SQL per listare libri con filtri
///
/// Supports OR logic for multiple values:
/// - Multiple authors: (author LIKE '%King%' OR author LIKE '%Tolkien%')
/// - Multiple formats: (format LIKE '%epub%' OR format LIKE '%pdf%')
/// - Different filter types are combined with AND
pub fn build_books_query(filters: &BookFilters) -> (String, Vec<String>) {
    let mut query = String::from(
        r#"
        SELECT DISTINCT
            books.id,
            books.name,
            books.original_title,
            publishers.name as publisher_name,
            formats.key as format_key,
            series.name as series_name,
            books.series_index,
            books.publication_date,
            books.isbn,
            books.pages,
            books.file_link,
            books.created_at
        FROM books
        LEFT JOIN publishers ON books.publisher_id = publishers.id
        LEFT JOIN formats ON books.format_id = formats.id
        LEFT JOIN series ON books.series_id = series.id
        "#,
    );

    let mut params: Vec<String> = Vec::new();
    let mut where_clauses: Vec<String> = Vec::new();

    // Filtro autori (OR logic if multiple, richiede JOIN con people)
    if !filters.authors.is_empty() {
        query.push_str(
            r#"
            LEFT JOIN x_books_people_roles ON books.id = x_books_people_roles.book_id
            LEFT JOIN people ON x_books_people_roles.person_id = people.id
            "#,
        );

        if let Some((clause, mut clause_params)) =
            build_or_clause("people.name", &filters.authors, true)
        {
            where_clauses.push(clause);
            params.append(&mut clause_params);
        }
    }

    // Filtro editori (OR logic if multiple)
    if let Some((clause, mut clause_params)) =
        build_or_clause("publishers.name", &filters.publishers, true)
    {
        where_clauses.push(clause);
        params.append(&mut clause_params);
    }

    // Filtro serie (OR logic if multiple)
    if let Some((clause, mut clause_params)) =
        build_or_clause("series.name", &filters.series_list, true)
    {
        where_clauses.push(clause);
        params.append(&mut clause_params);
    }

    // Filtro formati (OR logic if multiple)
    if let Some((clause, mut clause_params)) =
        build_or_clause("formats.key", &filters.formats, true)
    {
        where_clauses.push(clause);
        params.append(&mut clause_params);
    }

    // Filtro anno
    if let Some(year) = filters.year {
        where_clauses
            .push("strftime('%Y', datetime(books.publication_date, 'unixepoch')) = ?".to_string());
        params.push(year.to_string());
    }

    // Filtro ISBN
    if let Some(isbn) = &filters.isbn {
        where_clauses.push("books.isbn LIKE ?".to_string());
        params.push(format!("%{}%", isbn));
    }

    // Ricerca full-text
    if let Some(search) = &filters.search {
        where_clauses.push(
            "(books.name LIKE ? OR books.original_title LIKE ? OR books.notes LIKE ?)".to_string(),
        );
        let search_pattern = format!("%{}%", search);
        params.push(search_pattern.clone());
        params.push(search_pattern.clone());
        params.push(search_pattern);
    }

    // Filtro data acquisizione (dopo)
    if let Some(acquired_after) = filters.acquired_after {
        where_clauses.push("books.created_at >= ?".to_string());
        params.push(acquired_after.to_string());
    }

    // Filtro data acquisizione (prima)
    if let Some(acquired_before) = filters.acquired_before {
        where_clauses.push("books.created_at <= ?".to_string());
        params.push(acquired_before.to_string());
    }

    // Aggiungi WHERE se ci sono filtri
    if !where_clauses.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&where_clauses.join(" AND "));
    }

    // Ordinamento
    query.push_str(&format!(" ORDER BY {} ASC", filters.sort.to_sql()));

    // Limit e Offset
    if let Some(limit) = filters.limit {
        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, filters.offset));
    } else if filters.offset > 0 {
        query.push_str(&format!(" LIMIT -1 OFFSET {}", filters.offset));
    }

    (query, params)
}

/// Costruisce la query SQL per listare contenuti con filtri
///
/// Supports OR logic for multiple authors and content types.
pub fn build_contents_query(filters: &ContentFilters) -> (String, Vec<String>) {
    let mut query = String::from(
        r#"
        SELECT DISTINCT
            contents.id,
            contents.name,
            contents.original_title,
            types.key as type_key,
            contents.publication_date,
            contents.pages,
            contents.created_at
        FROM contents
        LEFT JOIN types ON contents.type_id = types.id
        "#,
    );

    let mut params: Vec<String> = Vec::new();
    let mut where_clauses: Vec<String> = Vec::new();

    // Filtro autori (OR logic if multiple, richiede JOIN con people)
    if !filters.authors.is_empty() {
        query.push_str(
            r#"
            LEFT JOIN x_contents_people_roles ON contents.id = x_contents_people_roles.content_id
            LEFT JOIN people ON x_contents_people_roles.person_id = people.id
            "#,
        );

        if let Some((clause, mut clause_params)) =
            build_or_clause("people.name", &filters.authors, true)
        {
            where_clauses.push(clause);
            params.append(&mut clause_params);
        }
    }

    // Filtro tipi di contenuto (OR logic if multiple)
    if let Some((clause, mut clause_params)) =
        build_or_clause("types.key", &filters.content_types, true)
    {
        where_clauses.push(clause);
        params.append(&mut clause_params);
    }

    // Filtro anno
    if let Some(year) = filters.year {
        where_clauses.push(
            "strftime('%Y', datetime(contents.publication_date, 'unixepoch')) = ?".to_string(),
        );
        params.push(year.to_string());
    }

    // Ricerca full-text
    if let Some(search) = &filters.search {
        where_clauses.push(
            "(contents.name LIKE ? OR contents.original_title LIKE ? OR contents.notes LIKE ?)"
                .to_string(),
        );
        let search_pattern = format!("%{}%", search);
        params.push(search_pattern.clone());
        params.push(search_pattern.clone());
        params.push(search_pattern);
    }

    // Aggiungi WHERE se ci sono filtri
    if !where_clauses.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&where_clauses.join(" AND "));
    }

    // Ordinamento
    query.push_str(&format!(" ORDER BY {} ASC", filters.sort.to_sql()));

    // Limit e Offset
    if let Some(limit) = filters.limit {
        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, filters.offset));
    } else if filters.offset > 0 {
        query.push_str(&format!(" LIMIT -1 OFFSET {}", filters.offset));
    }

    (query, params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filters::types::BookSortField;

    #[test]
    fn test_build_or_clause_empty() {
        let result = build_or_clause("field", &[], true);
        assert!(result.is_none());
    }

    #[test]
    fn test_build_or_clause_single() {
        let values = vec!["test".to_string()];
        let (clause, params) = build_or_clause("field", &values, true).unwrap();

        assert_eq!(clause, "field LIKE ?");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], "%test%");
    }

    #[test]
    fn test_build_or_clause_multiple() {
        let values = vec!["value1".to_string(), "value2".to_string()];
        let (clause, params) = build_or_clause("field", &values, true).unwrap();

        assert_eq!(clause, "(field LIKE ? OR field LIKE ?)");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], "%value1%");
        assert_eq!(params[1], "%value2%");
    }

    #[test]
    fn test_build_books_query_no_filters() {
        let filters = BookFilters::default();
        let (query, params) = build_books_query(&filters);

        assert!(query.contains("SELECT DISTINCT"));
        assert!(query.contains("FROM books"));
        assert!(query.contains("ORDER BY"));
        assert!(params.is_empty());
    }

    #[test]
    fn test_build_books_query_with_single_author() {
        let filters = BookFilters::default().with_author("Calvino");
        let (query, params) = build_books_query(&filters);

        assert!(query.contains("people.name LIKE ?"));
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], "%Calvino%");
    }

    #[test]
    fn test_build_books_query_with_multiple_authors() {
        let filters = BookFilters::default()
            .with_author("King")
            .with_author("Tolkien");
        let (query, params) = build_books_query(&filters);

        assert!(query.contains("people.name LIKE ?"));
        assert!(query.contains("OR"));
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], "%King%");
        assert_eq!(params[1], "%Tolkien%");
    }

    #[test]
    fn test_build_books_query_with_multiple_filters() {
        let filters = BookFilters {
            authors: vec!["Calvino".to_string()],
            formats: vec!["epub".to_string()],
            year: Some(2020),
            ..Default::default()
        };
        let (query, params) = build_books_query(&filters);

        assert!(query.contains("WHERE"));
        assert!(query.contains("AND"));
        assert_eq!(params.len(), 3); // author + format + year
    }

    #[test]
    fn test_build_books_query_with_limit() {
        let filters = BookFilters {
            limit: Some(10),
            offset: 20,
            ..Default::default()
        };
        let (query, params) = build_books_query(&filters);

        assert!(query.contains("LIMIT 10 OFFSET 20"));
        assert!(params.is_empty());
    }

    #[test]
    fn test_build_books_query_or_and_combination() {
        // (author=King OR author=Tolkien) AND (format=epub OR format=pdf)
        let filters = BookFilters {
            authors: vec!["King".to_string(), "Tolkien".to_string()],
            formats: vec!["epub".to_string(), "pdf".to_string()],
            ..Default::default()
        };
        let (query, params) = build_books_query(&filters);

        // Should have 2 OR groups combined with AND
        assert!(query.contains("OR"));
        assert!(query.contains("AND"));
        assert_eq!(params.len(), 4); // 2 authors + 2 formats
    }
}
