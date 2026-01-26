# Architecture

This document describes the architecture of the ritmo project.

## Workspace Crates

The project is organized as a Rust workspace with the following crates:

### ritmo_db
- Contains database models (structs) that mirror the SQLite schema
- Located in `src/models/`: books, people, publishers, series, tags, languages, formats, roles, types, aliases, contents
- Junction tables for many-to-many relationships: x_books_contents, x_books_people_roles, x_books_tags, x_contents_languages, x_contents_people_roles
- Database schema in `schema/schema.sql` - comprehensive schema with audit logging, stats caching, and metadata tables
- **i18n System**: Internationalization infrastructure with rust-i18n
  - `i18n_trait`: `I18nDisplayable` trait for consistent translation interface
  - `error_i18n`: `LocalizableError` trait for error message localization
  - `i18n_utils`: Locale detection and management utilities
  - Translation files in `locales/` directory (YAML format)
  - Models implement `I18nDisplayable` trait for translating display names
  - Errors implement `LocalizableError` trait for localized messages
  - See [i18n Guide](i18n.md) for details

### ritmo_db_core
- Low-level database infrastructure layer
- `LibraryConfig`: manages library directory structure (root, database, storage, config, bootstrap)
- `Database`: high-level database connection abstraction
- Template database embedded as bytes (`DB_TEMPLATE`) in `assets/template.db`
- Database initialization: copies from template if missing, recreates from schema.sql if template is corrupt
- Connection pooling via SQLx with configurable max connections and auto-vacuum
- **Filter System**: See [Filter System Documentation](filters.md)

### ritmo_core
- Core business logic and ebook management
- DTOs in `src/dto/`: book_dto, people_dto, publishers_dto, language_dto, alias_dto, content_dto, tags_dto (some placeholders)
- Services in `src/service/`:
  - `storage_service.rs`: File storage operations
  - `book_import_service.rs`: Book import with manual metadata (SHA256 hash, duplicate detection)
  - `book_update_service.rs`: Update book metadata with optional fields
  - `content_update_service.rs`: Update content metadata
  - `delete_service.rs`: Delete operations with file management + cleanup utilities
- Uses SHA2 for content hashing

### ritmo_cli
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
  - `ritmo update-book <id>`: Update book metadata
  - `ritmo delete-book <id>`: Delete book with optional file deletion
  - `ritmo add-content --title "..." [options]`: Create new content
  - `ritmo update-content <id>`: Update content metadata
  - `ritmo delete-content <id>`: Delete content from database
  - `ritmo link-content --content-id <id> --book-id <id>`: Associate content to book
  - `ritmo unlink-content --content-id <id> --book-id <id>`: Remove content-book association
  - `ritmo cleanup`: Remove orphaned entities
  - Global option: `--library PATH` to use specific library temporarily
- Integrates with `ritmo_config` for global settings management
- Auto-detects portable mode when run from bootstrap/portable_app/

### ritmo_config
- Global application configuration management
- `AppSettings`: manages last_library_path, recent_libraries (max 10), UI preferences
- Portable detection: auto-detects if running from bootstrap/portable_app/
- Config location: `~/.config/ritmo/settings.toml` (Linux/Mac) or `%APPDATA%/ritmo/settings.toml` (Windows)
- Functions: `config_dir()`, `settings_file()`, `detect_portable_library()`, `is_running_portable()`
- Shared between GUI and CLI for consistent user experience
- Uses `ritmo_errors` for error handling (no custom error types)

### ritmo_errors
- Shared error types across the project (RitmoErr)
- Includes error variants for: database, I/O, config parsing, paths, ML, etc.
- Conversions from common error types: sqlx::Error, std::io::Error, toml::de::Error, toml::ser::Error, serde_json::Error
- **RitmoReporter Trait** (`reporter.rs`):
  - Trait with 3 methods: `status()`, `progress()`, `error()`
  - `SilentReporter`: no-op implementation for libraries and tests
  - Allows frontends to implement custom reporters (CLI, GUI)

### ritmo_mapping
- Metadata mapping utilities

### ritmo_gui
- Graphical interface built with Slint (modern, lightweight UI framework)
- Minimalista design with sidebar navigation (Books, Authors, Publishers, Series)
- Features: book list view, search functionality, status messages
- Uses async/await for database operations
- Auto-initializes library at ~/RitmoLibrary or ./ritmo_library
- UI defined in `ui/main_window.slint`
- Currently displays sample data; database integration pending

### ritmo_ml
- Machine learning features for entity deduplication (authors, publishers, series)
- See [ML System Documentation](ml-system.md)

### ritmo_search (commented out)
- Search and indexing system (planned)

### ebook_parser (commented out)
- Critical utility for extracting metadata from EPUB files
- Originally standalone, now being integrated
- Must handle ~95% of books automatically (goal: 12,000+ books)

## Directory Structure

### Global Configuration
Created automatically by ritmo_config:

```
~/.config/ritmo/           # Linux/Mac
  └── settings.toml        # Global app settings (last library, recent libraries, preferences)

%APPDATA%/ritmo/           # Windows
  └── settings.toml
```

### Library Structure
Created when library is initialized:

