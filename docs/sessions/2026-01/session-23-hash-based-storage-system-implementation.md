## 2026-01-27 - Session 23: Hash-Based Storage System Implementation

**Context**
The previous storage system used human-readable filenames stored in a flat `storage/books/` directory. This approach had several limitations: poor scalability with large collections, possible filename collisions, inefficient filesystem performance with many files, and no content-based organization.

**Problem**
- **Scalability**: Flat directory structure becomes slow with thousands of files
- **Collisions**: Files with identical names would overwrite each other
- **No deduplication**: Same file imported twice would be stored twice
- **Human-readable not needed**: Users interact through database, not filesystem
- **Unused code**: `storage_service.rs` and `Book::set_book_persistence()` implemented metadata-based hashing but were not used

**Objective**
Implement a content-addressed, hash-based storage system using SHA256 file hashes for optimal performance, automatic deduplication, and scalability to millions of books.

**Implementation**

✅ **File Storage Architecture**:
- **Path Format**: `books/{hash[0:2]}/{hash[2:4]}/{hash[4:]}.{extension}`
- **Example**: Hash `d121b095fd222ac6d4f13eebaba7a3d08fe35fee3189b996d6020b3365c27252`
  - Stored as: `books/d1/21/b095fd222ac6d4f13eebaba7a3d08fe35fee3189b996d6020b3365c27252.epub`
- **Distribution**: 256×256 = 65,536 subdirectories
- **Performance**: With 1M books, ~15 files per directory (optimal)

✅ **ritmo_core/src/service/book_import_service.rs** (lines 104-119):
- Replaced filename-based path generation with hash-based generation
- Changed from: `format!("books/{}", file_name)`
- Changed to: `format!("books/{}/{}/{}.{}", &hash[0..2], &hash[2..4], &hash[4..], ext)`
- File hash already calculated for duplicate detection, reused for path generation
- Preserves file extension for format identification

✅ **Code Cleanup**:
- **Removed**: `ritmo_core/src/service/storage_service.rs` (unused, only called from nowhere)
- **Removed**: `ritmo_core/src/service/mod.rs` line 7 (module declaration)
- **Removed**: `ritmo_db/src/models/books.rs` lines 125-176 (`set_book_persistence()` function)
- **Removed**: `ritmo_db/src/models/books.rs` line 1 (`use sha2::Digest;` no longer needed)
- **Reason**: These implemented metadata-based hashing (not content-based) and were never used in the import workflow

✅ **Batch Import**:
- No changes needed in `batch_import_service.rs`
- Already calls `import_book()` which now uses hash-based paths
- Works automatically with new system

✅ **Database Migration**:
- Deleted existing database (23 books with old naming system)
- Reinitialized fresh library with hash-based storage
- No migration script needed (clean slate approach per user request)

**Testing**:
- ✅ Single book import: File stored in hash-based path `d1/21/b095fd222...epub`
- ✅ Multiple imports: Different hashes stored in different directories `3f/3e/e058cf096...epub`
- ✅ Duplicate detection: SHA256 hash comparison prevents duplicate imports
- ✅ List books: `list-books` displays all imported books correctly
- ✅ Delete with file: Physical file removed from hash-based path
- ✅ Compilation: `cargo build --workspace` successful
- ✅ Directory structure: Verified 2-level hierarchy created automatically

**Benefits**:

1. **Performance**:
   - O(1) lookup with known hash
   - Efficient filesystem performance (optimal file distribution)
   - Scales to millions of books without degradation

2. **Deduplication**:
   - Content-addressed: same file = same path
   - Automatic at filesystem level
   - SHA256 collision probability: ~1 in 2^256 (essentially impossible)

3. **Integrity**:
   - Hash stored in database (`books.file_hash`)
   - Can verify file integrity by recalculating hash
   - Detect file corruption or tampering

4. **Portability**:
   - Relative paths stored in database (`books.file_link`)
   - Library can be moved/copied without breaking references
   - Cross-platform compatible

5. **Scalability**:
   - 65,536 subdirectories distribute load
   - No single directory bottleneck
   - Works efficiently with any collection size

**Storage Path Examples**:
```
books/d1/21/b095fd222ac6d4f13eebaba7a3d08fe35fee3189b996d6020b3365c27252.epub
books/3f/3e/e058cf096e3ab079537c3bc168002e4d3acc656ff56b75268173fa9443c0.epub
```

**Database Records**:
```sql
id=2, file_link='books/d1/21/b095fd222...epub',
      file_hash='d121b095fd222ac6d4f13eebaba7a3d08fe35fee3189b996d6020b3365c27252'

id=3, file_link='books/3f/3e/e058cf096...epub',
      file_hash='3f3ee058cf096e3ab079537c3bc168002e4d3acc656ff56b75268173fa9443c0'
```

**Files Modified**:
- `ritmo_core/src/service/book_import_service.rs` (path generation logic)
- `ritmo_core/src/service/mod.rs` (removed storage_service module)
- `ritmo_db/src/models/books.rs` (removed set_book_persistence function)
- `docs/architecture.md` (comprehensive File Storage System section)
- `CLAUDE.md` (Session 23 summary)

**Files Removed**:
- `ritmo_core/src/service/storage_service.rs` (37 lines, unused code)

**Documentation**:
- Added comprehensive "File Storage System" section in `docs/architecture.md`
- Includes: path structure, key features, benefits, implementation details, workflow
- Updated Library Structure diagram with hash-based directory tree
- Updated ritmo_core services list

**Outcome**:
Ritmo now uses a production-ready, content-addressed storage system that scales efficiently to millions of books with automatic deduplication and optimal filesystem performance. The system is simple, robust, and requires no maintenance.

**Related Sessions**:
- Session 21 (2026-01-27): Batch Import Implementation (now uses hash-based storage)
- Future: Migration tool for existing libraries (not needed for this clean install)

---

