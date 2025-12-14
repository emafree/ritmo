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

## Recent Changes (2025-12-14)

### New Crate: ritmo_config
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

1. **GUI Integration**: Update `ritmo_gui` to use `ritmo_config` and add library selection dialog
2. **Portable Bootstrap**: Implement automatic copying of binaries to bootstrap/portable_app/ during library initialization
3. **Multi-platform Binaries**: Handle cross-compilation for Linux/Windows/Mac binaries in bootstrap
4. **CLI Book Management**: Add commands for adding/listing/searching books
5. **Documentation**: Create user documentation for portable library usage
