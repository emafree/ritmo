# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ritmo** is a Rust-based library management system inspired by Calibre, but focused solely on cataloging books and their metadata (not editing, reading, or converting). The primary goal is to catalog books, their contents, and contributors (authors, translators, illustrators, editors).

The project uses SQLite for database storage (no external server required) and is organized as a Rust workspace with multiple specialized crates.

## Quick Start

```bash
# Build entire workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Initialize a new library
cargo run -p ritmo_cli -- libraries init

# Import a book
cargo run -p ritmo_cli -- books add book.epub --title "Book Title" --author "Author Name"

# List books
cargo run -p ritmo_cli -- books list
```

For complete build and run commands, see [Development Guide](docs/development.md).

## Documentation Structure

Detailed documentation is organized in the `docs/` directory:

- **[Architecture](docs/architecture.md)** - Workspace crates, database schema, directory structure, key patterns
- **[Command Layer](docs/command-layer.md)** - Command pattern, design principles, usage examples
- **[Development Guide](docs/development.md)** - Build, test, and run commands
- **[Filter System](docs/filters.md)** - Comprehensive filter and preset system documentation
- **[ML System](docs/ml-system.md)** - Entity deduplication with machine learning
- **[Book Metadata Format](docs/book_metadata_format.md)** - JSON format specification for Levels 2 & 3
- **[Session History](docs/sessions/)** - Chronological changelog of all development sessions

## Command Layer Architecture

**ritmo** uses a **command pattern** to separate presentation logic (CLI/GUI) from business logic. The `ritmo_commands` crate provides stateless, testable commands that can be shared across frontends.

### Architecture Layers

```
Presentation (CLI/GUI) → Commands (ritmo_commands) → Services (ritmo_core) → Database
```

### Benefits

- **Code Reuse**: Same commands for CLI, GUI, and future API
- **Testability**: Commands testable without UI dependencies
- **Type Safety**: Structured inputs/outputs with Serde
- **Separation**: Clean boundaries between layers
- **Validation**: Centralized input validation

### Available Commands (10 total)

**Books (4)**:
- `AddBookCommand` - Import book with manual metadata
- `ListBooksCommand` - Query books with filters
- `UpdateBookCommand` - Update book metadata
- `DeleteBookCommand` - Delete book from DB/storage

**Contents (6)**:
- `AddContentCommand` - Create new content
- `ListContentsCommand` - Query contents with filters
- `UpdateContentCommand` - Update content metadata
- `DeleteContentCommand` - Delete content record
- `LinkContentCommand` - Associate content to book
- `UnlinkContentCommand` - Remove content-book link

### Quick Example

```rust
use ritmo_commands::{Command, books::{AddBookCommand, AddBookInput}};

// Prepare input
let input = AddBookInput {
    file_path: PathBuf::from("book.epub"),
    title: "My Book".to_string(),
    publisher: Some("Publisher".to_string()),
    ..Default::default()
};

// Execute command
let command = AddBookCommand;
let result = command.execute(&config, &pool, input).await?;

// Use structured result
println!("Book added! ID: {}", result.book_id);
```

See [Command Layer Documentation](docs/command-layer.md) for complete guide, examples, and how to add new commands.

## Key Features

### Library Management
- Multi-library support with global configuration (`~/.config/ritmo/settings.toml`)
- Portable mode: auto-detect when running from `bootstrap/portable_app/`
- Library initialization with directory structure and template database

### Book Management (CRUD Complete)
- **Create**: Import books with manual metadata, SHA256 hash for duplicate detection
- **Read**: List and filter books with comprehensive query system
- **Update**: Modify book metadata with optional field updates
- **Delete**: Remove books with CASCADE deletion of relationships, optional physical file deletion, and separate cleanup command for orphaned entities (people, publishers, series, tags, formats)

### Book Import Levels
The book import system is designed with progressive automation levels:

**Level 1 - Manual Import (IMPLEMENTED)**
- Single book import with manual metadata entry
- Title is required, all other metadata optional
- Format auto-detected from file extension
- SHA256 hash for duplicate detection
- Command: `books add book.epub --title "Title" --author "Author"`

**Level 2 - Batch Import via Pipe (IMPLEMENTED)**
- Import multiple books from JSON metadata file or stdin
- Uses same JSON format as Level 3 ebook_parser output
- Enables review/edit workflow: extract metadata → review → batch import
- Full import: books + contents + relationships (people, languages, tags, series)
- Validation: 16 rules with detailed error messages
- Duplicate detection via SHA256 hash
- Error handling: stop-on-error (default) or continue-on-error
- Dry-run mode for validation without importing
- Examples:
  - `ritmo books add-batch --input books_metadata.json`
  - `cat books_metadata.json | ritmo books add-batch`
  - `ritmo books add-batch --input metadata.json --dry-run` (validation only)
  - `ritmo books add-batch --input metadata.json --continue-on-error`
  - Future workflow with Level 3:
    - `ritmo extract-metadata ~/books/*.epub > metadata.json`
    - `# Review/edit metadata.json`
    - `ritmo books add-batch --input metadata.json`

