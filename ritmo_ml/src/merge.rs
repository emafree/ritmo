//! Merge operations for deduplicating entities
//!
//! This module provides safe merge operations for combining duplicate entities.
//! All merges are executed within database transactions to ensure data integrity.

use ritmo_errors::{RitmoErr, RitmoResult};
use sqlx::{Sqlite, SqlitePool, Transaction};

/// Statistics about a merge operation
#[derive(Debug, Clone)]
pub struct MergeStats {
    pub primary_id: i64,
    pub merged_ids: Vec<i64>,
    pub books_updated: usize,
    pub contents_updated: usize,
}

/// Merge duplicate people (authors) into a single primary record
///
/// This function:
/// 1. Validates that all IDs exist
/// 2. Updates all references in junction tables to point to primary_id
/// 3. Deletes duplicate person records
/// 4. Returns statistics about the merge
///
/// # Safety
/// This operation is executed within a transaction. If any step fails,
/// all changes are rolled back.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `primary_id` - ID of the person to keep (primary record)
/// * `duplicate_ids` - IDs of people to merge into primary (will be deleted)
pub async fn merge_people(
    pool: &SqlitePool,
    primary_id: i64,
    duplicate_ids: &[i64],
) -> RitmoResult<MergeStats> {
    if duplicate_ids.is_empty() {
        return Err(RitmoErr::Generic("No duplicate IDs provided".to_string()));
    }

    if duplicate_ids.contains(&primary_id) {
        return Err(RitmoErr::Generic(
            "Primary ID cannot be in duplicate IDs list".to_string(),
        ));
    }

    let mut tx = pool.begin().await?;

    // Step 1: Validate that all person IDs exist
    validate_people_exist(&mut tx, primary_id, duplicate_ids).await?;

    // Step 2: Update x_books_people_roles to point to primary_id
    let books_updated = update_books_people_roles(&mut tx, primary_id, duplicate_ids).await?;

    // Step 3: Update x_contents_people_roles to point to primary_id
    let contents_updated = update_contents_people_roles(&mut tx, primary_id, duplicate_ids).await?;

    // Step 4: Delete duplicate person records
    delete_people(&mut tx, duplicate_ids).await?;

    // Commit transaction
    tx.commit().await?;

    Ok(MergeStats {
        primary_id,
        merged_ids: duplicate_ids.to_vec(),
        books_updated,
        contents_updated,
    })
}

/// Merge duplicate publishers into a single primary record
///
/// Similar to merge_people but for publishers table.
pub async fn merge_publishers(
    pool: &SqlitePool,
    primary_id: i64,
    duplicate_ids: &[i64],
) -> RitmoResult<MergeStats> {
    if duplicate_ids.is_empty() {
        return Err(RitmoErr::Generic("No duplicate IDs provided".to_string()));
    }

    if duplicate_ids.contains(&primary_id) {
        return Err(RitmoErr::Generic(
            "Primary ID cannot be in duplicate IDs list".to_string(),
        ));
    }

    let mut tx = pool.begin().await?;

    // Step 1: Validate that all publisher IDs exist
    validate_publishers_exist(&mut tx, primary_id, duplicate_ids).await?;

    // Step 2: Update books.publisher_id to point to primary_id
    let books_updated = update_books_publisher(&mut tx, primary_id, duplicate_ids).await?;

    // Step 3: Delete duplicate publisher records
    delete_publishers(&mut tx, duplicate_ids).await?;

    // Commit transaction
    tx.commit().await?;

    Ok(MergeStats {
        primary_id,
        merged_ids: duplicate_ids.to_vec(),
        books_updated,
        contents_updated: 0, // Publishers not linked to contents
    })
}

/// Merge duplicate series into a single primary record
///
/// Similar to merge_people but for series table.
pub async fn merge_series(
    pool: &SqlitePool,
    primary_id: i64,
    duplicate_ids: &[i64],
) -> RitmoResult<MergeStats> {
    if duplicate_ids.is_empty() {
        return Err(RitmoErr::Generic("No duplicate IDs provided".to_string()));
    }

    if duplicate_ids.contains(&primary_id) {
        return Err(RitmoErr::Generic(
            "Primary ID cannot be in duplicate IDs list".to_string(),
        ));
    }

    let mut tx = pool.begin().await?;

    // Step 1: Validate that all series IDs exist
    validate_series_exist(&mut tx, primary_id, duplicate_ids).await?;

    // Step 2: Update books.series_id to point to primary_id
    let books_updated = update_books_series(&mut tx, primary_id, duplicate_ids).await?;

    // Step 3: Delete duplicate series records
    delete_series(&mut tx, duplicate_ids).await?;

    // Commit transaction
    tx.commit().await?;

    Ok(MergeStats {
        primary_id,
        merged_ids: duplicate_ids.to_vec(),
        books_updated,
        contents_updated: 0, // Series not linked to contents
    })
}

// ============================================================================
// Helper functions for people merging
// ============================================================================

