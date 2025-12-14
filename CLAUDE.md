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

# Show help
cargo run -p ritmo_cli -- --help
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

**ritmo_core**
- Core business logic and ebook management
- DTOs in `src/dto/`: book_dto, people_dto, publishers_dto, language_dto, alias_dto, content_dto, tags_dto (some placeholders)
- Services in `src/service/`: storage_service.rs for file storage operations
- Uses SHA2 for content hashing

**ritmo_cli**
- Command-line interface with full library management
- Uses clap for CLI argument parsing with subcommands
- Commands:
  - `ritmo init [PATH]`: Initialize/create new library
  - `ritmo list-libraries`: Show recent libraries (max 10)
  - `ritmo info`: Display current library information
  - `ritmo set-library PATH`: Set current library
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

### 2025-12-14 - Session 2: Filter System Implementation (IN PROGRESS)

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

### High Priority
1. **Complete Filter Implementation** (✅ COMPLETED - 2025-12-14):
   - ✅ CLI commands `list-books` and `list-contents` with full options
   - ✅ Filter structs and query builder with tests (8/8 passing)
   - ✅ Execute queries against real database
   - ✅ Create result structs (BookResult, ContentResult) and format output (table, json, simple)
   - ✅ Full integration tested end-to-end
   
2. **Filter Preset System - Phase 1** (✅ COMPLETED - 2025-12-14):
   - ✅ Phase 1: Global presets in `~/.config/ritmo/settings.toml`
   - ✅ Commands: save-preset, list-presets, delete-preset
   - ✅ --preset flag for list-books and list-contents
   - ✅ Parameter override priority (CLI > preset > default)
   - ⏸️ Phase 2: Library presets in `library/config/filters.toml` (FUTURE - for portable mode)
   - ⏸️ Phase 3: Auto-save last filter, interactive editing (FUTURE)
   - See "Filter Preset System" section below for full architecture
   
3. **GUI Integration**: Update `ritmo_gui` to use `ritmo_config` and add library selection dialog

4. **CLI Book Import**: Add `ritmo add <file>` command to import books into library

### Medium Priority
4. **Portable Bootstrap**: Implement automatic copying of binaries to bootstrap/portable_app/ during library initialization
5. **Multi-platform Binaries**: Handle cross-compilation for Linux/Windows/Mac binaries in bootstrap
6. **Documentation**: Create user documentation for portable library usage

### Low Priority
7. **Advanced Filters**: Implement `--filter` option with SQL-like query DSL for complex queries

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
