# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ritmo** is a Rust-based library management system inspired by Calibre, but focused solely on cataloging books and their metadata (not editing, reading, or converting). The primary goal is to catalog books, their contents, and contributors (authors, translators, illustrators, editors).

The project uses SQLite for database storage (no external server required) and is organized as a Rust workspace with multiple specialized crates.

## Build and Test Commands

### Building
```bash
# Build entire workspace
cargo build --workspace

# Build specific crate
cargo build -p ritmo_cli
cargo build -p ritmo_core

# Build in release mode
cargo build --workspace --release
```

### Testing
```bash
# Run all tests in workspace
cargo test --workspace

# Run tests for specific crate
cargo test -p ritmo_db_core
cargo test -p ritmo_core

# Run specific test by name
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Code Quality
```bash
# Format all code
cargo fmt --all

# Run linter
cargo clippy --all -- -D warnings

# Check code without building
cargo check --workspace
```

### Running

#### CLI Commands
```bash
# Initialize a new library (default: ~/RitmoLibrary)
cargo run -p ritmo_cli -- init
cargo run -p ritmo_cli -- init /path/to/library

# Show current library info
cargo run -p ritmo_cli -- info

# List all recent libraries
cargo run -p ritmo_cli -- list-libraries

# Set current library
cargo run -p ritmo_cli -- set-library /path/to/library

# Use specific library temporarily (doesn't change default)
cargo run -p ritmo_cli -- --library /path/to/library info

# Import a book into the library
cargo run -p ritmo_cli -- add /path/to/book.epub --title "Book Title" --author "Author Name"
cargo run -p ritmo_cli -- add book.pdf --title "My Book" --author "John Doe" --publisher "Publisher" --year 2024 --isbn "978-1234567890" --series "Series Name" --series-index 1 --notes "Some notes"

# List books with filters
cargo run -p ritmo_cli -- list-books
cargo run -p ritmo_cli -- list-books --author "Stephen King"
cargo run -p ritmo_cli -- list-books --format epub --year 2024

# Filter by acquisition date (absolute)
cargo run -p ritmo_cli -- list-books --acquired-after 2024-01-01
cargo run -p ritmo_cli -- list-books --acquired-before 2024-12-31

# Filter by acquisition date (relative)
cargo run -p ritmo_cli -- list-books --last-days 7        # Last 7 days
cargo run -p ritmo_cli -- list-books --last-months 1      # Last month
cargo run -p ritmo_cli -- list-books --recent-count 10    # 10 most recent books

# Update book metadata
cargo run -p ritmo_cli -- update-book 1 --title "New Title" --year 2024
cargo run -p ritmo_cli -- update-book 1 --author "New Author" --notes "Updated notes"

# Delete book
cargo run -p ritmo_cli -- delete-book 1                   # Delete record only
cargo run -p ritmo_cli -- delete-book 1 --delete-file     # Delete record + file
cargo run -p ritmo_cli -- delete-book 1 --delete-file --force  # Force deletion

# Update content metadata
cargo run -p ritmo_cli -- update-content 1 --title "New Title" --author "New Author"
cargo run -p ritmo_cli -- update-content 1 --content-type "Romanzo" --year 2024

# Delete content
cargo run -p ritmo_cli -- delete-content 1

# Cleanup orphaned entities (authors, publishers, series not referenced)
cargo run -p ritmo_cli -- cleanup
cargo run -p ritmo_cli -- cleanup --dry-run               # Preview without changes

# Show help
cargo run -p ritmo_cli -- --help
cargo run -p ritmo_cli -- add --help
cargo run -p ritmo_cli -- update-book --help
```

#### GUI Application
```bash
# Run GUI application
cargo run -p ritmo_gui