async fn validate_people_exist(
    tx: &mut Transaction<'_, Sqlite>,
    primary_id: i64,
    duplicate_ids: &[i64],
) -> RitmoResult<()> {
    // Check primary exists
    let primary_exists = sqlx::query!("SELECT id FROM people WHERE id = ?", primary_id)
        .fetch_optional(&mut **tx)
        .await?;

    if primary_exists.is_none() {
        return Err(RitmoErr::Generic(format!(
            "Primary person ID {} not found",
            primary_id
        )));
    }

    // Check all duplicates exist
    for &dup_id in duplicate_ids {
        let exists = sqlx::query!("SELECT id FROM people WHERE id = ?", dup_id)
            .fetch_optional(&mut **tx)
            .await?;

        if exists.is_none() {
            return Err(RitmoErr::Generic(format!(
                "Duplicate person ID {} not found",
                dup_id
            )));
        }
    }

    Ok(())
}

async fn update_books_people_roles(
    tx: &mut Transaction<'_, Sqlite>,
    primary_id: i64,
    duplicate_ids: &[i64],
) -> RitmoResult<usize> {
    let mut total_updated = 0;

    for &dup_id in duplicate_ids {
        let result = sqlx::query!(
            "UPDATE x_books_people_roles SET person_id = ? WHERE person_id = ?",
            primary_id,
            dup_id
        )
        .execute(&mut **tx)
        .await?;

        total_updated += result.rows_affected() as usize;
    }

    Ok(total_updated)
}

async fn update_contents_people_roles(
    tx: &mut Transaction<'_, Sqlite>,
    primary_id: i64,
    duplicate_ids: &[i64],
) -> RitmoResult<usize> {
    let mut total_updated = 0;

    for &dup_id in duplicate_ids {
        let result = sqlx::query!(
            "UPDATE x_contents_people_roles SET person_id = ? WHERE person_id = ?",
            primary_id,
            dup_id
        )
        .execute(&mut **tx)
        .await?;

        total_updated += result.rows_affected() as usize;
    }

    Ok(total_updated)
}

