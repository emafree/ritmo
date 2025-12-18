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

    #[tokio::test]
    #[ignore] // Requires actual database with test data
    async fn test_merge_people() {
        // This test requires a real database with test data
        // Run with: cargo test --package ritmo_ml -- --ignored
    }

    #[tokio::test]
    #[ignore]
    async fn test_merge_publishers() {
        // This test requires a real database with test data
    }

    #[tokio::test]
    #[ignore]
    async fn test_merge_series() {
        // This test requires a real database with test data
    }
}
