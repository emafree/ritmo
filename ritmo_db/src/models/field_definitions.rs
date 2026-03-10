use crate::crud_trait::CrudModel;
use sqlx::FromRow;

/// Represents a row in the `field_definitions` table.
/// The `field_name` is a canonical i18n key (e.g. "field-isbn").
/// Display names are resolved at runtime from the i18n JSON files.
#[derive(Debug, Clone, FromRow)]
pub struct FieldDefinitionRow {
    pub id: Option<i64>,
    pub entity: String,           // "book" | "content"
    pub field_name: String,       // canonical key, e.g. "field-isbn"
    pub data_kind: String,        // "string" | "quantity" | "date" | "enum" | "person"
    pub sort_order: i64,
    pub enum_values: Option<String>, // JSON array string, e.g. '["en","it","fr"]'
    pub created_at: i64,
}

impl CrudModel for FieldDefinitionRow {
    const TABLE_NAME: &'static str = "field_definitions";
    const ORDER_BY: &'static str = "sort_order";
}

impl FieldDefinitionRow {
    /// Save a new FieldDefinitionRow to the database.
    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "INSERT INTO field_definitions (entity, field_name, data_kind, sort_order, enum_values, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            self.entity,
            self.field_name,
            self.data_kind,
            self.sort_order,
            self.enum_values,
            now,
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// List all field definitions for a specific entity, ordered by sort_order.
    pub async fn list_for_entity(
        pool: &sqlx::SqlitePool,
        entity: &str,
    ) -> Result<Vec<FieldDefinitionRow>, sqlx::Error> {
        sqlx::query_as!(
            FieldDefinitionRow,
            "SELECT * FROM field_definitions WHERE entity = ? ORDER BY sort_order",
            entity
        )
        .fetch_all(pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crud_model_impl() {
        assert_eq!(FieldDefinitionRow::TABLE_NAME, "field_definitions");
        assert_eq!(FieldDefinitionRow::ORDER_BY, "sort_order");
    }
}
