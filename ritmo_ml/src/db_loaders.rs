//! Database loaders for ML entity records
//!
//! This module provides functions to load entity records from the ritmo database
//! and convert them into ML-ready structures for deduplication.

use crate::people::record::PersonRecord;
use crate::publishers::record::PublisherRecord;
use crate::series::record::SeriesRecord;
use crate::tags::record::TagRecord;
use crate::utils::MLStringUtils;
use ritmo_errors::RitmoResult;
use sqlx::SqlitePool;

/// Load all people (authors) from the database
///
/// Returns a vector of PersonRecord with normalized names ready for ML processing.
/// Each record includes:
/// - id: database ID
/// - full_name: original name from database
/// - normalized_key: NFC-normalized version for comparison
/// - aliases: empty vector (to be populated by ML)
pub async fn load_people_from_db(pool: &SqlitePool) -> RitmoResult<Vec<PersonRecord>> {
    let normalizer = MLStringUtils::default();

    let rows = sqlx::query!(
        r#"
        SELECT id, name
        FROM people
        ORDER BY id
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut records = Vec::new();

    for row in rows {
        let id = row.id;
        let name = row.name;

        // Use PersonRecord::new which handles parsing and normalization
        match PersonRecord::new(id, &name, &normalizer) {
            Ok(record) => records.push(record),
            Err(e) => {
                // Log error but continue with other records
                eprintln!(
                    "Warning: failed to parse person '{}' (id {}): {}",
                    name, id, e
                );
            }
        }
    }

    Ok(records)
}

/// Load all publishers from the database
///
/// Returns a vector of PublisherRecord with normalized names ready for ML processing.
pub async fn load_publishers_from_db(pool: &SqlitePool) -> RitmoResult<Vec<PublisherRecord>> {
    let normalizer = MLStringUtils::default();

    let rows = sqlx::query!(
        r#"
        SELECT id, name
        FROM publishers
        ORDER BY id
        "#
    )
    .fetch_all(pool)
    .await?;

    let records = rows
        .into_iter()
        .map(|row| {
            let id = row.id;
            let name = row.name;
            let normalized_name = normalizer.normalize_string(&name);

            PublisherRecord {
                id,
                name,
                normalized_name,
                variants: Vec::new(), // Will be populated by ML
            }
        })
        .collect();

    Ok(records)
}

/// Load all series from the database
///
/// Returns a vector of SeriesRecord with normalized names ready for ML processing.
pub async fn load_series_from_db(pool: &SqlitePool) -> RitmoResult<Vec<SeriesRecord>> {
    let normalizer = MLStringUtils::default();

    let rows = sqlx::query!(
        r#"
        SELECT id, name
        FROM series
        ORDER BY id
        "#
    )
    .fetch_all(pool)
    .await?;

    let records = rows
        .into_iter()
        .map(|row| {
            let id = row.id;
            let title = row.name;
            let normalized_title = normalizer.normalize_string(&title);

            SeriesRecord {
                id,
                title,
                normalized_title,
                variants: Vec::new(), // Will be populated by ML
            }
        })
        .collect();

    Ok(records)
}

/// Load all tags from the database
///
/// Returns a vector of TagRecord with normalized labels ready for ML processing.
pub async fn load_tags_from_db(pool: &SqlitePool) -> RitmoResult<Vec<TagRecord>> {
    let normalizer = MLStringUtils::default();

    let rows = sqlx::query!(
        r#"
        SELECT id, name
        FROM tags
        ORDER BY id
        "#
    )
    .fetch_all(pool)
    .await?;

    let records = rows
        .into_iter()
        .map(|row| {
            let id = row.id;
            let label = row.name;
            let normalized_label = normalizer.normalize_string(&label);

            TagRecord {
                id,
                label,
                normalized_label,
            }
        })
        .collect();

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires actual database
    async fn test_load_people_from_db() {
        // This test requires a real database with test data
        // Run with: cargo test --package ritmo_ml -- --ignored
    }

    #[tokio::test]
    #[ignore] // Requires actual database
    async fn test_load_publishers_from_db() {
        // This test requires a real database with test data
    }

    #[tokio::test]
    #[ignore] // Requires actual database
    async fn test_load_series_from_db() {
        // This test requires a real database with test data
    }

    #[tokio::test]
    #[ignore] // Requires actual database
    async fn test_load_tags_from_db() {
        // This test requires a real database with test data
    }
}
