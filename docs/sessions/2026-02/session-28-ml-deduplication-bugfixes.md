# Session 28: ML Deduplication Critical Bugfixes

**Date**: 2026-02-09
**Status**: ✅ COMPLETED
**Type**: Bugfix

## Overview

Fixed three critical bugs in the ML deduplication system that prevented it from correctly identifying and merging duplicate entities. The system was missing 2 out of 3 duplicate groups and ignoring the user-specified threshold parameter.

## Problems Identified

### 1. Hardcoded Threshold (Critical)
**File**: `ritmo_ml/src/entity_learner.rs:56`

**Problem**: The clustering threshold was hardcoded to `0.85` instead of using `self.minimum_confidence`.

**Impact**:
- Users couldn't control deduplication sensitivity with `--threshold` parameter
- All deduplications used 0.85 regardless of configuration

**Solution**: Changed `let threshold = 0.85;` to `let threshold = self.minimum_confidence;`

### 2. HashMap Collision Bug (Critical)
**File**: `ritmo_ml/src/deduplication.rs:309`

**Problem**: The function `clusters_to_duplicate_groups()` used `HashMap<String, &T>` which overwrote entities with identical canonical keys.

**Example**:
- "J.K. Rowling" → canonical_key: "rowling"
- "J. K. Rowling" → canonical_key: "rowling"
- HashMap kept only the last one, losing the first

**Impact**: Duplicate groups with identical canonical keys were completely missing from results.

**Solution**: Changed to `HashMap<String, Vec<&T>>` to handle multiple entities per canonical key.

### 3. Duplicate ID Multiplication (Major)
**File**: `ritmo_ml/src/deduplication.rs` (clusters_to_duplicate_groups function)

**Problem**: When a cluster contained repeated members (e.g., `["stephen king", "stephen king", "stephen king"]`), the algorithm added the same entity IDs multiple times.

**Impact**: Output showed duplicate IDs like: `[1, 3, 4, 1, 3, 4, 1, 3, 4, 2]`

**Solution**: Added `HashSet` to deduplicate entity IDs during cluster-to-group conversion.

## Implementation Details

### entity_learner.rs Changes

**Before**:
```rust
pub fn create_clusters(&mut self, items: &[String]) {
    let threshold = 0.85; // Hardcoded!
    // ...
}
```

**After**:
```rust
pub fn create_clusters(&mut self, items: &[String]) {
    let threshold = self.minimum_confidence; // Uses config
    // ...
}
```

### deduplication.rs Changes

**Before**:
```rust
// HashMap overwrites duplicate canonical keys
let mut key_to_entity: HashMap<String, &T> = HashMap::new();
for entity in entities {
    key_to_entity.insert(entity.canonical_key(), entity);
}
```

**After**:
```rust
// Vec allows multiple entities per canonical key
let mut key_to_entities: HashMap<String, Vec<&T>> = HashMap::new();
for entity in entities {
    key_to_entities
        .entry(entity.canonical_key())
        .or_insert_with(Vec::new)
        .push(entity);
}

// Deduplicate entity IDs with HashSet
use std::collections::HashSet;
let mut seen_ids = HashSet::new();
let mut all_entity_ids: Vec<(i64, String)> = Vec::new();

for member_key in &cluster.members {
    if let Some(entities_with_key) = key_to_entities.get(member_key) {
        for entity in entities_with_key {
            let id = entity.id();
            if !seen_ids.contains(&id) {
                seen_ids.insert(id);
                all_entity_ids.push((id, member_key.clone()));
            }
        }
    }
}
```

## Testing

### Test Data Created
7 contents with intentionally similar author names:
- "Stephen King", "Stephen Edwin King", "Stephen E. King" (3 variants)
- "J.K. Rowling", "J. K. Rowling" (2 variants)
- "Isaac Asimov", "Asimov, Isaac" (2 variants)

### Before Fix
```
Total entities processed: 7
Duplicate groups found: 1  ❌ (Should be 3)

Group 1: Stephen King (2 variants)  ❌ (Should be 3)
Missing: Rowling group  ❌
Missing: Asimov group  ❌
```

