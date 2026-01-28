use ritmo_errors::RitmoResult;
use sqlx::SqlitePool;

/// Mark a book for metadata sync
pub async fn mark_book_for_sync(
    pool: &SqlitePool,
    book_id: i64,
    reason: &str,
) -> RitmoResult<()> {
    sqlx::query!(
        "INSERT INTO pending_metadata_sync (book_id, reason) VALUES (?, ?)",
        book_id,
        reason
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Mark multiple books for sync in batch
pub async fn mark_books_for_sync(
    pool: &SqlitePool,
    book_ids: &[i64],
    reason: &str,
) -> RitmoResult<()> {
    for &book_id in book_ids {
        mark_book_for_sync(pool, book_id, reason).await?;
    }
    Ok(())
}

/// Get list of book IDs pending sync
pub async fn get_pending_sync_books(pool: &SqlitePool) -> RitmoResult<Vec<i64>> {
    let records = sqlx::query!("SELECT DISTINCT book_id FROM pending_metadata_sync")
        .fetch_all(pool)
        .await?;

    Ok(records.into_iter().map(|r| r.book_id).collect())
}

/// Get count of books pending sync
pub async fn count_pending_sync(pool: &SqlitePool) -> RitmoResult<i64> {
    let result = sqlx::query!("SELECT COUNT(DISTINCT book_id) as count FROM pending_metadata_sync")
        .fetch_one(pool)
        .await?;

    Ok(result.count)
}

/// Clear sync mark for a book
pub async fn clear_sync_mark(pool: &SqlitePool, book_id: i64) -> RitmoResult<()> {
    sqlx::query!("DELETE FROM pending_metadata_sync WHERE book_id = ?", book_id)
        .execute(pool)
        .await?;
    Ok(())
}
