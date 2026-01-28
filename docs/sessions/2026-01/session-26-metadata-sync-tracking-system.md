## 2026-01-28 - Session 26: Metadata Sync Tracking System

**Context**
After implementing EPUB OPF metadata modification (Session 25), EPUBs are updated during import. However, when duplicate entities are merged after import (e.g., deduplicating authors), the EPUB files become out of sync with the database. The user requested a tracking system to mark affected books and sync their metadata.

**Objective**
Implement a complete metadata sync tracking system:
1. Track books requiring metadata sync after entity merges
2. Mark affected books automatically during deduplication
3. Provide CLI command to sync EPUB files with database metadata
4. Update EPUBs, recalculate hash, and move files to new paths

**User Requirements** (confirmed via AskUserQuestion):
1. **Sync Direction**: DB → EPUB (database is source of truth)
2. **Hash Management**: Recalculate hash and move file to new path (not preserve old hash)
3. **OPF Backup**: Leave original in `originals_opf/` unchanged

**Implementation**

✅ **Step 1: Database Schema** (`ritmo_db/schema/schema.sql`)

Added `pending_metadata_sync` table:
```sql
CREATE TABLE IF NOT EXISTS "pending_metadata_sync" (
    "id"          INTEGER,
    "book_id"     INTEGER NOT NULL,
    "reason"      TEXT NOT NULL,
    "created_at"  INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY("id" AUTOINCREMENT),
    FOREIGN KEY("book_id") REFERENCES "books"("id") ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS "idx_pending_sync_book_lookup"
ON "pending_metadata_sync" ("book_id");
```

**Reason Values**:
- `"author_deduplicate"` - After merging duplicate authors
- `"publisher_deduplicate"` - After merging duplicate publishers
- `"series_deduplicate"` - After merging duplicate series
- `"tag_deduplicate"` - After merging duplicate tags
- `"role_deduplicate"` - After merging duplicate roles
- `"manual_edit"` - After manual metadata edit (future)

Regenerated `ritmo_db_core/assets/template.db` from updated schema.

✅ **Step 2: Database Helper Functions** (`ritmo_db/src/models/pending_sync.rs`, 58 lines)

Created helper functions:
```rust
pub async fn mark_book_for_sync(pool: &SqlitePool, book_id: i64, reason: &str) -> RitmoResult<()>
pub async fn mark_books_for_sync(pool: &SqlitePool, book_ids: &[i64], reason: &str) -> RitmoResult<()>
pub async fn get_pending_sync_books(pool: &SqlitePool) -> RitmoResult<Vec<i64>>
pub async fn count_pending_sync(pool: &SqlitePool) -> RitmoResult<i64>
pub async fn clear_sync_mark(pool: &SqlitePool, book_id: i64) -> RitmoResult<()>
```

✅ **Step 3: Merge Operations Enhancement** (`ritmo_ml/src/merge.rs`)

Modified `MergeStats` struct:
```rust
pub struct MergeStats {
    pub primary_id: i64,
    pub merged_ids: Vec<i64>,
    pub books_updated: usize,
    pub contents_updated: usize,
    pub affected_book_ids: Vec<i64>,  // NEW
}
```

Updated all merge functions to capture affected book IDs:
- `update_books_people_roles()` → Returns `(usize, Vec<i64>)`
- `update_contents_people_roles()` → Returns `(usize, Vec<i64>)` (via x_books_contents)
- `update_books_publisher()` → Returns `(usize, Vec<i64>)`
- `update_books_series()` → Returns `(usize, Vec<i64>)`
- `update_books_tags()` → Returns `(usize, Vec<i64>)`
- `update_contents_tags()` → Returns `(usize, Vec<i64>)` (via x_books_contents)
- `update_books_people_roles_role()` → Returns `(usize, Vec<i64>)`
- `update_contents_people_roles_role()` → Returns `(usize, Vec<i64>)` (via x_books_contents)

All affected book IDs are collected, sorted, and deduplicated before returning.

✅ **Step 4: Deduplication Commands Integration** (`ritmo_cli/src/commands/deduplication.rs`)