```
library_root/
├── database/              # SQLite database (ritmo.db)
├── storage/
│   ├── books/            # Actual book files
│   ├── covers/           # Cover images
│   └── temp/             # Temporary files
├── config/               # Configuration files
│   ├── ritmo.toml        # Library config
│   └── filters.toml      # Library-specific filter presets
└── bootstrap/
    └── portable_app/     # Portable application binaries (multi-platform)
        ├── ritmo_gui     # GUI executable
        ├── ritmo_cli     # CLI executable
        └── README.md     # Usage instructions
```

**Portable Mode**: When running from `library_root/bootstrap/portable_app/`, ritmo automatically detects and uses the parent library without needing global configuration.

## Database Architecture

- SQLite with comprehensive schema including:
  - Core entities: books, people, publishers, series, languages, formats, tags, types
  - Relationships: books-contents, books-people-roles, books-tags, contents-languages, contents-people-roles
  - System tables: system_config, audit_log, stats_cache
  - Normalized people records with confidence scoring and verification flags
- Template-based initialization: database copied from embedded template (`DB_TEMPLATE`)
- Async operations via SQLx with Tokio runtime
- Connection pooling for concurrent access

## Internationalization (i18n) Architecture

- **Framework**: rust-i18n v3 with YAML translation files
- **Translation Files**: `locales/en.yml` (English), `locales/it.yml` (Italian)
- **Locale Detection**: Priority order - RITMO_LANG env var → LANG env var → "en" default
- **Initialization**: `rust_i18n::i18n!()` macro in `ritmo_db/src/lib.rs`
- **Utilities**: `ritmo_db::i18n_utils` module
  - `detect_locale()` - Auto-detect best locale
  - `set_locale()` - Manually set locale
  - `get_locale()` - Get current locale
  - `init_i18n()` - Initialize with auto-detection
- **Usage**: `t!()` macro for translations throughout codebase
- **Key Convention**: `{namespace}.{category}.{subcategory}.{key}`
- **Model Integration**:
  - `I18nDisplayable` trait provides consistent translation interface for models
  - `Role::display_name()` translates role keys (e.g., "role.author" → "Author"/"Autore")
  - `RunningLanguages::display_role()` translates language role keys
  - `Type::display_name()` translates type keys (e.g., "type.novel" → "Novel"/"Romanzo")
  - `Format::display_name()` translates format keys (e.g., "format.epub" → "EPUB (ebook)")
  - All models delegate to the `translate()` method from `I18nDisplayable` trait
- **Error Localization**:
  - `LocalizableError` trait provides error message localization
  - `RitmoErr::localized_message()` returns translated error messages
  - 48 error translation keys covering all 40 RitmoErr variants
  - Keeps `ritmo_errors` crate independent, adds i18n in `ritmo_db`
- **Current Coverage**: ~54 initial keys (db.*, cli.*, error.*, gui.*, validation.*)
- **See**: [i18n Guide](i18n.md) for complete documentation

## Key Patterns

### Application Configuration Pattern
1. Load global settings: `AppSettings::load_or_create(settings_file()?)?`
2. Detect portable mode: `detect_portable_library()` returns Some(path) if portable
3. Get library to use: `app_settings.get_library_to_use()` (portable > last_library > None)
4. Update recent libraries: `app_settings.update_last_library(path)`
5. Save settings: `app_settings.save(settings_path)?`

### LibraryConfig workflow
1. Create config with root path: `LibraryConfig::new(path)`
2. Initialize directories: `config.initialize()`
3. Initialize database: `config.initialize_database().await`
4. Validate setup: `config.validate()` and `config.health_check()`
5. Save config: `config.save(config.main_config_file())?`
6. Create connection pool: `config.create_pool().await`

### Configuration Files
- Global app config: `~/.config/ritmo/settings.toml` (managed by `ritmo_config`)
- Per-library config: `{library}/config/ritmo.toml` (managed by `ritmo_db_core::LibraryConfig`)
- Library filter presets: `{library}/config/filters.toml` (managed by `ritmo_db_core::LibraryPresets`)

### Async/await
All database operations use async/await with Tokio runtime.

### Workspace dependencies
Common dependencies (serde, sqlx, tokio, chrono, toml) are defined in workspace Cargo.toml and inherited by members.

## Environment Variables

### Development Variables
Copy `.env.example` to `.env` for local development:
- `DATABASE_URL`: SQLite database path (e.g., `sqlite://./data/ritmo.db`)
- `RITMO_PORT`: HTTP backend port (default: 8080)
- `RITMO_LOG_LEVEL`: Logging level (debug, info, warn, error)

**Never commit `.env` files** - use `.env.example` for templates.

### i18n Variables
Control application language:
- `RITMO_LANG`: Override language (e.g., `RITMO_LANG=it` for Italian)
- `LANG`: System locale (e.g., `LANG=it_IT.UTF-8`, auto-extracted to "it")

Priority: `RITMO_LANG` > `LANG` > default ("en")

## Rust Version

Required Rust version: **stable** (currently 1.91+) (specified in `rust-toolchain.toml`)
- Updated from 1.75 to support Slint GUI framework and modern dependencies
- Edition 2024 features are now available

## Adding New Crates

1. Create crate: `cargo new --lib crate_name` or `cargo new --bin crate_name`
2. Add to workspace `members` in root `Cargo.toml`
3. Document in this file or dedicated documentation
