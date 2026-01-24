# Session History - December 2025

This document contains all development sessions from December 2025.

## 2025-12-14 - Session 1: Configuration System

**New Crate: ritmo_config**
- Created `ritmo_config` crate for global application configuration
- Manages `~/.config/ritmo/settings.toml` with last_library_path and recent_libraries
- Implements portable mode detection (auto-detects if running from bootstrap/portable_app/)
- Shared between GUI and CLI for consistent behavior
- Integrated with `ritmo_errors` (no custom error types)

**CLI Improvements**
- Refactored `ritmo_cli` from simple demo to full-featured command-line tool
- Added subcommands: init, info, list-libraries, set-library
- Global `--library PATH` option to temporarily override library
- Auto-saves to recent libraries when initializing or using libraries
- Fully integrated with `ritmo_config` for global settings

**GUI Status**
- GUI (`ritmo_gui`) not yet updated to use `ritmo_config`
- Currently uses hardcoded library path (~/RitmoLibrary or ./ritmo_library)
- TODO: Integrate library selection dialog and `ritmo_config` support

**Error Handling**
- Extended `ritmo_errors::RitmoErr` with config-related variants
- Added conversions from toml::de::Error and toml::ser::Error
- All crates now use shared error types consistently

---

## 2025-12-14 - Session 2: Filter System Implementation (COMPLETED)

**Filter System Architecture:**
- Added `list-books` and `list-contents` commands to CLI with comprehensive filter options
- Created filter structs in `ritmo_db_core/src/filters.rs`:
  - `BookFilters`: author, publisher, series, format, year, isbn, search, sort, limit, offset
  - `ContentFilters`: author, content_type, year, search, sort, limit, offset
  - `BookSortField` and `ContentSortField` enums with SQL mapping
- Implemented query builder in `ritmo_db_core/src/query_builder.rs`:
  - `build_books_query()`: Constructs parameterized SQL with JOINs and WHERE clauses
  - `build_contents_query()`: Similar for contents
  - Full test coverage (6 tests passing)
- CLI commands accept all filter options and build queries, but database execution not yet implemented

**Architecture Decision: Filter Location**
- Initially placed filters in `ritmo_core`, but caused cyclic dependency
- Moved to `ritmo_db_core` since filters are tightly coupled to database queries
- This keeps the dependency graph clean: `ritmo_cli` → `ritmo_db_core` (filters) → database

**Filter System Implementation Complete (Session 2):**
✅ Created result structs (`BookResult`, `ContentResult`) in `ritmo_db_core/src/results.rs`:
  - Struct definitions with `sqlx::FromRow` derive for direct query mapping
  - Helper methods for date formatting and short string representation
  - Full test coverage (2/2 tests passing)

✅ Implemented query execution functions in `ritmo_db_core/src/query_builder.rs`:
  - `execute_books_query()`: Executes parameterized book queries
  - `execute_contents_query()`: Executes parameterized content queries
  - Proper parameter binding for SQL injection prevention

✅ Created output formatter in `ritmo_cli/src/formatter.rs`:
  - Three output formats: `table` (default), `json`, `simple`
  - Table format with aligned columns and headers
  - JSON format for programmatic processing
  - Simple format for quick reading
  - Helper functions for string truncation

✅ Integrated query execution in CLI commands:
  - Updated `cmd_list_books()` to execute queries and format results
  - Updated `cmd_list_contents()` to execute queries and format results
  - Added `--output` / `-o` flag for format selection
  - Full error handling with library validation

✅ Testing and Validation:
  - All 8 unit tests passing in `ritmo_db_core`
  - All 2 unit tests passing in `ritmo_cli` formatter
  - End-to-end testing on empty database successful
  - All output formats verified (table, json, simple)
  - Help documentation complete and accurate

**Files Modified/Created:**
- Created: `ritmo_db_core/src/results.rs` (148 lines)
- Created: `ritmo_cli/src/formatter.rs` (223 lines)
- Modified: `ritmo_db_core/src/lib.rs` (exported new types)
- Modified: `ritmo_db_core/src/query_builder.rs` (added execution functions)
- Modified: `ritmo_cli/src/main.rs` (integrated query execution)
- Modified: `ritmo_cli/Cargo.toml` (added serde_json dependency)

