use crate::crud_trait::CrudModel;
use sqlx::FromRow;

/// Represents a row in the `page_fields` table.
/// The `field_key` is a canonical i18n key (e.g. "field-isbn").
/// Display names are resolved at runtime from the i18n JSON files.
#[derive(Debug, Clone, FromRow)]
pub struct PageFieldRow {
    pub id: Option<i64>,
    pub page: String,                    // "book_page" | "content_page" | "people_page"
    pub field_key: String,               // canonical key, e.g. "field-isbn"
    pub data_kind: String,               // "string" | "quantity" | "date" | "enum" | "person"
    pub sort_order: i64,
    pub enum_values: Option<String>,     // JSON array string, e.g. '["en","it","fr"]'
    pub relation_type: String,           // "direct" | "fk" | "junction"
    pub target_table: Option<String>,    // e.g. "publishers", "x_books_tags"
    pub target_field: Option<String>,    // e.g. "publisher_id"; only for relation_type = "fk"
    pub created_at: i64,
}

impl CrudModel for PageFieldRow {
    const TABLE_NAME: &'static str = "page_fields";
    const ORDER_BY: &'static str = "sort_order";
}

impl PageFieldRow {
    /// Save a new PageFieldRow to the database.
    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO page_fields (page, field_key, data_kind, sort_order, enum_values, relation_type, target_table, target_field)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&self.page)
        .bind(&self.field_key)
        .bind(&self.data_kind)
        .bind(self.sort_order)
        .bind(&self.enum_values)
        .bind(&self.relation_type)
        .bind(&self.target_table)
        .bind(&self.target_field)
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// List all page fields for a specific page, ordered by sort_order.
    pub async fn list_for_page(
        pool: &sqlx::SqlitePool,
        page: &str,
    ) -> Result<Vec<PageFieldRow>, sqlx::Error> {
        sqlx::query_as::<_, PageFieldRow>(
            "SELECT id, page, field_key, data_kind, sort_order, enum_values, relation_type, target_table, target_field, created_at
             FROM page_fields WHERE page = ? ORDER BY sort_order",
        )
        .bind(page)
        .fetch_all(pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crud_model_impl() {
        assert_eq!(PageFieldRow::TABLE_NAME, "page_fields");
        assert_eq!(PageFieldRow::ORDER_BY, "sort_order");
    }
}