# Build GUI in release mode (smaller and faster)
cargo build -p ritmo_gui --release
./target/release/ritmo_gui
```

## Architecture

### Workspace Crates

The project is organized as a Rust workspace with the following crates:

**ritmo_db**
- Contains database models (structs) that mirror the SQLite schema
- Located in `src/models/`: books, people, publishers, series, tags, languages, formats, roles, types, aliases, contents
- Junction tables for many-to-many relationships: x_books_contents, x_books_people_roles, x_books_tags, x_contents_languages, x_contents_people_roles
- Database schema in `schema/schema.sql` - comprehensive schema with audit logging, stats caching, and metadata tables

**ritmo_db_core**
- Low-level database infrastructure layer
- `LibraryConfig`: manages library directory structure (root, database, storage, config, bootstrap)
- `Database`: high-level database connection abstraction
- Template database embedded as bytes (`DB_TEMPLATE`) in `assets/template.db`
- Database initialization: copies from template if missing, recreates from schema.sql if template is corrupt
- Connection pooling via SQLx with configurable max connections and auto-vacuum
- **Filter System** (modular architecture):
  - `filters/types.rs`: Filter structures (BookFilters, ContentFilters) with Vec<String> for OR logic
  - `filters/builder.rs`: SQL query construction with OR/AND logic support
  - `filters/executor.rs`: Query execution with parameter binding
  - `filters/validator.rs`: Input validation (limits, dates, empty values)
  - Supports multiple values with OR logic: `--author "King" --author "Tolkien"` → `(author='King' OR author='Tolkien')`
  - Different filter types use AND logic: author AND format AND year
  - Backward compatible with old API via helper methods

**ritmo_core**
- Core business logic and ebook management
- DTOs in `src/dto/`: book_dto, people_dto, publishers_dto, language_dto, alias_dto, content_dto, tags_dto (some placeholders)
- Services in `src/service/`:
  - `storage_service.rs`: File storage operations
  - `book_import_service.rs`: Book import with manual metadata (SHA256 hash, duplicate detection)
- Uses SHA2 for content hashing

**ritmo_cli**
- Command-line interface with full library management
- Uses clap for CLI argument parsing with subcommands
- Commands:
  - `ritmo init [PATH]`: Initialize/create new library
  - `ritmo list-libraries`: Show recent libraries (max 10)
  - `ritmo info`: Display current library information
  - `ritmo set-library PATH`: Set current library
  - `ritmo add <file> --title "..." [options]`: Import a book with manual metadata
  - `ritmo list-books [--preset NAME] [--author ...] [--format ...] [--output table|json|simple]`: List books with filters
  - `ritmo list-contents [--preset NAME] [--author ...] [--output table|json|simple]`: List contents with filters
  - `ritmo save-preset books|contents --name NAME [--author ...] [--format ...]`: Save filter preset
  - `ritmo list-presets [books|contents]`: Show saved presets
  - `ritmo delete-preset books|contents NAME`: Delete preset
  - Global option: `--library PATH` to use specific library temporarily
- Integrates with `ritmo_config` for global settings management
- Auto-detects portable mode when run from bootstrap/portable_app/

**ritmo_config** (NEW - 2025-12-14)
- Global application configuration management
- `AppSettings`: manages last_library_path, recent_libraries (max 10), UI preferences
- Portable detection: auto-detects if running from bootstrap/portable_app/
- Config location: `~/.config/ritmo/settings.toml` (Linux/Mac) or `%APPDATA%/ritmo/settings.toml` (Windows)
- Functions: `config_dir()`, `settings_file()`, `detect_portable_library()`, `is_running_portable()`
- Shared between GUI and CLI for consistent user experience
- Uses `ritmo_errors` for error handling (no custom error types)

**ritmo_errors**
- Shared error types across the project (RitmoErr)
- Includes error variants for: database, I/O, config parsing, paths, ML, etc.
- Conversions from common error types: sqlx::Error, std::io::Error, toml::de::Error, toml::ser::Error, serde_json::Error

**ritmo_mapping**
- Metadata mapping utilities

**ritmo_gui**
- Graphical interface built with Slint (modern, lightweight UI framework)
- Minimalista design with sidebar navigation (Books, Authors, Publishers, Series)
- Features: book list view, search functionality, status messages
- Uses async/await for database operations
- Auto-initializes library at ~/RitmoLibrary or ./ritmo_library
- UI defined in `ui/main_window.slint`
- Currently displays sample data; database integration pending

**ritmo_search** (commented out)
- Search and indexing system (planned)

**ritmo_ml** (commented out)
- Machine learning features (planned)

**ebook_parser** (commented out)
- Critical utility for extracting metadata from EPUB files
- Originally standalone, now being integrated
- Must handle ~95% of books automatically (goal: 12,000+ books)

### Directory Structure

**Global Configuration** (created automatically):
```
~/.config/ritmo/           # Linux/Mac
  └── settings.toml        # Global app settings (last library, recent libraries, preferences)