**Preset System Implementation Complete (Session 2 - Phase 1):**
✅ Created preset data structures in `ritmo_config/src/presets.rs`:
  - `PresetType`: Enum for Books/Contents
  - `BookFilterPreset` and `ContentFilterPreset`: Filter configurations
  - `NamedPreset<T>`: Generic container with name and description
  - `GlobalPresets`: HashMap-based storage for both preset types
  - Full test coverage (5/5 tests passing)

✅ Integrated presets into AppSettings:
  - Added `presets: GlobalPresets` field to `AppSettings`
  - Presets automatically saved/loaded with settings.toml
  - Exported preset types from `ritmo_config`

✅ Implemented CLI preset commands:
  - `save-preset`: Save filter combinations with name and description
  - `list-presets`: Display all saved presets (supports filtering by type)
  - `delete-preset`: Remove saved presets
  - All commands support both books and contents presets

✅ Added `--preset` flag to list commands:
  - `list-books --preset <name>`: Apply saved preset
  - `list-contents --preset <name>`: Apply saved preset
  - CLI parameters override preset values (explicit > preset > default)
  - Preset not found error handling

✅ Testing and Validation:
  - End-to-end testing successful
  - Preset save/load/delete verified
  - Preset application in list commands verified
  - Parameter override priority confirmed
  - Help documentation updated with --preset flag

**Files Modified/Created:**
- Created: `ritmo_config/src/presets.rs` (280 lines)
- Modified: `ritmo_config/src/app_settings.rs` (added presets field)
- Modified: `ritmo_config/src/lib.rs` (exported preset types)
- Modified: `ritmo_cli/src/main.rs` (added preset commands and --preset flag, ~200 lines added)
- Commands: SavePreset, ListPresets, DeletePreset

**Phase 1 Complete - Global Presets Working**
The preset system is now fully functional for global presets stored in `~/.config/ritmo/settings.toml`.
Users can save filter combinations and reuse them across sessions.

**Next Phase (Future):**
- Library-specific presets in `library/config/filters.toml` (portable!)
- Default preset selection per library
- Preset resolution order (library > global)

**Book Import System Complete (Session 4):**
✅ Created book import service in `ritmo_core/src/service/book_import_service.rs`:
  - `BookImportMetadata`: Struct for manual metadata input
  - `import_book()`: Main function to import books with metadata
  - Helper functions: `get_or_create_format()`, `get_or_create_publisher()`, etc.
  - SHA256 hash calculation for duplicate detection
  - Automatic file format detection from extension
  - File copying to `storage/books/` directory

✅ Implemented CLI `add` command in `ritmo_cli`:
  - Command: `ritmo add <file> --title "..." [options]`
  - Required: `--title` parameter
  - Optional: `--author`, `--publisher`, `--year`, `--isbn`, `--format`, `--series`, `--series-index`, `--notes`
  - Automatic format detection if `--format` not specified
  - Duplicate detection by file hash
  - Integration with existing library system

✅ Testing and Validation:
  - All workspace tests passing (23/23 tests)
  - Successfully imported 3 test books (EPUB, PDF formats)
  - Duplicate detection verified (prevents re-importing same file)
  - Format auto-detection verified (PDF format correctly detected)
  - File storage verified (files copied to storage/books/)

**Files Created/Modified:**
- Created: `ritmo_core/src/service/book_import_service.rs` (~230 lines)
- Modified: `ritmo_core/src/service/mod.rs` (exported new types)
- Modified: `ritmo_cli/src/main.rs` (added `Add` command and `cmd_add()` function, ~80 lines)

**Example Usage:**
```bash
# Import a book with minimal metadata
ritmo add book.epub --title "My Book" --author "John Doe"

# Import with full metadata
ritmo add book.pdf --title "Complete Book" --author "Jane Smith" \
  --publisher "Publisher Name" --year 2024 --isbn "978-1234567890" \
  --series "Series Name" --series-index 1 --notes "First volume"

# Format is detected automatically from file extension
ritmo add file.mobi --title "Mobi Book"  # format = mobi
```