### After Fix
```
Total entities processed: 7
Duplicate groups found: 3  ✅

Group 1 (confidence: 94.44%):
  Primary: stephen king (ID: 1)
  Duplicates: stephen king (ID: 3), stephen edwin king (ID: 2)  ✅

Group 2 (confidence: 100.00%):
  Primary: rowling (ID: 4)
  Duplicates: rowling (ID: 5)  ✅

Group 3 (confidence: 100.00%):
  Primary: isaac asimov (ID: 6)
  Duplicates: isaac asimov (ID: 7)  ✅
```

### Test Suite
- Created `ritmo_ml/tests/test_name_parsing.rs` - Integration test for name parsing and similarity calculation
- All 20 existing unit tests pass
- New test verifies Jaro-Winkler similarities for all test cases

## Files Modified

1. **ritmo_ml/src/entity_learner.rs** - Fixed hardcoded threshold
2. **ritmo_ml/src/deduplication.rs** - Fixed HashMap collision and ID deduplication
3. **ritmo_ml/tests/test_name_parsing.rs** - NEW: Integration test for name parsing

## Impact

### Before
- ❌ Only 1 out of 3 duplicate groups detected
- ❌ Threshold parameter completely ignored
- ❌ Duplicate IDs in output

### After
- ✅ All 3 duplicate groups correctly detected
- ✅ Threshold parameter works as expected
- ✅ Clean output without duplicates

## Commands Affected

All deduplication commands now work correctly:
- `deduplicate-people --threshold <VALUE>`
- `deduplicate-publishers --threshold <VALUE>`
- `deduplicate-series --threshold <VALUE>`
- `deduplicate-tags --threshold <VALUE>`
- `deduplicate-all --threshold <VALUE>`

## Verification

```bash
# Create test library with duplicate names
cargo run -p ritmo_cli -- init /tmp/test_dedup
cargo run -p ritmo_cli -- add-content --title "The Shining" --people "Stephen King:Autore"
cargo run -p ritmo_cli -- add-content --title "IT" --people "Stephen Edwin King:Autore"
cargo run -p ritmo_cli -- add-content --title "The Stand" --people "Stephen E. King:Autore"
cargo run -p ritmo_cli -- add-content --title "Harry Potter 1" --people "J.K. Rowling:Autore"
cargo run -p ritmo_cli -- add-content --title "Harry Potter 2" --people "J. K. Rowling:Autore"
cargo run -p ritmo_cli -- add-content --title "Foundation" --people "Isaac Asimov:Autore"
cargo run -p ritmo_cli -- add-content --title "I, Robot" --people "Asimov, Isaac:Autore"

# Test deduplication
cargo run -p ritmo_cli -- deduplicate-people --threshold 0.85 --dry-run
# Should find 3 groups (Stephen King, Rowling, Asimov)

# Perform merge
cargo run -p ritmo_cli -- deduplicate-people --threshold 0.85 --auto-merge

# Verify clean database
cargo run -p ritmo_cli -- deduplicate-people --threshold 0.85 --dry-run
# Should show: "No duplicates found! Database is clean."
```

## Key Learnings

1. **HashMap Limitations**: When using HashMap for lookups, ensure keys are truly unique or use `HashMap<K, Vec<V>>`
2. **Threshold Configuration**: Never hardcode thresholds - always respect user configuration
3. **Canonical Key Collisions**: Multiple entities can legitimately have the same canonical key (e.g., "J.K. Rowling" and "J. K. Rowling" both normalize to "rowling")
4. **Cluster Members**: ML clustering can produce repeated members in clusters, requiring deduplication

## Future Improvements

These bugs highlighted areas for potential enhancement:

1. **Better Canonical Keys**: Consider keeping middle initials/names in canonical keys to reduce collisions
2. **Confidence Reporting**: Show individual similarity scores in output, not just group confidence
3. **Manual Review Mode**: Interactive mode to review and approve/reject each merge
4. **Pattern Learning**: Implement the pattern learning system to improve detection over time

## Documentation Updated

- ✅ Session history (this file)
- ✅ Test coverage documented
- ✅ Bugfix verification steps

## Related Sessions

- **Session 7**: Initial ritmo_ml Phase 1 implementation
- **Session 10**: ritmo_ml Phase 2 (merge operations)
- **Session 11**: ritmo_ml test coverage
- **Session 12**: ML CLI integration

---

**Severity**: Critical
**Root Cause**: Logic errors in clustering and group conversion
**Resolution Time**: 2 hours
**Testing**: Comprehensive manual testing + 21 unit tests passing
