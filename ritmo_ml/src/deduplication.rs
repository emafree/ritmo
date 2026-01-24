//! Deduplication workflow for finding and merging duplicate entities
//!
//! This module provides high-level functions that combine ML detection
//! with database operations to identify and optionally merge duplicates.

use crate::db_loaders::{load_people_from_db, load_publishers_from_db, load_series_from_db};
use crate::entity_learner::MLEntityLearner;
use crate::merge::{merge_people, merge_publishers, merge_series, MergeStats};
use crate::traits::MLProcessable;
use ritmo_errors::RitmoResult;
use sqlx::SqlitePool;
use std::collections::HashMap;

/// Configuration for deduplication process
#[derive(Debug, Clone)]
pub struct DeduplicationConfig {
    /// Minimum confidence threshold for considering entities as duplicates (0.0-1.0)
    pub min_confidence: f64,

    /// Minimum frequency for pattern to be considered learned
    pub min_frequency: usize,

    /// If true, automatically merge high-confidence duplicates
    pub auto_merge: bool,

    /// If true, only identify duplicates without merging
    pub dry_run: bool,
}

impl Default for DeduplicationConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.90, // High confidence for safety
            min_frequency: 3,
            auto_merge: false,
            dry_run: true, // Safe default: don't auto-merge
        }
    }
}

/// A group of duplicate entities identified by ML
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    /// ID of the primary entity to keep
    pub primary_id: i64,

    /// Name/title of the primary entity
    pub primary_name: String,

    /// IDs of duplicate entities to merge
    pub duplicate_ids: Vec<i64>,

    /// Names/titles of duplicate entities
    pub duplicate_names: Vec<String>,

    /// Confidence score for this duplicate detection (0.0-1.0)
    pub confidence: f64,
}

/// Result of a deduplication operation
#[derive(Debug)]
pub struct DeduplicationResult {
    /// Total entities processed
    pub total_entities: usize,

    /// Duplicate groups identified
    pub duplicate_groups: Vec<DuplicateGroup>,

    /// Groups that were auto-merged (if auto_merge=true)
    pub merged_groups: Vec<MergeStats>,

    /// Groups skipped due to low confidence
    pub skipped_low_confidence: usize,
}

/// Find and optionally merge duplicate people (authors)
///
/// This function:
/// 1. Loads all people from database
/// 2. Uses ML clustering to identify duplicates
/// 3. Optionally merges duplicates based on config
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `config` - Deduplication configuration
pub async fn deduplicate_people(
    pool: &SqlitePool,
    config: &DeduplicationConfig,
) -> RitmoResult<DeduplicationResult> {
    // Step 1: Load all people from database
    let people = load_people_from_db(pool).await?;
    let total_entities = people.len();

    if people.is_empty() {
        return Ok(DeduplicationResult {
            total_entities: 0,
            duplicate_groups: Vec::new(),
            merged_groups: Vec::new(),
            skipped_low_confidence: 0,
        });
    }

    // Step 2: Extract canonical keys for clustering
    let canonical_keys: Vec<String> = people.iter().map(|p| p.canonical_key()).collect();

    // Step 3: Run ML clustering
    let mut learner = MLEntityLearner::new();
    learner.minimum_confidence = config.min_confidence;
    learner.minimum_frequency = config.min_frequency;
    learner.create_clusters(&canonical_keys);

    // Step 4: Convert clusters to duplicate groups
    let duplicate_groups = clusters_to_duplicate_groups(&learner, &people);

    // Step 5: Optionally merge duplicates
    let (merged_groups, skipped) = if !config.dry_run && config.auto_merge {
        merge_duplicate_people(pool, &duplicate_groups, config).await?
    } else {
        (Vec::new(), 0)
    };

    Ok(DeduplicationResult {
        total_entities,
        duplicate_groups,
        merged_groups,
        skipped_low_confidence: skipped,
    })
}