async fn delete_people(tx: &mut Transaction<'_, Sqlite>, ids: &[i64]) -> RitmoResult<()> {
    for &id in ids {
        sqlx::query!("DELETE FROM people WHERE id = ?", id)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}

// ============================================================================
// Helper functions for publisher merging
// ============================================================================

async fn validate_publishers_exist(
    tx: &mut Transaction<'_, Sqlite>,
    primary_id: i64,
    duplicate_ids: &[i64],
) -> RitmoResult<()> {
    let primary_exists = sqlx::query!("SELECT id FROM publishers WHERE id = ?", primary_id)
        .fetch_optional(&mut **tx)
        .await?;

    if primary_exists.is_none() {
        return Err(RitmoErr::Generic(format!(
            "Primary publisher ID {} not found",
            primary_id
        )));
    }

    for &dup_id in duplicate_ids {
        let exists = sqlx::query!("SELECT id FROM publishers WHERE id = ?", dup_id)
            .fetch_optional(&mut **tx)
            .await?;

        if exists.is_none() {
            return Err(RitmoErr::Generic(format!(
                "Duplicate publisher ID {} not found",
                dup_id
            )));
        }
    }

    Ok(())
}

async fn update_books_publisher(
    tx: &mut Transaction<'_, Sqlite>,
    primary_id: i64,
    duplicate_ids: &[i64],
) -> RitmoResult<usize> {
    let mut total_updated = 0;

    for &dup_id in duplicate_ids {
        let result = sqlx::query!(
            "UPDATE books SET publisher_id = ? WHERE publisher_id = ?",
            primary_id,
            dup_id
        )
        .execute(&mut **tx)
        .await?;

        total_updated += result.rows_affected() as usize;
    }

    Ok(total_updated)
}

async fn delete_publishers(tx: &mut Transaction<'_, Sqlite>, ids: &[i64]) -> RitmoResult<()> {
    for &id in ids {
        sqlx::query!("DELETE FROM publishers WHERE id = ?", id)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}

// ============================================================================
// Helper functions for series merging
// ============================================================================

async fn validate_series_exist(
    tx: &mut Transaction<'_, Sqlite>,
    primary_id: i64,
    duplicate_ids: &[i64],
) -> RitmoResult<()> {
    let primary_exists = sqlx::query!("SELECT id FROM series WHERE id = ?", primary_id)
        .fetch_optional(&mut **tx)
        .await?;

    if primary_exists.is_none() {
        return Err(RitmoErr::Generic(format!(
            "Primary series ID {} not found",
            primary_id
        )));
    }

    for &dup_id in duplicate_ids {
        let exists = sqlx::query!("SELECT id FROM series WHERE id = ?", dup_id)
            .fetch_optional(&mut **tx)
            .await?;

        if exists.is_none() {
            return Err(RitmoErr::Generic(format!(
                "Duplicate series ID {} not found",
                dup_id
            )));
        }
    }

    Ok(())
}

async fn update_books_series(
    tx: &mut Transaction<'_, Sqlite>,
    primary_id: i64,
    duplicate_ids: &[i64],
) -> RitmoResult<usize> {
    let mut total_updated = 0;

    for &dup_id in duplicate_ids {
        let result = sqlx::query!(
            "UPDATE books SET series_id = ? WHERE series_id = ?",
            primary_id,
            dup_id
        )
        .execute(&mut **tx)
        .await?;

        total_updated += result.rows_affected() as usize;
    }

    Ok(total_updated)
}

async fn delete_series(tx: &mut Transaction<'_, Sqlite>, ids: &[i64]) -> RitmoResult<()> {
    for &id in ids {
        sqlx::query!("DELETE FROM series WHERE id = ?", id)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[tokio::test]
    async fn test_merge_people() {
        // Create test database with people and books
        let pool = create_test_db().await.unwrap();
        populate_test_people(&pool).await.unwrap();
        populate_test_books_with_people(&pool).await.unwrap();

        // Verify initial state: 12 people, 3 books
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM people")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 12);

        // Merge Stephen King variants (IDs 2, 3, 4) into ID 1
        let stats = merge_people(&pool, 1, &[2, 3, 4]).await.unwrap();

        // Verify merge stats
        assert_eq!(stats.primary_id, 1);
        assert_eq!(stats.merged_ids, vec![2, 3, 4]);
        assert_eq!(stats.books_updated, 2); // Books 2 and 3 should be updated

        // Verify people count reduced by 3
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM people")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 9);

        // Verify that duplicate IDs no longer exist
        let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM people WHERE id IN (2, 3, 4)")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(exists, 0);

        // Verify all book relationships now point to primary ID (1)
        let book_refs: Vec<i64> = sqlx::query_scalar(
            "SELECT person_id FROM x_books_people_roles WHERE book_id IN (1, 2, 3) ORDER BY book_id"
        )
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(book_refs, vec![1, 1, 1]); // All should point to ID 1
    }

    #[tokio::test]
    async fn test_merge_publishers() {
        // Create test database with publishers
        let pool = create_test_db().await.unwrap();
        populate_test_publishers(&pool).await.unwrap();

        // Create test books with publishers
        sqlx::query("INSERT INTO books (id, name, publisher_id) VALUES (1, 'Book 1', 2), (2, 'Book 2', 3)")
            .execute(&pool)
            .await
            .unwrap();

        // Verify initial state: 9 publishers
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM publishers")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 9);

        // Merge Penguin/Random House variants (IDs 2, 3) into ID 1
        let stats = merge_publishers(&pool, 1, &[2, 3]).await.unwrap();

        // Verify merge stats
        assert_eq!(stats.primary_id, 1);
        assert_eq!(stats.merged_ids, vec![2, 3]);
        assert_eq!(stats.books_updated, 2);

        // Verify publisher count reduced by 2
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM publishers")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 7);

        // Verify all book relationships now point to primary ID (1)
        let publisher_ids: Vec<Option<i64>> = sqlx::query_scalar(
            "SELECT publisher_id FROM books WHERE id IN (1, 2) ORDER BY id"
        )
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(publisher_ids, vec![Some(1), Some(1)]);
    }

    #[tokio::test]
    async fn test_merge_series() {
        // Create test database with series
        let pool = create_test_db().await.unwrap();
        populate_test_series(&pool).await.unwrap();

        // Create test books with series
        sqlx::query("INSERT INTO books (id, name, series_id) VALUES (1, 'Book 1', 2), (2, 'Book 2', 1)")
            .execute(&pool)
            .await
            .unwrap();

        // Verify initial state: 8 series
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM series")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 8);

        // Merge Dark Tower variants (ID 2) into ID 1
        let stats = merge_series(&pool, 1, &[2]).await.unwrap();

        // Verify merge stats
        assert_eq!(stats.primary_id, 1);
        assert_eq!(stats.merged_ids, vec![2]);
        assert_eq!(stats.books_updated, 1);

        // Verify series count reduced by 1
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM series")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 7);

        // Verify all book relationships now point to primary ID (1)
        let series_ids: Vec<Option<i64>> = sqlx::query_scalar(
            "SELECT series_id FROM books WHERE id IN (1, 2) ORDER BY id"
        )
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(series_ids, vec![Some(1), Some(1)]);
    }

    #[tokio::test]
    async fn test_merge_people_validation_errors() {
        let pool = create_test_db().await.unwrap();
        populate_test_people(&pool).await.unwrap();

        // Test empty duplicate IDs
        let result = merge_people(&pool, 1, &[]).await;
        assert!(result.is_err());

        // Test primary ID in duplicate list
        let result = merge_people(&pool, 1, &[1, 2]).await;
        assert!(result.is_err());

        // Test non-existent ID
        let result = merge_people(&pool, 1, &[999]).await;
        assert!(result.is_err());
    }
}