%APPDATA%/ritmo/           # Windows
  └── settings.toml
```

**Library Structure** (when initialized):
```
library_root/
├── database/              # SQLite database (ritmo.db)
├── storage/
│   ├── books/            # Actual book files
│   ├── covers/           # Cover images
│   └── temp/             # Temporary files
├── config/               # Configuration files (ritmo.toml)
└── bootstrap/
    └── portable_app/     # Portable application binaries (multi-platform)
        ├── ritmo_gui     # GUI executable
        ├── ritmo_cli     # CLI executable
        └── README.md     # Usage instructions
```

**Portable Mode**: When running from `library_root/bootstrap/portable_app/`, ritmo automatically detects and uses the parent library without needing global configuration.

### Database Architecture

- SQLite with comprehensive schema including:
  - Core entities: books, people, publishers, series, languages, formats, tags, types
  - Relationships: books-contents, books-people-roles, books-tags, contents-languages, contents-people-roles
  - System tables: system_config, audit_log, stats_cache
  - Normalized people records with confidence scoring and verification flags
- Template-based initialization: database copied from embedded template (`DB_TEMPLATE`)
- Async operations via SQLx with Tokio runtime
- Connection pooling for concurrent access

### Key Patterns

**Application Configuration Pattern** (NEW):
1. Load global settings: `AppSettings::load_or_create(settings_file()?)?`
2. Detect portable mode: `detect_portable_library()` returns Some(path) if portable
3. Get library to use: `app_settings.get_library_to_use()` (portable > last_library > None)
4. Update recent libraries: `app_settings.update_last_library(path)`
5. Save settings: `app_settings.save(settings_path)?`

**LibraryConfig workflow**:
1. Create config with root path: `LibraryConfig::new(path)`
2. Initialize directories: `config.initialize()`
3. Initialize database: `config.initialize_database().await`
4. Validate setup: `config.validate()` and `config.health_check()`
5. Save config: `config.save(config.main_config_file())?`
6. Create connection pool: `config.create_pool().await`

**Configuration Files**:
- Global app config: `~/.config/ritmo/settings.toml` (managed by `ritmo_config`)
- Per-library config: `{library}/config/ritmo.toml` (managed by `ritmo_db_core::LibraryConfig`)

**Async/await**: All database operations use async/await with Tokio runtime

**Workspace dependencies**: Common dependencies (serde, sqlx, tokio, chrono, toml) are defined in workspace Cargo.toml and inherited by members

### Filter System Architecture (ritmo_db_core/src/filters/)

The filter system is organized as a modular, isolated subsystem:

**Structure**:
```
filters/
├── mod.rs          # Public API and documentation
├── types.rs        # Filter structures and result types
├── builder.rs      # SQL query construction
├── executor.rs     # Query execution
└── validator.rs    # Input validation
```

**OR Logic Support**:
```rust
// Multiple values for the same filter → OR logic
let filters = BookFilters::default()
    .with_author("King")
    .with_author("Tolkien")
    .with_format("epub");

