//! Generic CRUD trait per standardizzare operazioni comuni
//!
//! Questo trait riduce la duplicazione di codice eliminando ~450 linee
//! di save/get/delete ripetuti nei vari modelli.

use sqlx::SqlitePool;
use std::fmt::Debug;

/// Generic CRUD trait per tutti i modelli database
///
/// Implementare questo trait permette di usare funzioni generiche
/// per operazioni comuni senza duplicare codice
pub trait CrudModel: Sized + Send + Sync + Debug + Unpin
where
    Self: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
{
    /// Nome della tabella nel database
    const TABLE_NAME: &'static str;

    /// Campo per ordinamento di default (default: "id")
    const ORDER_BY: &'static str = "id";

    /// Getter per il table name (permite override)
    fn table_name() -> &'static str {
        Self::TABLE_NAME
    }

    /// Getter per il campo di ordinamento
    fn order_by() -> &'static str {
        Self::ORDER_BY
    }
}

/// Generic READ operation
pub async fn crud_get<T: CrudModel>(pool: &SqlitePool, id: i64) -> Result<Option<T>, sqlx::Error> {
    let table = T::table_name();
    let query_str = format!("SELECT * FROM {} WHERE id = ?", table);

    sqlx::query_as::<_, T>(&query_str)
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Generic LIST operation
pub async fn crud_list_all<T: CrudModel>(pool: &SqlitePool) -> Result<Vec<T>, sqlx::Error> {
    let table = T::table_name();
    let order_by = T::order_by();
    let query_str = format!("SELECT * FROM {} ORDER BY {}", table, order_by);

    sqlx::query_as::<_, T>(&query_str).fetch_all(pool).await
}

/// Generic DELETE operation
pub async fn crud_delete<T: CrudModel>(pool: &SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
    let table = T::table_name();
    let query_str = format!("DELETE FROM {} WHERE id = ?", table);

    let result = sqlx::query(&query_str).bind(id).execute(pool).await?;

    Ok(result.rows_affected())
}

/// Generic SEARCH operation
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `pattern` - Search pattern (will be wrapped with %)
/// * `search_fields` - Which columns to search in (e.g., &["name", "notes"])
pub async fn crud_search<T: CrudModel>(
    pool: &SqlitePool,
    pattern: &str,
    search_fields: &[&str],
) -> Result<Vec<T>, sqlx::Error> {
    if search_fields.is_empty() {
        return Ok(Vec::new());
    }

    let table = T::table_name();
    let order_by = T::order_by();

    // Costruisci la clausola WHERE dinamicamente
    let conditions = search_fields
        .iter()
        .map(|field| format!("{} LIKE ?", field))
        .collect::<Vec<_>>()
        .join(" OR ");

    let query_str = format!(
        "SELECT * FROM {} WHERE {} ORDER BY {}",
        table, conditions, order_by
    );

    // Bind dello stesso pattern per ogni campo
    let mut query = sqlx::query_as::<_, T>(&query_str);
    let search_pattern = format!("%{}%", pattern);

    for _ in search_fields {
        query = query.bind(search_pattern.clone());
    }

    query.fetch_all(pool).await
}

#[cfg(test)]
mod tests {
    use super::*;

    // I test concreti verranno aggiunti dopo implementazione nei modelli
}