---

## 2025-12-16 - Session 5: Library-Specific Preset System (Phase 2) - COMPLETED

**Library-Specific Presets Implementation:**
✅ Created `LibraryPresets` struct in `ritmo_db_core/src/library_presets.rs`:
  - Manages presets stored in `library/config/filters.toml` (portable with library!)
  - Supports both book and content presets
  - Default preset selection per library
  - Auto-creates example presets: `epub_only`, `pdf_only`, `novels`
  - Full test coverage (7/7 tests passing)

✅ Implemented preset resolution system in `ritmo_config/src/preset_resolver.rs`:
  - `PresetResolver`: resolves presets with priority order (library > global)
  - `LibraryPresetsHolder`: bridge struct to avoid circular dependencies
  - `PresetSource`: enum to identify preset origin (Library/Global)
  - Full test coverage (4/4 tests passing)

✅ Enhanced CLI commands:
  - `save-preset --in-library`: saves preset in library instead of globally
  - `set-default-filter <type> <name>`: sets library default preset
  - `list-presets`: shows library and global presets separately with clear labels
  - `list-books/list-contents`: automatic preset resolution (library > global)
  - `init`: auto-creates example presets in new libraries

✅ Testing and Validation:
  - End-to-end testing with real library
  - Preset resolution priority verified (library overrides global)
  - Portable `filters.toml` file created and tested
  - Default preset functionality confirmed
  - All 23 workspace tests passing

**Files Created:**
- `ritmo_db_core/src/library_presets.rs` (~280 lines)
- `ritmo_config/src/preset_resolver.rs` (~240 lines)

**Files Modified:**
- `ritmo_db_core/src/lib.rs`: exported LibraryPresets and added helper methods
- `ritmo_db_core/Cargo.toml`: added ritmo_config dependency
- `ritmo_config/src/lib.rs`: made presets module public, exported resolver
- `ritmo_cli/src/main.rs`: added --in-library flag, set-default-filter command, updated list-presets

**Portable Workflow Verified:**
```bash
# Initialize library (creates example presets automatically)
ritmo init /media/usb/MyLibrary

# Save custom preset to library (travels with library!)
ritmo --library /media/usb/MyLibrary save-preset books \
  --name my_collection --in-library --format epub

# Set as default for this library
ritmo --library /media/usb/MyLibrary set-default-filter books my_collection

# Preset resolution: library preset takes priority over global
ritmo --library /media/usb/MyLibrary list-books --preset my_collection
```

**Benefits:**
- Presets are fully portable with the library
- Per-library customization without affecting global settings
- Seamless integration with portable mode
- Clear separation between library and global presets

---

## 2025-12-16 - Session 6: Relative Date Filters for Book Acquisition - COMPLETED

**Relative Date Filters Implementation:**
✅ Added three new CLI parameters for relative date filtering:
  - `--last-days N`: Filter books acquired in the last N days
  - `--last-months N`: Filter books acquired in the last N months (approximated to 30 days/month)
  - `--recent-count N`: Get the N most recently acquired books (auto-sorts by date_added DESC)

✅ Implementation details:
  - Created helper functions: `timestamp_days_ago()` and `timestamp_months_ago()`
  - Used chrono's `Duration` for accurate timestamp calculation
  - `--last-days` and `--last-months` conflict with `--acquired-after` to prevent ambiguous queries
  - `--recent-count` automatically overrides sort and limit parameters for convenience
  - All filters work with existing preset system

✅ Testing and Validation:
  - Tested all three relative date filters successfully
  - Verified `--recent-count` properly limits and sorts results
  - All 34 workspace tests passing
  - End-to-end testing with real books confirmed

