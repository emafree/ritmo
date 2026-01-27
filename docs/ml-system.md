# Machine Learning System Documentation

This document describes the ritmo_ml crate and its entity deduplication system.

## Overview

The `ritmo_ml` crate provides machine learning features for entity deduplication, allowing ritmo to identify and merge duplicate authors, publishers, series, and tags automatically.

## Core Features

### 1. Pattern Classification System

The system classifies variant patterns into 7 types:

- **Abbreviation**: "J.R.R. Tolkien" ← "John Ronald Reuel Tolkien"
- **Prefix**: "Dr. Smith" ← "Smith"
- **Suffix**: "Smith Jr." ← "Smith"
- **Compound**: "Stephen King" ← "King, Stephen"
- **Transliteration**: "Dostoyevsky" ← "Dostoevskij"
- **Typo**: small edit distance variations
- **Other**: unclassified patterns

### 2. Confidence Scoring

Smart confidence scoring with bonuses and penalties:

- Bonus for abbreviations with matching initials
- Penalty for large edit distance (>3)
- Penalty for length difference >50%

### 3. ML Entity Learner

Clustering and pattern detection:

- **Clustering**: Uses Jaro-Winkler similarity (threshold 0.85)
- **Pattern Detection**: Customizable pattern identification
- **Serializable**: Save/load ML data to database

## Deduplication Workflow

### Configuration

```rust
use ritmo_ml::deduplication::DeduplicationConfig;

let config = DeduplicationConfig {
    min_confidence: 0.90,     // High confidence for safety
    min_frequency: 3,         // Minimum pattern frequency
    auto_merge: false,        // Requires manual approval by default
    dry_run: true,            // Preview mode by default
};
```

### Deduplication Steps

1. **Load** all entities from database
2. **Extract** canonical keys for ML comparison
3. **Cluster** using Jaro-Winkler similarity
4. **Identify** duplicate groups with confidence scores
5. **Merge** high-confidence duplicates (if auto_merge=true)
6. **Return** detailed results and statistics

### Example Usage

```rust
use ritmo_ml::deduplication::{deduplicate_people, DeduplicationConfig};

// Create configuration
let config = DeduplicationConfig {
    min_confidence: 0.90,
    min_frequency: 3,
    auto_merge: false,
    dry_run: true,
};

// Run deduplication
let pool = config.create_pool().await?;
let result = deduplicate_people(&pool, &config).await?;

// Review results
println!("Total entities: {}", result.total_entities);
println!("Duplicate groups found: {}", result.duplicate_groups.len());
println!("Skipped (low confidence): {}", result.skipped_low_confidence);

// Review each duplicate group
for group in result.duplicate_groups {
    println!("Primary: {} (ID: {})", group.primary_name, group.primary_id);
    println!("Confidence: {:.2}", group.confidence);
    for (id, name) in group.duplicate_ids.iter().zip(&group.duplicate_names) {
        println!("  - {} (ID: {})", name, id);
    }
}
```

## CLI Usage

The ritmo CLI provides five commands for ML-based deduplication:

### Command Overview

```bash
# Find duplicate people (authors, translators, etc.) - dry-run mode (safe preview)
ritmo deduplicate-people --dry-run

# Find duplicate publishers
ritmo deduplicate-publishers --dry-run

# Find duplicate series
ritmo deduplicate-series --dry-run

# Find duplicate tags
ritmo deduplicate-tags --dry-run

# Run deduplication for all entity types
ritmo deduplicate-all --dry-run
```

### Command Options

All deduplication commands support these options:

- `--threshold <VALUE>` - Minimum confidence threshold (0.0-1.0), default: 0.85
- `--auto-merge` - Automatically merge high-confidence duplicates (requires `--no-dry-run`)
- `--dry-run` - Preview mode, shows what would be merged without making changes (default: true)

### Examples

