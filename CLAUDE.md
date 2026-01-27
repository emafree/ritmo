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
cargo run -p ritmo_cli -- init

# Import a book
cargo run -p ritmo_cli -- add book.epub --title "Book Title" --author "Author Name"

# List books
cargo run -p ritmo_cli -- list-books
```

For complete build and run commands, see [Development Guide](docs/development.md).

## Documentation Structure

Detailed documentation is organized in the `docs/` directory:

- **[Architecture](docs/architecture.md)** - Workspace crates, database schema, directory structure, key patterns
- **[Development Guide](docs/development.md)** - Build, test, and run commands
- **[Filter System](docs/filters.md)** - Comprehensive filter and preset system documentation
- **[ML System](docs/ml-system.md)** - Entity deduplication with machine learning
- **[Book Metadata Format](docs/book_metadata_format.md)** - JSON format specification for Levels 2 & 3
- **[Session History](docs/sessions/)** - Chronological changelog of all development sessions

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
- Command: `add book.epub --title "Title" --author "Author"`

**Level 2 - Batch Import via Pipe (PLANNED)**
- Import multiple books from JSON metadata file or stdin
- Uses same JSON format as Level 3 ebook_parser output
- Enables review/edit workflow: extract metadata → review → batch import
- Supports per-book metadata with optional shared defaults
- Input format: JSON array of book metadata objects
- Examples:
  - `ritmo add-batch --input books_metadata.json`
  - `cat books_metadata.json | ritmo add-batch`
  - `ritmo extract-metadata ~/books/*.epub > metadata.json` (Level 3)
  - `# Review/edit metadata.json, then:`
  - `ritmo add-batch --input metadata.json`

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
cargo run -p ritmo_cli -- init [PATH]           # Initialize library
cargo run -p ritmo_cli -- info                  # Show library info
cargo run -p ritmo_cli -- list-libraries        # Show recent libraries
cargo run -p ritmo_cli -- set-library PATH      # Set current library
```

### Book Operations
```bash
# Add book
cargo run -p ritmo_cli -- add book.epub --title "Title" --author "Author"

# List books with filters
cargo run -p ritmo_cli -- list-books --author "King" --format epub

# Update book
cargo run -p ritmo_cli -- update-book 1 --title "New Title"

# Delete book (database only)
cargo run -p ritmo_cli -- delete-book 1

# Delete book with physical file
cargo run -p ritmo_cli -- delete-book 1 --delete-file

# Force deletion even if file errors occur
cargo run -p ritmo_cli -- delete-book 1 --delete-file --force

# Cleanup orphaned entities (people, publishers, series, tags, formats)
cargo run -p ritmo_cli -- cleanup
```

### Content Operations
```bash
# Create new content
cargo run -p ritmo_cli -- add-content --title "Story Title" --author "Author Name"
cargo run -p ritmo_cli -- add-content --title "Novel" --content-type "Romanzo" --year 2024

# Create content and associate to book
cargo run -p ritmo_cli -- add-content --title "Novel" --author "Author" --book-id 1

# Update content
cargo run -p ritmo_cli -- update-content 1 --title "New Title" --year 2024

# Delete content
cargo run -p ritmo_cli -- delete-content 1

# Associate/unassociate content and book
cargo run -p ritmo_cli -- link-content --content-id 1 --book-id 1
cargo run -p ritmo_cli -- unlink-content --content-id 1 --book-id 1
```

### ML Deduplication Operations
```bash
# Find duplicate people (authors, translators, etc.) - dry-run by default
cargo run -p ritmo_cli -- deduplicate-people --dry-run

# Merge duplicate people with custom threshold
cargo run -p ritmo_cli -- deduplicate-people --threshold 0.90 --auto-merge

# Find duplicate publishers
cargo run -p ritmo_cli -- deduplicate-publishers --dry-run

# Find duplicate series
cargo run -p ritmo_cli -- deduplicate-series --dry-run

# Find duplicate tags
cargo run -p ritmo_cli -- deduplicate-tags --dry-run

# Run deduplication for all entity types (people, publishers, series, tags, roles)
cargo run -p ritmo_cli -- deduplicate-all --threshold 0.85 --dry-run
```

For complete command reference, see [Development Guide](docs/development.md).

## Rust Version

Required: **stable** (currently 1.91+) as specified in `rust-toolchain.toml`
- Edition 2024 features available
- Supports Slint GUI framework

## Recent Changes

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
2. **Book Import Level 2**: Batch import via pipe (file/stdin) for bulk operations
3. **Book Import Level 3**: ebook_parser integration for automatic metadata extraction (95% automation goal)

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
