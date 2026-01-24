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
    use crate::test_helpers::*;

    #[tokio::test]
    async fn test_load_people_from_db() {
        // Create test database with people data
        let pool = create_test_db().await.unwrap();
        populate_test_people(&pool).await.unwrap();

        // Load people from database
        let people = load_people_from_db(&pool).await.unwrap();

        // Verify we loaded all 12 people
        assert_eq!(people.len(), 12);

        // Verify first person (Stephen King)
        assert_eq!(people[0].id, 1);
        assert_eq!(people[0].original_input, "Stephen King");

        // Verify normalization is working
        assert!(!people[0].normalized_key.is_empty());

        // Verify all records have valid IDs
        for person in &people {
            assert!(person.id > 0);
            assert!(!person.original_input.is_empty());
        }
    }

    #[tokio::test]
    async fn test_load_publishers_from_db() {
        // Create test database with publishers data
        let pool = create_test_db().await.unwrap();
        populate_test_publishers(&pool).await.unwrap();

        // Load publishers from database
        let publishers = load_publishers_from_db(&pool).await.unwrap();

        // Verify we loaded all 9 publishers
        assert_eq!(publishers.len(), 9);

        // Verify first publisher
        assert_eq!(publishers[0].id, 1);
        assert_eq!(publishers[0].name, "Penguin Random House");

        // Verify all records have valid data
        for publisher in &publishers {
            assert!(publisher.id > 0);
            assert!(!publisher.name.is_empty());
            assert!(!publisher.normalized_name.is_empty());
        }
    }

    #[tokio::test]
    async fn test_load_series_from_db() {
        // Create test database with series data
        let pool = create_test_db().await.unwrap();
        populate_test_series(&pool).await.unwrap();

        // Load series from database
        let series = load_series_from_db(&pool).await.unwrap();

        // Verify we loaded all 8 series
        assert_eq!(series.len(), 8);

        // Verify first series
        assert_eq!(series[0].id, 1);
        assert_eq!(series[0].title, "The Dark Tower");

        // Verify all records have valid data
        for s in &series {
            assert!(s.id > 0);
            assert!(!s.title.is_empty());
            assert!(!s.normalized_title.is_empty());
        }
    }

    #[tokio::test]
    async fn test_load_tags_from_db() {
        // Create test database with tags data
        let pool = create_test_db().await.unwrap();
        populate_test_tags(&pool).await.unwrap();

        // Load tags from database
        let tags = load_tags_from_db(&pool).await.unwrap();

        // Verify we loaded all 8 tags
        assert_eq!(tags.len(), 8);

        // Verify first tag
        assert_eq!(tags[0].id, 1);
        assert_eq!(tags[0].label, "Fantasy");

        // Verify all records have valid data
        for tag in &tags {
            assert!(tag.id > 0);
            assert!(!tag.label.is_empty());
            assert!(!tag.normalized_label.is_empty());
        }
    }
}