```bash
# Preview duplicate people with default threshold (0.85)
cargo run -p ritmo_cli -- deduplicate-people --dry-run

# Find duplicate people with higher confidence threshold
cargo run -p ritmo_cli -- deduplicate-people --threshold 0.90 --dry-run

# Actually merge duplicate people (CAUTION: modifies database!)
cargo run -p ritmo_cli -- deduplicate-people --threshold 0.90 --auto-merge

# Find duplicate tags
cargo run -p ritmo_cli -- deduplicate-tags --dry-run
cargo run -p ritmo_cli -- deduplicate-tags --threshold 0.85 --auto-merge

# Find duplicates across all entity types (people, publishers, series, tags, roles)
cargo run -p ritmo_cli -- deduplicate-all --threshold 0.85 --dry-run

# Merge all duplicates with high confidence
cargo run -p ritmo_cli -- deduplicate-all --threshold 0.92 --auto-merge
```

### Output Format

The commands provide detailed output:

```
📊 Deduplication Results for Authors:
   Total entities processed: 150
   Duplicate groups found: 8

📋 Duplicate Groups:

   Group 1 (confidence: 95.40%):
     Primary: stephen king (ID: 1)
     Duplicates:
       1. Stephen King (ID: 42)
       2. STEPHEN KING (ID: 89)

   Group 2 (confidence: 88.20%):
     Primary: j.k. rowling (ID: 5)
     Duplicates:
       1. J. K. Rowling (ID: 73)
       2. Joanne Rowling (ID: 105)

🔍 Dry-run mode: No changes were made to the database
```

### Safety Recommendations

1. **Always use `--dry-run` first** to preview what would be merged
2. **Review the duplicate groups** and their confidence scores carefully
3. **Start with high threshold** (0.90+) for automatic merges
4. **Back up your database** before running auto-merge operations
5. **Test on a copy** of your library before running on production data

## Database Loaders

Load entities from database with normalization:

```rust
use ritmo_ml::db_loaders::{load_people_from_db, load_publishers_from_db, load_series_from_db};

let people = load_people_from_db(&pool).await?;
let publishers = load_publishers_from_db(&pool).await?;
let series = load_series_from_db(&pool).await?;
```

## Merge Operations

Safely merge duplicate entities:

```rust
use ritmo_ml::merge::{merge_people, merge_publishers, merge_series};

// Merge duplicate authors (with transaction)
let stats = merge_people(&pool, primary_id, vec![dup_id_1, dup_id_2]).await?;

println!("Merged IDs: {:?}", stats.merged_ids);
println!("Books updated: {}", stats.books_updated);
println!("Contents updated: {}", stats.contents_updated);
```

### Safety Features

- **Transactions**: All operations atomic with rollback on error
- **Validation**: Check all IDs exist before merge
- **Update all references**: Foreign keys and junction tables updated atomically
- **Error resilience**: Skip failed merges, continue with rest
- **Detailed logging**: Track all operations and failures

## Entity Records

### PersonRecord
```rust
use ritmo_ml::people::PersonRecord;

let person = PersonRecord::new("Stephen King", "en", &string_utils);
// Handles: first_name, last_name, initials, aliases
```

### PublisherRecord
```rust
use ritmo_ml::publishers::PublisherRecord;

let publisher = PublisherRecord {
    id: 1,
    name: "Penguin Books".to_string(),
    canonical_name: "penguin books".to_string(),
    variants: vec![],
};
```

### SeriesRecord
```rust
use ritmo_ml::series::SeriesRecord;

let series = SeriesRecord {
    id: 1,
    title: "The Dark Tower".to_string(),
    canonical_title: "the dark tower".to_string(),
    variants: vec![],
};
```

## Deduplication Results

### DeduplicationResult
- `total_entities: usize` - Count of entities processed
- `duplicate_groups: Vec<DuplicateGroup>` - List of duplicate groups
- `merged_groups: Vec<MergeStats>` - Statistics (if auto_merge=true)
- `skipped_low_confidence: usize` - Count of groups below threshold

### DuplicateGroup
- `primary_id: i64` - Entity to keep
- `primary_name: String` - Name of primary entity
- `duplicate_ids: Vec<i64>` - Entities to merge
- `duplicate_names: Vec<String>` - Names of duplicates
- `confidence: f64` - ML confidence score (0.0-1.0)