**Level 3 - Automatic Metadata Extraction (PLANNED)**
- Parse EPUB metadata from content.opf automatically
- Extract: title, authors, publisher, publication date, ISBN, language
- Output JSON format compatible with Level 2 batch import
- Goal: 95% automation for ~12,000+ books
- Confidence scores for each extracted field
- Two modes:
  - Extract-only: output JSON for review (use with Level 2)
  - Direct import: extract and import in one step
- Integration with ebook_parser crate

**JSON Metadata Format (Levels 2 & 3)**

The format uses a Book/Contents structure reflecting ritmo's database architecture:
- **Book**: Physical book file with edition metadata (publisher, ISBN, series, format)
- **Contents**: Literary works contained in the book (authors, translators, languages, type)
- **People**: Contributors (book-level: editors, preface; content-level: authors, translators)

```json
[
  {
    "file_path": "/path/to/book.epub",
    "book": {
      "title": "Complete Works Edition",
      "original_title": "Original Edition Title",
      "people": [
        {"name": "Editor Name", "role": "role.editor"},
        {"name": "Preface Author", "role": "role.preface"}
      ],
      "publisher": "Publisher Name",
      "year": 2024,
      "isbn": "978-1234567890",
      "format": "epub",
      "series": "Series Name",
      "series_index": 1,
      "pages": 350,
      "notes": "Collected edition",
      "tags": ["fiction", "collection"]
    },
    "contents": [
      {
        "title": "Novel Title",
        "original_title": "Original Title",
        "people": [
          {"name": "Author Name", "role": "role.author"},
          {"name": "Translator Name", "role": "role.translator"}
        ],
        "type": "type.novel",
        "year": 2020,
        "languages": [
          {"code": "en", "role": "language_role.original"},
          {"code": "it", "role": "language_role.actual"}
        ]
      }
    ],
    "confidence": {
      "book.title": 0.95,
      "book.publisher": 0.85,
      "book.series": 0.85,
      "contents[0].title": 0.95,
      "contents[0].people": 0.90
    }
  }
]
```

See [Book Metadata Format](docs/book_metadata_format.md) for complete specification.

### Filter System
- Multiple filter types: author, publisher, series, format, year, ISBN, dates
- OR logic for multiple values within same filter type
- Preset system: global and library-specific filter presets
- Three output formats: table, JSON, simple
- Relative date filters: `--last-days`, `--last-months`, `--recent-count`

See [Filter System Documentation](docs/filters.md) for complete details.

### ML Deduplication (ritmo_ml)
- Pattern classification system (7 pattern types)
- Jaro-Winkler similarity clustering
- Safe database merging with transactions
- Configurable confidence thresholds
- Dry-run mode for preview

See [ML System Documentation](docs/ml-system.md) for complete details.

### RitmoReporter Trait
- Abstraction layer for output in shared modules
- Prevents unwanted console output in GUI applications
- `SilentReporter` for libraries, custom reporters for frontends

## Essential Commands

### Library Operations
```bash
cargo run -p ritmo_cli -- libraries init [PATH]  # Initialize library
cargo run -p ritmo_cli -- libraries info         # Show library info
cargo run -p ritmo_cli -- libraries list         # Show recent libraries
cargo run -p ritmo_cli -- libraries set PATH     # Set current library
```

### Book Operations
```bash
# Add single book (Level 1 - Manual)
cargo run -p ritmo_cli -- books add book.epub --title "Title" --author "Author"

# Add books in batch (Level 2 - Batch Import)
cargo run -p ritmo_cli -- books add-batch --input books_metadata.json
cargo run -p ritmo_cli -- books add-batch --input books_metadata.json --dry-run
cargo run -p ritmo_cli -- books add-batch --input books_metadata.json --continue-on-error
cat books_metadata.json | cargo run -p ritmo_cli -- books add-batch

# List books with filters
cargo run -p ritmo_cli -- books list --author "King" --format epub

# Update single book by ID
cargo run -p ritmo_cli -- books update --id 1 --set-title "New Title"

# Update multiple books by filters (bulk operation with confirmation)
cargo run -p ritmo_cli -- books update --filter-author "King" --set-publisher "New Pub"

# Delete book by ID (database only)
cargo run -p ritmo_cli -- books delete --id 1

# Delete book with physical file
cargo run -p ritmo_cli -- books delete --id 1 --delete-file

# Delete multiple books by filters (bulk operation with confirmation)
cargo run -p ritmo_cli -- books delete --filter-format epub --filter-year 2020

# Dry-run mode: preview without executing
cargo run -p ritmo_cli -- books delete --filter-publisher "Old Pub" --dry-run

# Skip confirmation with --yes flag
cargo run -p ritmo_cli -- books update --filter-author "King" --set-publisher "New" --yes

# Cleanup orphaned entities (people, publishers, series, tags, formats)
cargo run -p ritmo_cli -- books cleanup
```

