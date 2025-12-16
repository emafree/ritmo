//! Query executor for filters
//!
//! This module contains the logic for executing SQL queries against the database.

use super::builder::{build_books_query, build_contents_query};
use super::types::{BookFilters, BookResult, ContentFilters, ContentResult};
use sqlx::SqlitePool;

/// Esegue la query per libri e restituisce i risultati
pub async fn execute_books_query(
    pool: &SqlitePool,
    filters: &BookFilters,
) -> Result<Vec<BookResult>, sqlx::Error> {
    let (query, params) = build_books_query(filters);

    // Costruisci la query con sqlx
    let mut sql_query = sqlx::query_as::<_, BookResult>(&query);

    // Bind dei parametri
    for param in params {
        sql_query = sql_query.bind(param);
    }

    // Esegui la query
    sql_query.fetch_all(pool).await
}

/// Esegue la query per contenuti e restituisce i risultati
pub async fn execute_contents_query(
    pool: &SqlitePool,
    filters: &ContentFilters,
) -> Result<Vec<ContentResult>, sqlx::Error> {
    let (query, params) = build_contents_query(filters);

    // Costruisci la query con sqlx
    let mut sql_query = sqlx::query_as::<_, ContentResult>(&query);

    // Bind dei parametri
    for param in params {
        sql_query = sql_query.bind(param);
    }

    // Esegui la query
    sql_query.fetch_all(pool).await
}