### MergeStats
- `primary_id: i64` - Primary entity ID
- `merged_ids: Vec<i64>` - IDs that were merged
- `books_updated: usize` - Number of books affected
- `contents_updated: usize` - Number of contents affected

## Design Principles

### Safety First
- **Dry-run by default**: No accidental data loss
- **High confidence threshold**: 0.90 for auto-merge
- **Transactional merges**: Atomic operations with rollback

### Configurability
- Adjust thresholds for different use cases
- Custom pattern classification functions
- Custom confidence scoring functions

### Performance
- Jaro-Winkler similarity for fast clustering
- Unicode NFC normalization
- Efficient database queries

## CLI Integration (COMPLETED)

The following commands are now available:
```bash
# Deduplicate people (authors, translators, etc.)
ritmo deduplicate-people --dry-run
ritmo deduplicate-people --auto-merge --threshold 0.95

# Deduplicate publishers
ritmo deduplicate-publishers --dry-run
ritmo deduplicate-publishers --auto-merge

# Deduplicate series
ritmo deduplicate-series --dry-run
ritmo deduplicate-series --auto-merge

# Deduplicate tags
ritmo deduplicate-tags --dry-run

# Deduplicate all entities
ritmo deduplicate-all --dry-run
```

## Testing

The `ritmo_ml` crate has comprehensive test coverage with **17 unit tests** covering all major functionality.

### Test Infrastructure

The test suite uses an in-memory SQLite database with realistic test data:

```rust
use ritmo_ml::test_helpers::*;

// Create test database with sample data
let pool = create_test_db().await?;
populate_test_people(&pool).await?;  // 12 people with duplicates
populate_test_publishers(&pool).await?;  // 9 publishers with duplicates
populate_test_series(&pool).await?;  // 8 series with duplicates
```

### Test Categories

**Database Loaders (4 tests)**
- `test_load_people_from_db` - Loads 12 people, verifies normalization
- `test_load_publishers_from_db` - Loads 9 publishers
- `test_load_series_from_db` - Loads 8 series
- `test_load_tags_from_db` - Loads 8 tags

**Merge Operations (4 tests)**
- `test_merge_people` - Merges Stephen King variants, verifies relationship updates
- `test_merge_publishers` - Merges publisher duplicates
- `test_merge_series` - Merges series duplicates
- `test_merge_people_validation_errors` - Tests error handling

**Deduplication Workflow (2 tests)**
- `test_deduplicate_people` - End-to-end dry-run workflow
- `test_deduplicate_people_with_auto_merge` - Auto-merge workflow with verification

**Pattern Functions (7 tests)**
- Classification and confidence scoring tests
- Abbreviation, prefix, suffix, typo detection
- Initials matching and penalties

### Test Data

Test databases include realistic duplicates:
- **People**: "Stephen King", "Stephen Edwin King", "King, Stephen", "S. King"
- **Publishers**: "Penguin Random House", "Penguin Books", "Random House"
- **Series**: "The Dark Tower", "Dark Tower"

### Running Tests

```bash
# Run all tests
cargo test -p ritmo_ml

# Run with output
cargo test -p ritmo_ml -- --nocapture

# Include ignored tests (none currently)
cargo test -p ritmo_ml -- --include-ignored
```

All tests run in milliseconds using in-memory databases.

## Implementation History

- **Session 7 (2025-12-18)**: Phase 1 - Core ML infrastructure
  - MLProcessable trait implementation
  - Pattern classification system
  - MLEntityLearner enhancements
  - 6 unit tests for pattern functions
  - All workspace tests passing (59)

- **Session 10 (2025-12-18)**: Phase 2 - End-to-end workflow
  - Database loaders (~190 lines)
  - Merge operations (~410 lines)
  - Deduplication workflow (~380 lines)
  - Safety features and error handling
  - Comprehensive documentation

- **Session 11 (2026-01-25)**: Test Coverage Complete
  - Created `test_helpers.rs` module with in-memory test databases
  - Implemented 9 comprehensive integration tests
  - Added realistic test data with duplicates
  - All 17 tests passing (previously 8 were empty/ignored)
  - Full test coverage for db_loaders, merge, and deduplication modules

For detailed session history, see [Session Documentation](sessions/).