### Content Operations
```bash
# Create new content
cargo run -p ritmo_cli -- contents add --title "Story Title" --author "Author Name"
cargo run -p ritmo_cli -- contents add --title "Novel" --content-type "Romanzo" --year 2024

# Create content and associate to book
cargo run -p ritmo_cli -- contents add --title "Novel" --author "Author" --book-id 1

# Update single content by ID
cargo run -p ritmo_cli -- contents update --id 1 --set-title "New Title" --set-year 2024

# Update multiple contents by filters (bulk operation with confirmation)
cargo run -p ritmo_cli -- contents update --filter-author "King" --set-content-type "Romanzo"

# Delete single content by ID
cargo run -p ritmo_cli -- contents delete --id 1

# Delete multiple contents by filters (bulk operation with confirmation)
cargo run -p ritmo_cli -- contents delete --filter-content-type "Racconto"

# Associate/unassociate content and book
cargo run -p ritmo_cli -- contents link --content-id 1 --book-id 1
cargo run -p ritmo_cli -- contents unlink --content-id 1 --book-id 1

# List contents with filters
cargo run -p ritmo_cli -- contents list --author "King" --content-type "Romanzo"
```

### ML Deduplication Operations
```bash
# Find duplicate people (authors, translators, etc.) - dry-run by default
cargo run -p ritmo_cli -- deduplicate people --dry-run

# Merge duplicate people with custom threshold
cargo run -p ritmo_cli -- deduplicate people --threshold 0.90 --auto-merge

# Find duplicate publishers
cargo run -p ritmo_cli -- deduplicate publishers --dry-run

# Find duplicate series
cargo run -p ritmo_cli -- deduplicate series --dry-run

# Find duplicate tags
cargo run -p ritmo_cli -- deduplicate tags --dry-run

# Run deduplication for all entity types (people, publishers, series, tags, roles)
cargo run -p ritmo_cli -- deduplicate all --threshold 0.85 --dry-run
```

### Metadata Sync Operations
```bash
# Check how many books need metadata sync
cargo run -p ritmo_cli -- sync metadata --status

# Preview what would be synced (dry-run)
cargo run -p ritmo_cli -- sync metadata --dry-run

# Actually sync EPUB files with database metadata
cargo run -p ritmo_cli -- sync metadata
```

### Language Preference Operations
```bash
# Set language preference (en or it)
cargo run -p ritmo_cli -- language set en

# Get current language settings
cargo run -p ritmo_cli -- language get
```

### Preset Operations
```bash
# Save a book filter preset
cargo run -p ritmo_cli -- presets save book "my_preset" --author "King" --format epub

# List all saved presets
cargo run -p ritmo_cli -- presets list

# Delete a preset
cargo run -p ritmo_cli -- presets delete "my_preset"

# Set default preset for a library
cargo run -p ritmo_cli -- presets set-default "my_preset" book
```

### Entity Management Operations
```bash
# List all tags (with optional output format)
cargo run -p ritmo_cli -- tags list
cargo run -p ritmo_cli -- tags list --output json
cargo run -p ritmo_cli -- tags list --output simple

# List all publishers
cargo run -p ritmo_cli -- publishers list
cargo run -p ritmo_cli -- publishers list --output json

# List all series
cargo run -p ritmo_cli -- series list
cargo run -p ritmo_cli -- series list --output json

# List all people (authors, translators, etc.)
cargo run -p ritmo_cli -- people list
cargo run -p ritmo_cli -- people list --output json
```

For complete command reference, see [Development Guide](docs/development.md).

## Rust Version

Required: **stable** (currently 1.91+) as specified in `rust-toolchain.toml`
- Edition 2024 features available
- Supports Slint GUI framework

## Recent Changes

### 2026-02-10 - Session 31: Command Layer Architecture - COMPLETED
Implemented complete command layer pattern to separate business logic from presentation layer, enabling code sharing between CLI and GUI.