**Example Usage:**
```bash
# Books acquired in the last week
ritmo list-books --last-days 7

# Books acquired in the last month
ritmo list-books --last-months 1

# 10 most recently acquired books
ritmo list-books --recent-count 10

# Combine with other filters
ritmo list-books --last-days 30 --format epub --author "King"
```

**Files Modified:**
- `ritmo_cli/src/main.rs`: Added CLI parameters, helper functions, and filter logic

**Commit:** 2bf411a - "Add relative date filters for book acquisition"

---

## 2025-12-17 - Session 7: Filter System Refactoring (Phase 1 & 2) - COMPLETED

**Phase 1: Modular Architecture**
✅ Reorganized filter system into isolated modules:
  - Created `ritmo_db_core/src/filters/` directory structure
  - Separated concerns: types, builder, executor into dedicated files
  - Added comprehensive module documentation
  - Maintained 100% backward compatibility via re-exports
  - All 34 tests passing with zero breaking changes

**Phase 2: OR Logic and Validation**
✅ Implemented OR logic for multiple filter values:
  - Changed filter fields from `Option<String>` to `Vec<String>` (authors, publishers, formats, series)
  - Multiple values use OR logic: `(author='King' OR author='Tolkien')`
  - Different filters combined with AND: `author AND format AND year`
  - Helper function `build_or_clause()` for clean SQL generation

✅ Created validation module (`filters/validator.rs`):
  - Validates negative offsets, invalid limits, too many values (max 50)
  - Validates date ranges and empty values
  - Custom `ValidationError` type with Display trait
  - 8 new validation tests

✅ Builder pattern and helpers:
  - Fluent API: `with_author()`, `with_format()`, `with_series()`
  - Backward compatibility: `set_author_opt()`, `set_publisher_opt()`
  - CLI updated to use new builder pattern

**Statistics:**
- Tests: 44 passing (was 34, +10 new tests for OR logic and validation)
- Files created: `filters/{types,builder,executor,validator}.rs`
- Backward compatibility: 100% - all existing code works unchanged
- Commits: `11c1e6f` (Phase 1), `89f881a` (Phase 2)

**Example Usage:**
```rust
// OR logic for authors
let filters = BookFilters::default()
    .with_author("King")
    .with_author("Tolkien")
    .with_format("epub");
// SQL: (author LIKE '%King%' OR author LIKE '%Tolkien%') AND format LIKE '%epub%'

// Validation
validate_book_filters(&filters)?;
execute_books_query(&pool, &filters).await?;
```

---

## 2025-12-18 - Session 8: Complete CRUD System Implementation - COMPLETED

**System CRUD Implementation:**
✅ Implemented full CRUD operations for Books and Contents:
  - **UPDATE**: `update-book` and `update-content` commands with optional field updates
  - **DELETE**: `delete-book` and `delete-content` commands with file management options
  - **CLEANUP**: `cleanup` command to remove orphaned entities

✅ Database Layer (ritmo_db):
  - Added `update()` method to Book model
  - Added `get_by_name()` and `get_or_create_by_name()` to Type model
  - Content model already had update/delete methods

✅ Service Layer (ritmo_core/src/service/):
  - `book_update_service.rs`: Update book metadata with optional fields
  - `content_update_service.rs`: Update content metadata
  - `delete_service.rs`: Delete operations with file management + cleanup utilities
  - All services properly handle relationships (authors, publishers, series)

✅ CLI Commands:
  - `update-book <id>`: Update book metadata (title, author, publisher, year, isbn, format, series, notes, pages)
  - `delete-book <id>`: Delete book with `--delete-file` and `--force` options
  - `update-content <id>`: Update content metadata (title, author, type, year, notes, pages)
  - `delete-content <id>`: Delete content from database
  - `cleanup`: Remove orphaned entities with `--dry-run` option

✅ Features:
  - Optional field updates (only specified fields are modified)
  - Automatic relationship management (get_or_create for related entities)
  - File deletion support with force mode for error handling
  - CASCADE deletion for related records via database constraints
  - Comprehensive cleanup statistics (people, publishers, series, formats, types, tags)

✅ Testing:
  - End-to-end CRUD workflow verified
  - All operations tested successfully
  - 100% compilation success
  - All existing tests passing