/// Find and optionally merge duplicate publishers
pub async fn deduplicate_publishers(
    pool: &SqlitePool,
    config: &DeduplicationConfig,
) -> RitmoResult<DeduplicationResult> {
    let publishers = load_publishers_from_db(pool).await?;
    let total_entities = publishers.len();

    if publishers.is_empty() {
        return Ok(DeduplicationResult {
            total_entities: 0,
            duplicate_groups: Vec::new(),
            merged_groups: Vec::new(),
            skipped_low_confidence: 0,
        });
    }

    let canonical_keys: Vec<String> = publishers.iter().map(|p| p.canonical_key()).collect();

    let mut learner = MLEntityLearner::new();
    learner.minimum_confidence = config.min_confidence;
    learner.minimum_frequency = config.min_frequency;
    learner.create_clusters(&canonical_keys);

    let duplicate_groups = clusters_to_duplicate_groups(&learner, &publishers);

    let (merged_groups, skipped) = if !config.dry_run && config.auto_merge {
        merge_duplicate_publishers(pool, &duplicate_groups, config).await?
    } else {
        (Vec::new(), 0)
    };

    Ok(DeduplicationResult {
        total_entities,
        duplicate_groups,
        merged_groups,
        skipped_low_confidence: skipped,
    })
}

/// Find and optionally merge duplicate series
pub async fn deduplicate_series(
    pool: &SqlitePool,
    config: &DeduplicationConfig,
) -> RitmoResult<DeduplicationResult> {
    let series = load_series_from_db(pool).await?;
    let total_entities = series.len();

    if series.is_empty() {
        return Ok(DeduplicationResult {
            total_entities: 0,
            duplicate_groups: Vec::new(),
            merged_groups: Vec::new(),
            skipped_low_confidence: 0,
        });
    }

    let canonical_keys: Vec<String> = series.iter().map(|s| s.canonical_key()).collect();

    let mut learner = MLEntityLearner::new();
    learner.minimum_confidence = config.min_confidence;
    learner.minimum_frequency = config.min_frequency;
    learner.create_clusters(&canonical_keys);

    let duplicate_groups = clusters_to_duplicate_groups(&learner, &series);

    let (merged_groups, skipped) = if !config.dry_run && config.auto_merge {
        merge_duplicate_series(pool, &duplicate_groups, config).await?
    } else {
        (Vec::new(), 0)
    };

    Ok(DeduplicationResult {
        total_entities,
        duplicate_groups,
        merged_groups,
        skipped_low_confidence: skipped,
    })
}

// ============================================================================
// Helper functions
// ============================================================================

/// Convert ML clusters into duplicate groups with entity details
fn clusters_to_duplicate_groups<T: MLProcessable>(
    learner: &MLEntityLearner,
    entities: &[T],
) -> Vec<DuplicateGroup> {
    // Build a map from canonical_key to entity for quick lookup
    let mut key_to_entity: HashMap<String, &T> = HashMap::new();
    for entity in entities {
        key_to_entity.insert(entity.canonical_key(), entity);
    }

    let mut groups = Vec::new();

    for cluster in &learner.clusters {
        if cluster.members.len() < 2 {
            continue; // Not a duplicate if only one member
        }

        // First member is the centroid (primary)
        let primary_key = &cluster.centroid;
        let primary_entity = match key_to_entity.get(primary_key) {
            Some(e) => e,
            None => continue, // Skip if entity not found
        };

        let primary_id = primary_entity.id();
        let primary_name = primary_key.clone();

        // Rest are duplicates
        let mut duplicate_ids = Vec::new();
        let mut duplicate_names = Vec::new();

        for member_key in &cluster.members {
            if member_key == primary_key {
                continue; // Skip primary itself
            }

            if let Some(dup_entity) = key_to_entity.get(member_key) {
                duplicate_ids.push(dup_entity.id());
                duplicate_names.push(member_key.clone());
            }
        }

        if !duplicate_ids.is_empty() {
            groups.push(DuplicateGroup {
                primary_id,
                primary_name,
                duplicate_ids,
                duplicate_names,
                confidence: cluster.confidence,
            });
        }
    }

    groups
}

/// Merge duplicate people based on duplicate groups
async fn merge_duplicate_people(
    pool: &SqlitePool,
    groups: &[DuplicateGroup],
    config: &DeduplicationConfig,
) -> RitmoResult<(Vec<MergeStats>, usize)> {
    let mut merged = Vec::new();
    let mut skipped = 0;

    for group in groups {
        if group.confidence < config.min_confidence {
            skipped += 1;
            continue;
        }

        match merge_people(pool, group.primary_id, &group.duplicate_ids).await {
            Ok(stats) => merged.push(stats),
            Err(e) => {
                eprintln!(
                    "Warning: failed to merge people group (primary={}, duplicates={:?}): {}",
                    group.primary_id, group.duplicate_ids, e
                );
                skipped += 1;
            }
        }
    }

    Ok((merged, skipped))
}