**Phase 1: Command Infrastructure + Books Add/List**
- **New Crate**: `ritmo_commands` with Command trait and error types
- **Command Trait**: Generic trait with typed Input/Output and validation
- **Structured Types**: AddBookResult, ListBooksResult, BookSummary with Serde
- **First Commands**: AddBookCommand, ListBooksCommand (proof-of-concept)

**Phase 2: CLI Migration to Commands**
- **Migrated Handlers**: handle_books_add, handle_books_list
- **Formatter Extension**: Added format_book_summaries for BookSummary type
- **Architecture**: CLI → Commands → Services (clean separation)

**Phase 3: Books Update/Delete Commands**
- **New Commands**: UpdateBookCommand, DeleteBookCommand
- **Output Types**: UpdateBookResult, DeleteBookResult
- **Pattern**: CLI handles bulk iteration, commands handle single operations

**Phase 4: Complete Contents Commands**
- **All 6 Commands**: Add, List, Update, Delete, Link, Unlink
- **Output Types**: 6 new result types for contents operations
- **Handler Migration**: All contents handlers migrated to commands

**Final Status**
- ✅ **10 Commands**: 4 books + 6 contents (complete CRUD)
- ✅ **11 Unit Tests**: All passing
- ✅ **Documentation**: Comprehensive command-layer.md with examples
- ✅ **Architecture**: 3-layer separation (Presentation → Commands → Services)

**Benefits**
- Code reuse between CLI and GUI
- Commands testable without UI
- Type-safe structured outputs
- Centralized validation
- Future-ready for API integration

**Files**: 17 new command files, ~1200 lines of command layer code
See [Command Layer Documentation](docs/command-layer.md) for complete guide.

### 2026-02-10 - Session 30: GUI Phase 2 + Entity CRUD Commands - COMPLETED
**Part 1: GUI Nested Data Implementation (Phase 2)**
Implemented complete nested queries for GUI with books+contents+people relationships.
- **New Module**: `ritmo_db/src/gui_queries.rs` (~500 lines) with optimized 3-step queries
- **Functions**: `get_books_with_nested_data()`, `get_contents_with_nested_data()`
- **i18n Helpers**: `translate_format_key()`, `translate_type_key()`, `translate_role_key()`
- **GUI Integration**: Updated `ritmo_gui/src/main.rs` to populate all nested arrays
- **Performance**: Limit 100 records, 3 queries max (books → contents → people)
- **Data Structures**: BookWithDetails, ContentWithDetails, PersonWithRole, BookBasicInfo
- **Result**: GUI now displays complete relational data (books with contents and people)

**Part 2: CLI Entity Management Commands**
Added list commands for direct database entity inspection.
- **New Commands**: `tags list`, `publishers list`, `series list`, `people list`
- **Output Formats**: table (default), json, simple (for scripting)
- **Handlers**: 4 new handler modules (tags.rs, publishers.rs, series.rs, people.rs)
- **Implementation**: Direct SQL queries for optimal performance
- **CLI Structure**: Added 4 new command groups (Tags, Publishers, Series, People)
- **Testing**: Verified with empty and populated databases
- **Total Commands**: 27 commands (23 from Session 29 + 4 entity list commands)

### 2026-02-09 - Session 29: CLI Refactoring + Database Optimizations - COMPLETED
**Part 1: CLI Refactoring v2.0.0**
Complete restructuring from flat verb-noun commands to nested noun-verb structure with filter-based bulk operations.
- **New Structure**: 23 commands across 7 groups (libraries, books, contents, presets, deduplicate, sync, language)
- **Syntax Change**: `ritmo list-books` → `ritmo books list`, `ritmo delete-book 1` → `ritmo books delete --id 1`
- **Bulk Operations**: Update/delete by ID or filters with interactive confirmation
- **New Features**: `--dry-run` preview, `--yes` skip confirmation, prefixed arguments (`--filter-*`, `--set-*`)
- **Architecture**: Created filter_args.rs (650 lines), confirmation.rs (350 lines), handlers/ directory
- **Performance**: main.rs reduced 40% (1646 → 1000 lines)
- **Files**: 15 files modified, +1888/-1838 lines

**Part 2: Database Optimizations**
Comprehensive performance and data integrity improvements targeting duplicate prevention and bulk operations.
- **Critical**: UNIQUE indexes for file_hash, publishers, series, tags (prevents duplicates)
- **High Priority**: Covering indexes for bulk filters (3-5x faster list/update/delete operations)
- **Maintenance**: Auto-cleanup triggers for audit logs and cache (prevents database bloat)
- **Bug Fixes**: Updated all 10 views to use i18n keys (formats.key, types.key, roles.key)
- **Performance Impact**: Duplicate detection 1000x+ faster, bulk operations 10-50x faster
- **Database Stats**: 59 indexes (+18%), 13 triggers (+2), 10 views (all updated)
- **Files**: optimizations.sql (461 lines), schema.sql, template.db

