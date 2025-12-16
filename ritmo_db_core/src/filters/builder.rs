//! Query builder for filters
//!
//! This module contains the logic for building SQL queries from filter structures.

use super::types::{BookFilters, ContentFilters};

/// Costruisce la query SQL per listare libri con filtri
pub fn build_books_query(filters: &BookFilters) -> (String, Vec<String>) {
    let mut query = String::from(
        r#"
        SELECT DISTINCT
            books.id,
            books.name,
            books.original_title,
            publishers.name as publisher_name,
            formats.name as format_name,
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

    // Filtro autore (richiede JOIN con people)
    if filters.author.is_some() {
        query.push_str(
            r#"
            LEFT JOIN x_books_people_roles ON books.id = x_books_people_roles.book_id
            LEFT JOIN people ON x_books_people_roles.person_id = people.id
            "#,
        );
        where_clauses.push("people.name LIKE ?".to_string());
        params.push(format!("%{}%", filters.author.as_ref().unwrap()));
    }

    // Filtro editore
    if let Some(pub_name) = &filters.publisher {
        where_clauses.push("publishers.name LIKE ?".to_string());
        params.push(format!("%{}%", pub_name));
    }

    // Filtro serie
    if let Some(ser_name) = &filters.series {
        where_clauses.push("series.name LIKE ?".to_string());
        params.push(format!("%{}%", ser_name));
    }

    // Filtro formato
    if let Some(fmt_name) = &filters.format {
        where_clauses.push("formats.name LIKE ?".to_string());
        params.push(format!("%{}%", fmt_name));
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
pub fn build_contents_query(filters: &ContentFilters) -> (String, Vec<String>) {
    let mut query = String::from(
        r#"
        SELECT DISTINCT
            contents.id,
            contents.name,
            contents.original_title,
            types.name as type_name,
            contents.publication_date,
            contents.pages,
            contents.created_at
        FROM contents
        LEFT JOIN types ON contents.type_id = types.id
        "#,
    );

    let mut params: Vec<String> = Vec::new();
    let mut where_clauses: Vec<String> = Vec::new();

    // Filtro autore (richiede JOIN con people)
    if filters.author.is_some() {
        query.push_str(
            r#"
            LEFT JOIN x_contents_people_roles ON contents.id = x_contents_people_roles.content_id
            LEFT JOIN people ON x_contents_people_roles.person_id = people.id
            "#,
        );
        where_clauses.push("people.name LIKE ?".to_string());
        params.push(format!("%{}%", filters.author.as_ref().unwrap()));
    }

    // Filtro tipo
    if let Some(type_name) = &filters.content_type {
        where_clauses.push("types.name LIKE ?".to_string());
        params.push(format!("%{}%", type_name));
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
    fn test_build_books_query_no_filters() {
        let filters = BookFilters::default();
        let (query, params) = build_books_query(&filters);

        assert!(query.contains("SELECT DISTINCT"));
        assert!(query.contains("FROM books"));
        assert!(query.contains("ORDER BY"));
        assert!(params.is_empty());
    }

    #[test]
    fn test_build_books_query_with_author() {
        let filters = BookFilters {
            author: Some("Calvino".to_string()),
            ..Default::default()
        };
        let (query, params) = build_books_query(&filters);

        assert!(query.contains("people.name LIKE ?"));
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], "%Calvino%");
    }

    #[test]
    fn test_build_books_query_with_multiple_filters() {
        let filters = BookFilters {
            author: Some("Calvino".to_string()),
            format: Some("epub".to_string()),
            year: Some(2020),
            ..Default::default()
        };
        let (query, params) = build_books_query(&filters);

        assert!(query.contains("WHERE"));
        assert!(query.contains("AND"));
        assert_eq!(params.len(), 3);
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
}