**Files Created:**
- `ritmo_cli/src/commands/cleanup.rs` (63 lines)
- `ritmo_core/src/service/book_update_service.rs` (127 lines)
- `ritmo_core/src/service/content_update_service.rs` (107 lines)
- `ritmo_core/src/service/delete_service.rs` (245 lines)

**Files Modified:**
- `ritmo_db/src/models/books.rs`: Added update() method
- `ritmo_db/src/models/types.rs`: Added get_by_name() and get_or_create_by_name()
- `ritmo_cli/src/commands/books.rs`: Added cmd_update_book() and cmd_delete_book()
- `ritmo_cli/src/commands/contents.rs`: Added cmd_update_content() and cmd_delete_content()
- `ritmo_cli/src/commands/mod.rs`: Exported new command functions
- `ritmo_cli/src/main.rs`: Added command enum variants and handlers
- `ritmo_core/src/service/mod.rs`: Exported new services

**Statistics:**
- 11 files changed
- 955 insertions (+), 14 deletions (-)
- 5 new CLI commands
- 100% CRUD coverage for Books and Contents

---

## 2025-12-18 - Session 7: ritmo_ml Activation - Phase 1 (Core ML Infrastructure) - COMPLETED

**Goal**: Activate and complete the core ML infrastructure for entity deduplication (authors, publishers, series).

**Problem**: The `ritmo_ml` crate was disabled and incomplete, with missing trait implementations and no pattern classification system.

**Solution**: Implemented complete ML infrastructure with pattern detection and confidence scoring.

✅ **MLProcessable Trait Implementation**:
  - `PublisherRecord`: added `variants: Vec<String>` field, full trait implementation
  - `SeriesRecord`: added `variants: Vec<String>` field, full trait implementation
  - `PersonRecord`: uncommented and fixed `aliases` field, implemented `add_alias()` and `all_canonical_keys()`
  - `TagRecord`: completed trait implementation with label normalization
  - All 4 entity types now support ML operations (clustering, pattern detection)

✅ **Pattern Classification System** (`pattern_functions.rs`):
  - `default_classify_pattern_type()`: classifies variants into 7 pattern types:
    - **Abbreviation**: "J.R.R. Tolkien" ← "John Ronald Reuel Tolkien"
    - **Prefix**: "Dr. Smith" ← "Smith"
    - **Suffix**: "Smith Jr." ← "Smith"
    - **Compound**: "Stephen King" ← "King, Stephen"
    - **Transliteration**: "Dostoyevsky" ← "Dostoevskij"
    - **Typo**: small edit distance variations
    - **Other**: unclassified patterns
  - `default_confidence_function()`: smart scoring with bonuses/penalties
    - Bonus for abbreviations with matching initials
    - Penalty for large edit distance (>3)
    - Penalty for length difference >50%
  - `are_initials_matching()`: helper for abbreviation verification
  - 6 unit tests covering all pattern types (all passing)

✅ **MLEntityLearner Enhancements**:
  - `create_clusters()`: clustering with Jaro-Winkler similarity (threshold 0.85)
  - `identify_variant_patterns()`: customizable pattern detection with user functions
  - `identify_variant_patterns_with_defaults()`: convenience method using default functions
  - Serializable structures for ML persistence

✅ **Supporting Infrastructure**:
  - `entity_persistence.rs`: save/load ML data to `ml_data` table (already implemented)
  - `feedback.rs`: positive/negative pair management for supervised learning (already implemented)
  - `utils.rs`: MLStringUtils with Unicode NFC normalization (already implemented)

✅ **Workspace Integration**:
  - Enabled `ritmo_ml` in root `Cargo.toml` members
  - Added dependencies: `human_name`, `strsim`, `unicode-normalization`
  - All workspace tests passing (no regressions)

**Statistics**:
- Files modified: 8 files
- Lines added: +401, removed: -41
- Tests: 6 new pattern_functions tests (all passing)
- Total workspace tests: 59 passing