### 2026-02-09 - Session 28: ML Deduplication Critical Bugfixes - COMPLETED
Fixed three critical bugs in ML deduplication system that prevented correct duplicate detection.
- **Bugs Fixed**:
  - Hardcoded threshold (0.85) ignored user's `--threshold` parameter
  - HashMap collision lost entities with identical canonical keys (e.g., "J.K. Rowling" and "J. K. Rowling")
  - Duplicate IDs in output due to unrepeated cluster members
- **Impact**: System was missing 2 out of 3 duplicate groups in testing
- **Testing**: Comprehensive test with 7 contents and 3 duplicate groups, all 21 unit tests pass
- **Files Modified**: `ritmo_ml/src/entity_learner.rs`, `ritmo_ml/src/deduplication.rs`
- **New Test**: `ritmo_ml/tests/test_name_parsing.rs` for name parsing verification
- **Result**: All deduplication commands now work correctly with configurable thresholds

### 2026-01-28 - Session 27: Code Cleanup and Test Fixes - COMPLETED
Fixed compiler warnings and test race conditions for clean builds and reliable test execution.
- **Issues Resolved**:
  - Eliminated 2 compiler warnings (unused function, unused struct)
  - Fixed race conditions in 32 i18n tests modifying global locale state
  - Fixed missing table in ritmo_ml test schema
  - Fixed doctest compilation error
- **Implementation**:
  - Added `#[cfg(test)]` to `is_valid_library()` in ritmo_config
  - Removed unused `CliReporter` struct from ritmo_cli
  - Added `serial_test = "3.2"` dependency to ritmo_db
  - Marked 33 i18n tests with `#[serial]` to prevent parallel execution interference
  - Added `x_books_contents` table to ritmo_ml test helpers
  - Fixed doctest block from ` ``` to ` ```text in delete_service
- **Key Learning**: Tests with global state side effects (i18n locale) require serialization via `serial_test` crate
- **Testing**: Full workspace build with zero warnings, all 33+ tests passing consistently
- **Impact**: CI/CD ready codebase with reliable test suite

### 2026-01-28 - Session 26: Metadata Sync Tracking System - COMPLETED
Implemented complete metadata sync tracking system to keep EPUB files in sync with database after entity deduplication.
- **Feature**: Track books requiring metadata sync after entity merges (authors, publishers, series, tags, roles)
- **Architecture**: DB → EPUB sync direction (database is source of truth)
- **Implementation**:
  - Added `pending_metadata_sync` table with CASCADE deletion, reasons tracking, and index
  - Created `ritmo_db/src/models/pending_sync.rs` with helper functions (mark, get, count, clear)
  - Modified `ritmo_ml/src/merge.rs` - Added `affected_book_ids` to MergeStats, all merge functions now capture affected books
  - Updated all deduplicate commands - Automatically mark affected books after successful merge
  - Created `ritmo_core/src/service/metadata_sync_service.rs` (~350 lines) - Complete sync service
  - Created `ritmo_cli/src/commands/sync.rs` (~135 lines) - CLI command with 3 modes
- **CLI Command**: `ritmo sync-metadata` with `--status`, `--dry-run`, and default (sync) modes
- **Sync Workflow**:
  1. Read all metadata from DB (book + contents + relations)
  2. Build OPFMetadata from DB data
  3. Modify EPUB using existing OPF modification system
  4. **Recalculate SHA256 hash**
  5. **Move file to new hash-based path**
  6. Update DB with new file_hash and file_link
  7. Clear sync mark
- **Key Features**:
  - Automatic marking during deduplication (5 entity types)
  - Hash recalculation ensures content-addressed storage integrity
  - Original OPF backup in `originals_opf/` preserved unchanged
  - Graceful error handling - failed syncs don't clear marks (retry possible)
  - Progress reporting with detailed output
- **Testing**: Full workspace build successful
- **Documentation**: Complete session history in docs/sessions/2026-01-sessions.md