/// Merge duplicate publishers based on duplicate groups
async fn merge_duplicate_publishers(
    pool: &SqlitePool,
    groups: &[DuplicateGroup],
    config: &DeduplicationConfig,
) -> RitmoResult<(Vec<MergeStats>, usize)> {
    let mut merged = Vec::new();
    let mut skipped = 0;

    for group in groups {
        if group.confidence < config.min_confidence {
            skipped += 1;
            continue;
        }

        match merge_publishers(pool, group.primary_id, &group.duplicate_ids).await {
            Ok(stats) => merged.push(stats),
            Err(e) => {
                eprintln!(
                    "Warning: failed to merge publishers group (primary={}, duplicates={:?}): {}",
                    group.primary_id, group.duplicate_ids, e
                );
                skipped += 1;
            }
        }
    }

    Ok((merged, skipped))
}

/// Merge duplicate series based on duplicate groups
async fn merge_duplicate_series(
    pool: &SqlitePool,
    groups: &[DuplicateGroup],
    config: &DeduplicationConfig,
) -> RitmoResult<(Vec<MergeStats>, usize)> {
    let mut merged = Vec::new();
    let mut skipped = 0;

    for group in groups {
        if group.confidence < config.min_confidence {
            skipped += 1;
            continue;
        }

        match merge_series(pool, group.primary_id, &group.duplicate_ids).await {
            Ok(stats) => merged.push(stats),
            Err(e) => {
                eprintln!(
                    "Warning: failed to merge series group (primary={}, duplicates={:?}): {}",
                    group.primary_id, group.duplicate_ids, e
                );
                skipped += 1;
            }
        }
    }

    Ok((merged, skipped))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication_config_default() {
        let config = DeduplicationConfig::default();
        assert_eq!(config.min_confidence, 0.90);
        assert_eq!(config.min_frequency, 3);
        assert!(!config.auto_merge);
        assert!(config.dry_run);
    }

    #[tokio::test]
    async fn test_deduplicate_people() {
        use crate::test_helpers::*;

        // Create test database with duplicate people
        let pool = create_test_db().await.unwrap();
        populate_test_people(&pool).await.unwrap();
        populate_test_books_with_people(&pool).await.unwrap();

        // Configure deduplication with low confidence threshold to catch test duplicates
        let config = DeduplicationConfig {
            min_confidence: 0.80,
            min_frequency: 2,
            auto_merge: false, // Dry run only
            dry_run: true,
        };

        // Run deduplication
        let result = deduplicate_people(&pool, &config).await.unwrap();

        // Verify we found duplicates
        assert_eq!(result.total_entities, 12);
        assert!(!result.duplicate_groups.is_empty(), "Should find duplicate groups");

        // Verify duplicate groups structure
        for group in &result.duplicate_groups {
            assert!(group.duplicate_ids.len() >= 1, "Each group should have at least 1 duplicate");
            assert!(group.confidence >= config.min_confidence);
            assert!(!group.primary_name.is_empty());
        }

        // Since dry_run is true, no merges should have happened
        assert_eq!(result.merged_groups.len(), 0);

        // Verify database is unchanged (still 12 people)
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM people")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 12);
    }

    #[tokio::test]
    async fn test_deduplicate_people_with_auto_merge() {
        use crate::test_helpers::*;

        // Create test database with duplicate people
        let pool = create_test_db().await.unwrap();
        populate_test_people(&pool).await.unwrap();
        populate_test_books_with_people(&pool).await.unwrap();

        // Initial count
        let initial_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM people")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(initial_count, 12);

        // Configure deduplication with auto-merge enabled
        let config = DeduplicationConfig {
            min_confidence: 0.85,
            min_frequency: 2,
            auto_merge: true,
            dry_run: false,
        };

        // Run deduplication with merge
        let result = deduplicate_people(&pool, &config).await.unwrap();

        // Verify duplicates were found
        assert_eq!(result.total_entities, 12);
        assert!(!result.duplicate_groups.is_empty());

        // Verify some merges happened
        assert!(!result.merged_groups.is_empty(), "Should have merged some groups");

        // Verify database has fewer people now
        let final_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM people")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(final_count < initial_count, "People count should decrease after merging");

        // Verify merged groups have valid stats
        for merged in &result.merged_groups {
            assert!(merged.primary_id > 0);
            assert!(!merged.merged_ids.is_empty());
        }
    }
}