**What's Ready**:
- ✅ Core ML algorithms (clustering, pattern detection)
- ✅ Entity record structures with ML support
- ✅ Database persistence layer
- ✅ Feedback system for supervised learning

**What's Missing (Phase 2)**:
- ❌ End-to-end deduplication workflow
- ❌ Database loading/merging functions
- ❌ CLI commands for deduplication
- ❌ Integration tests

**Files Modified**:
- `ritmo_ml/src/entity_learner.rs`: added pattern detection methods
- `ritmo_ml/src/pattern_functions.rs`: created (179 lines)
- `ritmo_ml/src/people/record.rs`: fixed aliases implementation
- `ritmo_ml/src/publishers/record.rs`: added variants field
- `ritmo_ml/src/series/record.rs`: added variants field
- `Cargo.toml`: enabled ritmo_ml
- `Cargo.lock`: added new dependencies

**Commit**: `1b66179` - "Activate and complete ritmo_ml crate - Phase 1"

**Next Phase**: End-to-end workflow with database integration and CLI commands.

---

## 2025-12-18 - Session 9: RitmoReporter Trait System - COMPLETED

**Problem**: Shared library modules (`ritmo_db_core`, `ritmo_core`, `ritmo_ml`) contained `println!`, `eprintln!`, and `dbg!` statements that would cause unwanted console output when used in GUI applications.

**Solution**: Created a reporter trait abstraction layer to decouple output from business logic.

✅ **RitmoReporter Trait** (`ritmo_errors/src/reporter.rs`):
  - Trait with 3 methods: `status()`, `progress()`, `error()`
  - `SilentReporter`: no-op implementation for libraries and tests
  - Comprehensive documentation with GuiReporter implementation guidelines
  - 8 tests (3 unit tests + 5 doc tests)

✅ **Refactored Core Modules**:
  - `ritmo_db_core` (4 statements removed):
    - `Database::from_pool(&mut impl RitmoReporter)` 
    - `LibraryConfig::create_pool(&mut impl RitmoReporter)`
    - `LibraryConfig::create_database(&mut impl RitmoReporter)`
    - Removed `dbg!()` from `LibraryConfig::save()`
  - `ritmo_core` (3 statements removed):
    - `delete_book(&mut impl RitmoReporter)`
    - `delete_content(&mut impl RitmoReporter)`
  - `ritmo_ml` (1 statement removed):
    - `TagRecord::set_variants()` cleaned up

✅ **CLI Integration**:
  - Implemented `CliReporter` struct in `ritmo_cli/src/main.rs`
  - Updated all CLI commands to use `SilentReporter` for library calls
  - CLI continues to print directly for user-facing output (unchanged behavior)

**Benefits**:
- GUI compatibility: shared modules no longer output to console
- Testability: tests run without console noise
- Flexibility: different frontends can implement custom reporters
- No breaking changes: all existing code works unchanged
- All 59 tests passing

**Files Modified**: 11 files, +299 lines, -28 lines

**Commit**: `42e6759` - "Refactor: replace println!/eprintln! with RitmoReporter trait"

---

## 2025-12-18 - Session 10: ritmo_ml Phase 2 - End-to-end Deduplication Workflow - COMPLETED

**Goal**: Build complete deduplication workflow from database loading to safe merging.

**What Was Implemented**:

✅ **Step 1: Database Loaders** (`db_loaders.rs`, ~190 lines):
  - `load_people_from_db()`: Load and parse all people with MLStringUtils normalization
  - `load_publishers_from_db()`: Load all publishers with normalized names
  - `load_series_from_db()`: Load all series with normalized titles
  - `load_tags_from_db()`: Load all tags with normalized labels
  - Smart parsing for PersonRecord using `PersonRecord::new()`
  - Error handling: skip unparseable records, continue loading rest