### 2026-01-28 - Session 25: EPUB OPF Metadata Modification - COMPLETED
Implemented automatic modification of EPUB OPF metadata with user-provided data during import.
- **Feature**: During import, EPUB metadata is updated with user-provided data (title, people, publisher, year, ISBN, tags, series)
- **Workflow**: Extract original OPF (backup) → Modify OPF XML → Rebuild EPUB → Save to storage
- **Implementation**:
  - Created `ritmo_core/src/epub_opf_modifier.rs` (~500 lines) with OPF metadata structures and modification logic
  - `build_opf_metadata()` - Aggregates metadata from BookImportMetadata + ALL ContentInputs (Level 2 batch import)
  - `modify_opf_xml()` - Updates Dublin Core metadata (dc:title, dc:creator, dc:publisher, dc:date, dc:subject, dc:language)
  - `modify_epub_metadata()` - ZIP read/write operations to rebuild EPUB with modified OPF
  - Role mapping: "role.author" → "aut", "role.translator" → "trl", etc. (MARC relator codes)
- **Integration**:
  - Modified `book_import_service.rs` - Renamed `import_book()` → `import_book_with_contents(contents: &[ContentInput])`
  - Wrapper `import_book()` maintained for Level 1 backward compatibility
  - Modified `batch_import_service.rs` - Uses `import_book_with_contents()` to pass content metadata for aggregation