All deduplicate commands now mark affected books:
```rust
// After successful merge (if not dry-run)
if !actual_dry_run && !result.merged_groups.is_empty() {
    let mut all_affected_books = Vec::new();
    for stats in &result.merged_groups {
        all_affected_books.extend(&stats.affected_book_ids);
    }
    all_affected_books.sort();
    all_affected_books.dedup();

    if !all_affected_books.is_empty() {
        mark_books_for_sync(&pool, &all_affected_books, "author_deduplicate").await?;
        println!("\n📝 Marked {} books for metadata sync", all_affected_books.len());
        println!("   Run 'ritmo sync-metadata' to update EPUB files with new metadata");
    }
}
```

Applied to all commands:
- `cmd_deduplicate_people()` → `"author_deduplicate"`
- `cmd_deduplicate_publishers()` → `"publisher_deduplicate"`
- `cmd_deduplicate_series()` → `"series_deduplicate"`
- `cmd_deduplicate_tags()` → `"tag_deduplicate"`
- `cmd_deduplicate_roles()` → `"role_deduplicate"`

✅ **Step 5: Metadata Sync Service** (`ritmo_core/src/service/metadata_sync_service.rs`, ~350 lines)

Created complete sync service with:

**Main Function**:
```rust
pub async fn sync_book_metadata(
    config: &LibraryConfig,
    pool: &sqlx::SqlitePool,
    book_id: i64,
) -> RitmoResult<SyncResult>
```

**Sync Workflow**:
1. Read book metadata from DB (book + contents + relations)
2. Build `BookImportMetadata` from DB data
3. Read all contents associated with book
4. Build `OPFMetadata` using existing `build_opf_metadata()`
5. Modify EPUB using existing `modify_epub_metadata()`
6. Calculate new SHA256 hash
7. Determine new hash-based path: `books/{hash[0:2]}/{hash[2:4]}/{hash[4:]}.epub`
8. Move file to new location
9. Delete old file if different location
10. Update DB with new `file_hash` and `file_link`
11. Clear sync mark

**Helper Functions**:
- `build_book_metadata_from_db()` - Reconstruct metadata from DB
- `get_book_contents()` - Get all contents with people, languages, types
- `calculate_hash()` - SHA256 hash calculation

**Return Type**:
```rust
pub struct SyncResult {
    pub book_id: i64,
    pub old_hash: String,
    pub new_hash: String,
    pub old_path: PathBuf,
    pub new_path: PathBuf,
}
```

✅ **Step 6: CLI Command** (`ritmo_cli/src/commands/sync.rs`, ~135 lines)

Created `sync-metadata` command with 3 modes:

**1. Status Mode** (`--status`):
```bash
ritmo sync-metadata --status
```
Shows count of books pending sync.

**2. Dry-Run Mode** (`--dry-run`):
```bash
ritmo sync-metadata --dry-run
```
Lists books that would be synchronized without making changes.

**3. Sync Mode** (default):
```bash
ritmo sync-metadata
```
Actually synchronizes all pending books:
- Shows progress: `[1/15] Syncing book ID 42... ✓`
- Displays old/new hash (first 16 chars)
- Shows file moves when hash changes
- Provides summary: successful/failed counts

**CLI Integration** (`ritmo_cli/src/main.rs`):

Added command to enum:
```rust
SyncMetadata {
    #[arg(long)]
    status: bool,

    #[arg(long)]
    dry_run: bool,

    #[arg(short, long)]
    library: Option<PathBuf>,
}
```

Match arm routes to appropriate handler based on flags.

**Architecture**

```
┌─────────────────────────────────────────────────────┐
│ TRIGGER: Deduplicate merges duplicate entities      │
└───────────────────┬─────────────────────────────────┘
                    │
                    v
┌─────────────────────────────────────────────────────┐
│ MARK: Insert affected book_ids into                 │
│       pending_metadata_sync table                   │
└───────────────────┬─────────────────────────────────┘
                    │
                    v
┌─────────────────────────────────────────────────────┐
│ USER: Run `ritmo sync-metadata` command              │
└───────────────────┬─────────────────────────────────┘
                    │
                    v
┌─────────────────────────────────────────────────────┐
│ SYNC: For each pending book:                        │
│   1. Read all metadata from DB                      │
│   2. Build OPFMetadata                              │
│   3. Modify EPUB OPF                                │
│   4. Calculate new hash                             │
│   5. Move file: old_path → new_path                │
│   6. Update DB: file_hash, file_link               │
│   7. Clear sync mark                                │
└─────────────────────────────────────────────────────┘
```