// SQL: (author LIKE '%King%' OR author LIKE '%Tolkien%') AND format LIKE '%epub%'
```

**Field Types**:
- `authors: Vec<String>` - Multiple authors (OR)
- `publishers: Vec<String>` - Multiple publishers (OR)
- `formats: Vec<String>` - Multiple formats (OR)
- `series_list: Vec<String>` - Multiple series (OR)
- `year: Option<i32>` - Single year (exact match)
- `acquired_after/before: Option<i64>` - Date range

**Validation**:
- Maximum 50 values per filter (performance limit)
- No negative offsets
- Positive limits only
- Valid date ranges (after < before)
- No empty filter values

**Builder Pattern**:
```rust
// Fluent API
BookFilters::default()
    .with_author("Calvino")
    .with_format("epub")
    .with_format("pdf")
    // SQL: author LIKE '%Calvino%' AND (format LIKE '%epub%' OR format LIKE '%pdf%')
```

**Backward Compatibility**:
```rust
// Old way (still works)
filters.set_author_opt(Some("King".to_string()))

// New way (preferred)
filters.with_author("King")
```

**Usage Example**:
```rust
use ritmo_db_core::filters::{BookFilters, execute_books_query, validate_book_filters};

let filters = BookFilters::default()
    .with_author("King")
    .with_author("Tolkien")
    .with_format("epub");

// Validate before executing
validate_book_filters(&filters)?;

// Execute query
let pool = config.create_pool().await?;
let books = execute_books_query(&pool, &filters).await?;
```

## Environment Variables

Copy `.env.example` to `.env` for local development:
- `DATABASE_URL`: SQLite database path (e.g., `sqlite://./data/ritmo.db`)
- `RITMO_PORT`: HTTP backend port (default: 8080)
- `RITMO_LOG_LEVEL`: Logging level (debug, info, warn, error)

**Never commit `.env` files** - use `.env.example` for templates.

## Rust Version

Required Rust version: **stable** (currently 1.91+) (specified in `rust-toolchain.toml`)
- Updated from 1.75 to support Slint GUI framework and modern dependencies
- Edition 2024 features are now available

## Adding New Crates

1. Create crate: `cargo new --lib crate_name` or `cargo new --bin crate_name`
2. Add to workspace `members` in root `Cargo.toml`
3. Document in `docs/workspace.md`

## Testing Notes

- Many crates use `serial_test` for tests that access shared resources (database)
- Dev dependencies include `tempfile` for temporary test directories
- Use `tokio-test` for async test utilities

## Recent Changes

### 2025-12-18 - Session 9: RitmoReporter Trait System - COMPLETED

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

### 2025-12-18 - Session 7: ritmo_ml Activation - Phase 1 (Core ML Infrastructure) - COMPLETED

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

### 2025-12-18 - Session 8: Complete CRUD System Implementation - COMPLETED

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

### 2025-12-17 - Session 7: Filter System Refactoring (Phase 1 & 2) - COMPLETED

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

### 2025-12-16 - Session 6: Relative Date Filters for Book Acquisition - COMPLETED

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

### 2025-12-16 - Session 5: Library-Specific Preset System (Phase 2) - COMPLETED

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

### 2025-12-14 - Session 2: Filter System Implementation (COMPLETED)

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

### 2025-12-14 - Session 1: Configuration System

**New Crate: ritmo_config**
- Created `ritmo_config` crate for global application configuration
- Manages `~/.config/ritmo/settings.toml` with last_library_path and recent_libraries
- Implements portable mode detection (auto-detects if running from bootstrap/portable_app/)
- Shared between GUI and CLI for consistent behavior
- Integrated with `ritmo_errors` (no custom error types)

### CLI Improvements
- Refactored `ritmo_cli` from simple demo to full-featured command-line tool
- Added subcommands: init, info, list-libraries, set-library
- Global `--library PATH` option to temporarily override library
- Auto-saves to recent libraries when initializing or using libraries
- Fully integrated with `ritmo_config` for global settings

### GUI Status
- GUI (`ritmo_gui`) not yet updated to use `ritmo_config`
- Currently uses hardcoded library path (~/RitmoLibrary or ./ritmo_library)
- TODO: Integrate library selection dialog and `ritmo_config` support

### Error Handling
- Extended `ritmo_errors::RitmoErr` with config-related variants
- Added conversions from toml::de::Error and toml::ser::Error
- All crates now use shared error types consistently

## TODO/Next Steps