- **Metadata Handling**:
  - Level 1 (manual): Only book-level metadata
  - Level 2 (batch): Aggregates ALL people and languages from ALL contents into EPUB OPF
  - None values: Preserves original OPF elements (doesn't remove existing metadata)
- **Error Handling**: Graceful degradation - OPF modification failure falls back to original EPUB copy
- **Testing**: Verified with Level 1 import - EPUB correctly modified with user metadata, original OPF preserved as backup
- **Dependencies**: Added `quick-xml = "0.36"` to `ritmo_core/Cargo.toml`
- **Documentation**: Complete plan in `/home/ema/.claude/plans/wobbly-floating-raven.md`

### 2026-01-28 - Session 24: OPF Metadata Preservation - COMPLETED
Implemented automatic extraction and storage of original EPUB OPF metadata files during import.
- **Feature**: Extract OPF (Open Packaging Format) XML from EPUB files during import
- **Storage**: `storage/originals_opf/{hash[0:2]}/{hash[2:4]}/{hash[4:]}.opf.xml` (same hierarchy as books)
- **Implementation**:
  - Created `ritmo_core/src/epub_utils.rs` with `extract_opf()` function
  - EPUB opened as ZIP, reads `META-INF/container.xml` to find OPF path
  - Extracts OPF XML and saves with hash-based filename
  - Integrated into `book_import_service.rs` (step 9 after file copy)
- **Error Handling**: Graceful degradation - OPF extraction failure doesn't block import
- **Use Cases**: Metadata analysis, Level 3 auto-extraction, validation, ML training
- **Testing**: Verified extraction, storage, and XML validity with real EPUB
- **Dependencies**: Added `zip = "2.2"` to `ritmo_core/Cargo.toml`
- **Documentation**: Comprehensive "OPF Metadata Preservation" section in architecture.md

### 2026-01-27 - Session 23: Hash-Based Storage System Implementation - COMPLETED
Implemented content-addressed hash-based file storage system for optimal performance and deduplication.
- **Problem**: Previous system used human-readable filenames (flat directory, collision-prone, poor scalability)
- **Solution**: SHA256 hash-based hierarchical storage: `books/{hash[0:2]}/{hash[2:4]}/{hash[4:]}.{ext}`
- **Implementation**:
  - Modified `book_import_service.rs` to generate hash-based paths from file content hash
  - Removed unused `storage_service.rs` and `Book::set_book_persistence()` (metadata-based hashing)
  - Both single and batch import use new system automatically
- **Benefits**:
  - 65,536 subdirectories (256×256) for optimal distribution
  - Content-addressed: same file = same path (automatic deduplication)
  - O(1) lookup performance with known hash
  - Scalable to millions of books
- **Testing**: Verified import, list, delete, and duplicate detection
- **Database Cleared**: Removed 23 existing books (old naming system), fresh start
- **Documentation**: Comprehensive "File Storage System" section in architecture.md

### 2026-01-27 - Session 22: Filter System Schema Migration Bugfix - COMPLETED
Fixed SQL errors in list-books and list-contents commands after Session 17 i18n schema changes.
- **Problem**: Commands failed with "no such column: formats.name" and "types.name" errors
- **Root Cause**: Filter system queries not updated after Session 17 changed `formats.name` → `formats.key` and `types.name` → `types.key`
- **Files Modified**: 3 files (builder.rs, types.rs, formatter.rs)
- **Changes**: Updated 14 references from `format_name/type_name` to `format_key/type_key`
- **Testing**: All commands now work correctly with table, JSON, and simple output formats
- **Related**: Session 17 (i18n Phase 2 - Type and Format Models)

### 2026-01-27 - Session 21: Book Import Level 2 - Batch Import Implementation - COMPLETED
Implemented complete batch import system for importing multiple books from JSON files.
- **CLI Command**: `add-batch` with support for file input (`--input`) and stdin
- **DTO Structures**: Complete JSON deserialization (ImportObject, BookInput, ContentInput, PersonInput, LanguageInput)
- **Batch Import Service**: `batch_import_service.rs` with validation, error handling, and summary reporting
- **Features**:
  - Dry-run mode for validation without importing
  - Continue-on-error vs stop-on-first-error modes
  - Duplicate detection via SHA256 hash
  - Progress reporting and detailed summary
  - Full book+contents+relationships import (people, languages, tags, series)
- **Validation**: 16 validation rules for import objects, books, and contents
- **Testing**: Comprehensive testing with single/multi-book imports, contents, duplicates, stdin input
- **Format**: Uses same JSON format as Level 3 ebook_parser output (see docs/book_metadata_format.md)

### 2026-01-27 - Session 20: Language Preference Management (i18n Phase 5) - COMPLETED
Implemented persistent language preference management with two new CLI commands.
- **Commands**: `set-language` (save preference), `get-language` (show current settings)
- **Priority**: RITMO_LANG env → saved preference → LANG env → default ("en")
- **Implementation**: Enhanced i18n_utils with preference-aware functions, added language commands
- **Translation Keys**: 6 new keys for language management messages
- **Testing**: Full workflow tested (set, get, env override, validation)
- **Total Coverage**: 158 translation keys (DB models + errors + CLI + language management)

### 2026-01-27 - Session 19: I18n Phase 4 - CLI Runtime Messages - COMPLETED
Implemented i18n for CLI runtime messages (success, info, warnings), allowing CLI to display messages in English or Italian based on RITMO_LANG environment variable.
- **Scope**: Runtime messages only (not help text, following standard CLI conventions)
- **Translation Keys**: Added ~40 CLI message keys for 4 core commands (init, info, list-libraries, set-library)
- **Implementation**: Added rust-i18n to ritmo_cli, initialized i18n system in main.rs, converted println! to t!() macro
- **Testing**: All 4 commands tested with both English and Italian
- **Total Coverage**: 152 translation keys (DB models + errors + CLI core commands)

### 2026-01-26 - Session 18: I18n Phase 3 - Error Messages - COMPLETED
Implemented full i18n support for all error messages in ritmo_errors crate through new LocalizableError trait.
- **Translation Keys**: Added 48 error translation keys organized by category (database, file, import/export, config, ML, validation, search, record, generic)
- **LocalizableError Trait**: Generic trait with localize() method for error translation
- **Implementation**: All RitmoErr variants now implement LocalizableError for consistent error messages
- **Testing**: 48 tests verify error translation in both English and Italian

### 2026-01-26 - Session 17: I18n Phase 2 - Type and Format Models - COMPLETED
Converted Type and Format models to use canonical i18n keys instead of translated strings.
- **Schema Changes**: Changed types.name → types.key, formats.name → formats.key
- **Models**: Implemented I18nDisplayable trait for both Type and Format models
- **New Methods**: get_by_key(), get_or_create_by_key() for both models
- **Services Updated**: 4 service files updated to use new key-based methods
- **Deprecated**: Old name-based methods kept for backward compatibility
- **Total Coverage**: 64 translation keys (roles, language_role, types, formats)

### 2026-01-26 - Session 16: I18nDisplayable Trait Implementation - COMPLETED
Created I18nDisplayable trait to eliminate duplicate translation code across models.
- **Trait**: Generic trait with i18n_key(), i18n_namespace(), translate() methods
- **Implementation**: Applied to Role and RunningLanguages models
- **Benefits**: Reduced code duplication, improved maintainability, enabled generic functions
- **Code Reduction**: 10+ lines per model → 3 lines of trait implementation

### 2026-01-26 - Session 15: i18n Infrastructure Implementation (Phase 1) - COMPLETED
Implemented complete i18n infrastructure with rust-i18n framework, translation files, and locale detection.
- **Framework**: Added rust-i18n v3 with YAML translation files (locales/en.yml, locales/it.yml)
- **Initial Coverage**: ~54 translation keys (db.*, cli.*, error.*, gui.*, validation.*)
- **Utilities**: Created i18n_utils module with detect_locale(), set_locale(), get_locale(), init_i18n()
- **Locale Detection**: Priority order - RITMO_LANG env var → LANG env var → "en" default
- **Models Updated**: Role::display_name() and RunningLanguages::display_role() now use t!() macro
- **Testing**: 7 integration tests verify translations work in both English and Italian
- **Documentation**: Complete developer guide (docs/i18n.md) and translator guide (locales/README.md)
- Foundation ready for Phase 2-5 (progressive translation of ~500 remaining strings)

### 2026-01-26 - Session 14: Roles & Language Roles i18n Integration - COMPLETED
Refactored roles and language_role systems to use canonical i18n keys instead of translated strings.
- **Roles**: Changed `roles` table schema `name` → `key` (e.g., "role.author")
  - Updated Role model with `display_name()`, `get_all()`, `get_by_key()`, `get_or_create_by_key()`
  - Deprecated `get_by_name()` and `get_or_create_by_name()` for backward compatibility
  - Updated 4 services in ritmo_core and ritmo_ml integration
- **Language Roles**: Changed `running_languages` CHECK constraint to use i18n keys
  - Values: "language_role.original", "language_role.source", "language_role.actual"
  - Added `language_role` constants module in languages.rs
  - Added `display_role()` method to RunningLanguages model
- Updated schema.sql and regenerated template.db
- All tests passing, full workspace build successful
- Foundation ready for future i18n implementation

### 2026-01-26 - Session 13: Complete CRUD for Contents - COMPLETED
Implemented full CRUD operations for Contents with 3 new CLI commands.
- `add-content` - Create new contents with metadata (title, author, type, year, etc.)
- `link-content` - Associate existing content to a book
- `unlink-content` - Remove content-book association
- New service: `content_create_service.rs` with validation and entity management
- Contents can be created standalone or directly associated to books
- Full test coverage and documentation updates

### 2026-01-25 - Session 12: ML CLI Integration - COMPLETED
Integrated ritmo_ml deduplication system into CLI with 5 new commands.
- `deduplicate-people` - Find and merge duplicate people (authors, translators, etc.) using ML
- `deduplicate-publishers` - Find and merge duplicate publishers
- `deduplicate-series` - Find and merge duplicate series
- `deduplicate-tags` - Find and merge duplicate tags
- `deduplicate-all` - Run deduplication for all entity types
- Configurable threshold, auto-merge, and dry-run modes
- User-friendly output with confidence scores and merge statistics

### 2026-01-25 - Session 11: ritmo_ml Test Coverage - COMPLETED
Comprehensive test suite for ritmo_ml with 17 tests (previously 8 were empty/ignored).
- Created test_helpers module with in-memory test databases
- Realistic test data with duplicate entities (Stephen King variants, etc.)
- Full coverage: db_loaders (4), merge operations (4), deduplication (2), patterns (7)
- All tests passing in ~10ms
See [Session History](docs/sessions/2026-01-sessions.md) for details.

### 2025-12-18 - Session 10: ritmo_ml Phase 2 - COMPLETED
Complete deduplication workflow with database loaders, merge operations, and configurable safety features.
See [Session History](docs/sessions/2025-12-sessions.md) for details.

### 2025-12-18 - Session 9: RitmoReporter Trait System - COMPLETED
Created reporter trait abstraction to decouple output from business logic for GUI compatibility.

### 2025-12-18 - Session 8: Complete CRUD System - COMPLETED
Full CRUD operations for Books and Contents with 5 new CLI commands.

### 2025-12-18 - Session 7: ritmo_ml Phase 1 - COMPLETED
Core ML infrastructure with pattern classification and confidence scoring.

### 2025-12-17 - Filter System Refactoring (Phase 1 & 2) - COMPLETED
Modular architecture with OR logic support and validation.

For complete session history, see [docs/sessions/](docs/sessions/).

## TODO/Next Steps

### High Priority
1. **Portable Bootstrap**: Automatic binary copying to bootstrap/portable_app/
2. **Book Import Level 3**: ebook_parser integration for automatic metadata extraction (95% automation goal)

### Medium Priority
3. **Advanced Filters**: SQL-like query DSL for complex queries
4. **Preset System Phase 3**: Auto-save last filter, interactive editing
5. **Documentation**: Comprehensive user documentation

### Low Priority
6. **GUI Integration**: Update `ritmo_gui` to use `ritmo_config`
7. **ML GUI Integration**: Add deduplication features to GUI
8. **Cover Management**: Extract and display book covers

## Quick Reference

| Task | Command |
|------|---------|
| Build | `cargo build --workspace` |
| Test | `cargo test --workspace` |
| Format | `cargo fmt --all` |
| Lint | `cargo clippy --all -- -D warnings` |
| Run CLI | `cargo run -p ritmo_cli -- [command]` |
| Run GUI | `cargo run -p ritmo_gui` |
| Help | `cargo run -p ritmo_cli -- --help` |

---

For detailed information, always refer to the specialized documentation in `docs/`.