**Usage Examples**

```bash
# 1. Deduplicate authors (marks affected books)
$ ritmo deduplicate-people --auto-merge --threshold 0.90

📊 Deduplication Results for People:
   Total entities processed: 127
   Duplicate groups found: 3

✓ Merged 3 groups:
   1. Primary ID 1: merged 3 duplicates (15 books, 0 contents updated)

📝 Marked 15 books for metadata sync
   Run 'ritmo sync-metadata' to update EPUB files with new metadata

# 2. Check sync status
$ ritmo sync-metadata --status

📊 Books pending metadata sync: 15

Run 'ritmo sync-metadata' to sync EPUB files with database metadata
Run 'ritmo sync-metadata --dry-run' to preview changes

# 3. Preview changes (dry-run)
$ ritmo sync-metadata --dry-run

🔍 Dry-run: 15 books would be synchronized

Books that would be updated:
  • [42] The Shining
  • [43] IT
  • [44] The Stand
  ...

⚠️  Dry-run mode: No changes were made
Run 'ritmo sync-metadata' without --dry-run to perform sync

# 4. Actually sync metadata
$ ritmo sync-metadata

🔄 Synchronizing metadata for 15 books...

[1/15] Syncing book ID 42... ✓
  Old hash: d121b095fd222ac6
  New hash: a7f3c4e8912bd56f
  Moved: d121...3189.epub → a7f3...56f.epub
[2/15] Syncing book ID 43... ✓
  Old hash: e234c5f1ab34de78
  New hash: b8g4d5f9a23ce67g
  Moved: e234...de78.epub → b8g4...67g.epub
...

📊 Sync Summary:
  ✓ Successful: 15
```

**Key Features**

1. **Automatic Tracking**: Books marked automatically during deduplication
2. **Hash Recalculation**: EPUBs get new hash after metadata modification
3. **File Movement**: Files moved to new hash-based paths
4. **Database Updates**: `file_hash` and `file_link` updated in DB
5. **OPF Backup Preservation**: Original OPF in `originals_opf/` unchanged
6. **Graceful Error Handling**: Failed syncs don't clear marks (retry possible)
7. **CASCADE Deletion**: Sync marks auto-deleted when book is deleted
8. **Progress Reporting**: Real-time progress with detailed output

**Files Modified/Created**

**New Files**:
- `ritmo_db/src/models/pending_sync.rs` (58 lines)
- `ritmo_core/src/service/metadata_sync_service.rs` (~350 lines)
- `ritmo_cli/src/commands/sync.rs` (~135 lines)

**Modified Files**:
- `ritmo_db/schema/schema.sql` (added table + index)
- `ritmo_db_core/assets/template.db` (regenerated from schema)
- `ritmo_db/src/models/mod.rs` (export pending_sync)
- `ritmo_core/src/service/mod.rs` (export metadata_sync_service)
- `ritmo_ml/src/merge.rs` (updated MergeStats, return affected_book_ids)
- `ritmo_cli/src/commands/deduplication.rs` (mark books after merge)
- `ritmo_cli/src/commands/mod.rs` (export sync commands)
- `ritmo_cli/src/main.rs` (add SyncMetadata command)

**Build Status**
✅ Full workspace build successful
- All crates compile without errors
- Only minor warnings about unused code

**Testing Strategy**

Recommended tests:
1. Deduplicate marks books correctly
2. Dry-run shows correct preview
3. Sync updates EPUB and recalculates hash
4. File moves to new path correctly
5. DB fields updated correctly
6. Sync mark cleared after success
7. Failed sync preserves mark for retry

**Impact**
The metadata sync tracking system ensures EPUB files remain consistent with database metadata after entity deduplication. The DB → EPUB sync direction establishes the database as the single source of truth, with EPUB files automatically updated to match.

**Future Enhancements**
- Manual metadata edit could also mark books for sync
- Bulk sync with parallelization for large libraries
- Sync statistics and history tracking
- Rollback capability for failed syncs

---

_For December 2025 sessions, see [2025-12-sessions.md](2025-12-sessions.md)_