### Completed
1. **Complete Filter Implementation** (✅ COMPLETED - 2025-12-14)
2. **Filter Preset System** (✅ COMPLETED - 2025-12-16)
3. **Complete CRUD System** (✅ COMPLETED - 2025-12-18):
   - ✅ UPDATE operations for Books and Contents
   - ✅ DELETE operations with file management
   - ✅ CLEANUP command for orphaned entities
   - ✅ 5 new CLI commands (update-book, delete-book, update-content, delete-content, cleanup)
   - ✅ Optional field updates, relationship management, CASCADE deletion
   - ✅ End-to-end testing verified

### High Priority
1. **Portable Bootstrap**: 
   - Implement automatic copying of binaries to bootstrap/portable_app/ during library initialization
   - Handle cross-compilation for Linux/Windows/Mac binaries
   - Complete portable library distribution workflow

### Medium Priority
2. **Advanced Filters**: Implement `--filter` option with SQL-like query DSL for complex queries
3. **Preset System Phase 3**: Auto-save last filter, interactive editing
4. **Documentation**: Create comprehensive user documentation

### Low Priority
5. **GUI Integration**: Update `ritmo_gui` to use `ritmo_config` and add library selection dialog
6. **Integrate ebook_parser**: 
   - Reactivate and integrate the `ebook_parser` crate
   - Extract metadata automatically from EPUB files
   - Goal: handle ~95% of books automatically (target: 12,000+ books)

## Filter Preset System (Planned Architecture)

### Two-Level Filter Storage

**1. Global Presets** (`~/.config/ritmo/settings.toml`):
- User's personal filter preferences
- Not portable with library
- Used across all libraries
- Examples: "my favorites", "recent additions"

**2. Library Presets** (`library/config/filters.toml`):
- Library-specific filter configuration
- ✅ Portable with library (critical for portable mode!)
- Travels with library when copied/shared
- Examples: "default view", "collection-specific filters"

### Filter Resolution Order (Priority)

When executing `ritmo list-books`:
1. Explicit CLI options (`--author`, `--format`, etc.) - highest priority
2. `--preset <name>` (searches library first, then global)
3. Library default filter (from `library/config/filters.toml`)
4. Last used filter (global `~/.config/ritmo/last_filters`)
5. No filter (list all) - lowest priority

### File Structure

```
~/.config/ritmo/
├── settings.toml          # Global config + global presets + last_filters

/path/to/library/
├── config/
│   ├── ritmo.toml        # Library config (existing)
│   └── filters.toml      # Library-specific presets (NEW)
```

### Implementation Phases

**Phase 1** (Next session - Foundation):
- Data structures: `FilterPreset`, preset management
- Save/load global presets in `~/.config/ritmo/settings.toml`
- Commands: `save-preset`, `list-presets`, `--preset <name>`

**Phase 2** (Essential - Portability):
- Save/load library presets in `library/config/filters.toml`
- Resolution order (library > global)
- Default filters per library
- Include example filters when creating new library

**Phase 3** (UX Enhancement):
- Auto-save last used filter
- Commands: `--use-last`, `--clear-filters`
- Interactive preset editing

### Example Commands

```bash
# Global presets
ritmo save-preset books --name "my_ebooks" --format epub
ritmo list-presets
ritmo delete-preset "my_ebooks"

# Library presets (portable!)
ritmo save-preset books --name "default_view" --format epub --library
ritmo set-default-filter books default_view
ritmo list-presets --library-only

# Usage
ritmo list-books --preset default_view  # Searches library first, then global
ritmo list-books                        # Uses library default if set
ritmo list-books --use-last            # Uses last global filter
```

### Portable Library Workflow

```bash
# Create library with useful presets
ritmo init /media/usb/SharedLibrary
ritmo save-preset books --name "epub_only" --format epub --library
ritmo set-default-filter books epub_only

# Copy to USB and share
# Colleague opens portable library
cd /media/usb/SharedLibrary/bootstrap/portable_app
./ritmo_gui    # Opens with "epub_only" filter already active!
./ritmo_cli list-books  # Automatically uses "epub_only" preset
```