✅ **Step 2: Merge Operations** (`merge.rs`, ~410 lines):
  - `merge_people(pool, primary_id, duplicate_ids)`: Merge duplicate authors
    - Updates `x_books_people_roles` junction table
    - Updates `x_contents_people_roles` junction table
    - Deletes duplicate person records
  - `merge_publishers(pool, primary_id, duplicate_ids)`: Merge duplicate publishers
    - Updates `books.publisher_id` foreign key
    - Deletes duplicate publisher records
  - `merge_series(pool, primary_id, duplicate_ids)`: Merge duplicate series
    - Updates `books.series_id` foreign key
    - Deletes duplicate series records
  - `MergeStats`: Return statistics (primary_id, merged_ids, books/contents updated)
  - **Safety Features**:
    - All operations in database transactions (rollback on error)
    - Full validation: check all IDs exist before merge
    - Update all foreign keys and junction tables atomically
    - Detailed error messages for debugging

✅ **Step 3: Deduplication Workflow** (`deduplication.rs`, ~380 lines):
  - `deduplicate_people(pool, config)`: Complete workflow for authors
  - `deduplicate_publishers(pool, config)`: Complete workflow for publishers
  - `deduplicate_series(pool, config)`: Complete workflow for series
  - **DeduplicationConfig**:
    - `min_confidence: 0.90` (default, high confidence for safety)
    - `min_frequency: 3` (minimum pattern frequency)
    - `auto_merge: false` (default, requires manual approval)
    - `dry_run: true` (default, preview mode only)
  - **DeduplicationResult**:
    - `total_entities`: count of entities processed
    - `duplicate_groups`: list of DuplicateGroup with details
    - `merged_groups`: list of MergeStats (if auto_merge=true)
    - `skipped_low_confidence`: count of groups below threshold
  - **DuplicateGroup**:
    - `primary_id`, `primary_name`: entity to keep
    - `duplicate_ids`, `duplicate_names`: entities to merge
    - `confidence`: ML confidence score (0.0-1.0)
  - **Workflow Steps**:
    1. Load all entities from database
    2. Extract canonical keys for ML comparison
    3. Run ML clustering (Jaro-Winkler similarity)
    4. Convert clusters to duplicate groups with confidence
    5. Optionally auto-merge high-confidence duplicates
    6. Return detailed results and statistics

✅ **Utils Enhancement**:
  - Added `MLStringUtils::default()` for convenient initialization
  - Enables use without pre-configured name variants

**Statistics**:
- Files created: 3 (db_loaders.rs, merge.rs, deduplication.rs)
- Files modified: 2 (lib.rs, utils.rs)
- Lines added: ~1,000 lines of production code
- Tests: 2 unit tests + comprehensive docs (ignored, require real DB)
- Compilation: ✅ Success (1 minor warning)

**Safety & Design Principles**:
- **Dry-run by default**: No accidental data loss
- **High confidence threshold**: 0.90 for auto-merge
- **Transactional merges**: Atomic operations with rollback
- **Error resilience**: Skip failed merges, continue with rest
- **Detailed logging**: Track all operations and failures
- **Configurable**: Adjust thresholds for different use cases

**What's Complete**:
- ✅ ML clustering and duplicate detection
- ✅ Safe database merging with transactions
- ✅ End-to-end workflow (load → detect → merge)
- ✅ Configurable behavior (thresholds, dry-run, auto-merge)
- ✅ Comprehensive error handling and statistics

**What's Missing (Next Session)**:
- ❌ CLI commands to expose functionality (`ritmo deduplicate-people`, etc.)
- ❌ Integration tests with real database
- ❌ Interactive mode for manual duplicate review

**Files Modified**:
- `ritmo_ml/src/db_loaders.rs`: created (~190 lines)
- `ritmo_ml/src/merge.rs`: created (~410 lines)
- `ritmo_ml/src/deduplication.rs`: created (~380 lines)
- `ritmo_ml/src/utils.rs`: added `default()` method
- `ritmo_ml/src/lib.rs`: exported new modules

**Commits**: 
- `74dd548` - "ritmo_ml Phase 2: Database loaders and merge operations"
- `bd8073f` - "ritmo_ml Phase 2 Step 3: Deduplication workflow"

**Next Phase**: CLI commands and integration tests to make ML deduplication user-accessible.
