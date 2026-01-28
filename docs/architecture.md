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
  - `book_import_service.rs`: Book import with manual metadata (SHA256 hash-based storage, duplicate detection)
  - `book_update_service.rs`: Update book metadata with optional fields
  - `content_update_service.rs`: Update content metadata
  - `delete_service.rs`: Delete operations with file management + cleanup utilities
  - `batch_import_service.rs`: Batch import for multiple books from JSON
- Uses SHA2 for content hashing and hash-based file storage

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
- Part of Level 3 book import automation (see Book Import Levels below)

## Book Import Levels

The book import system is designed with progressive automation levels:

### Level 1 - Manual Import (IMPLEMENTED)
**Status**: Fully functional in ritmo_cli

**Location**:
- CLI: `ritmo_cli/src/commands/books.rs` - `cmd_add()`
- Service: `ritmo_core/src/service/book_import_service.rs` - `import_book()`

**Features**:
- Single book import with command-line arguments
- Title is required, all other metadata optional
- Format auto-detected from file extension
- SHA256 hash calculation for duplicate detection
- Full entity management (authors, publishers, series, tags)

**Usage**:
```bash
cargo run -p ritmo_cli -- add book.epub --title "Title" --author "Author"
```

### Level 2 - Batch Import via Pipe (IMPLEMENTED)
**Status**: Fully implemented and tested ✅

**Implementation**:
- **Location**:
  - Service: `ritmo_core/src/service/batch_import_service.rs`
  - DTOs: `ritmo_core/src/dto/batch_import_dto.rs`
  - CLI: `ritmo_cli/src/commands/books.rs` - `cmd_add_batch()`
- **Testing**: Comprehensive tests with single/multi-book imports, validation, duplicates, stdin
- **Session**: Session 21 (2026-01-27)

**Design Goals** (Achieved):
- Accept JSON metadata file from stdin or file input
- Use same JSON format as Level 3 ebook_parser output for seamless integration
- Enable review/edit workflow: extract → review → import
- Support per-book metadata with optional shared defaults
- Maintain same validation and duplicate detection as Level 1

**Implemented Features**:
- ✅ Read metadata from file: `--input books_metadata.json`
- ✅ Read from stdin pipe: `cat books_metadata.json | ritmo add-batch`
- ✅ Ready for Level 3 integration: `ritmo extract-metadata *.epub | ritmo add-batch`
- ✅ Progress reporting for batch operations (N/M books imported)
- ✅ Error handling modes:
  - `--stop-on-error`: abort on first failure (default, implemented as default behavior)
  - `--continue-on-error`: skip failed books, report at end (flag implemented)
  - `--dry-run`: validate metadata without importing (flag implemented)
- ✅ Duplicate detection: skip books with existing SHA256 hash
- ✅ Summary report: success/failure counts, skipped duplicates, errors
- ✅ Full import: books + contents + relationships (people, languages, tags, series)
- ✅ Comprehensive validation: 16 validation rules with detailed error messages

**JSON Input Format**:
The batch import uses a JSON array of import objects, where each object represents a physical book file and its associated contents. This structure reflects ritmo's database architecture with Books and Contents as separate entities. See [book_metadata_format.json](book_metadata_format.json) for a complete example file.

Structure:

```json
[
  {
    "file_path": "/absolute/path/to/book.epub",
    "book": {
      "title": "Complete Works Edition",
      "original_title": "Original Title",
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
      "contents[0].title": 0.95,
      "contents[0].people": 0.90
    }
  }
]
```

**Field Specifications**:

*Import Object Level:*
- `file_path` (required): Absolute or relative path to book file
- `book` (required): Book object (see below)
- `contents` (optional): Array of content objects (see below)
- `confidence` (optional): Confidence scores for Level 3 extracted fields (ignored during import)

*Book Object:*
- `title` (required): Book title (physical edition title)
- `original_title` (optional): Original title if different
- `people` (optional): Array of {name, role} objects for book-level contributors (editors, preface writers)
- `publisher` (optional): Publisher name
- `year` (optional): Publication year of this edition (integer)
- `isbn` (optional): ISBN identifier
- `format` (optional): File format (auto-detected if omitted)
- `series` (optional): Series name
- `series_index` (optional): Position in series (integer)
- `pages` (optional): Page count (integer)
- `notes` (optional): Free-text notes
- `tags` (optional): Array of tag strings

*Content Object:*
- `title` (required): Content title (work title)
- `original_title` (optional): Original title if different
- `people` (optional): Array of {name, role} objects for content creators (authors, translators)
- `type` (optional): Content type (i18n key: "type.novel", "type.short_story", "type.essay", etc.)
- `year` (optional): Original publication year of work (integer)
- `languages` (optional): Array of {code, role} objects (role uses i18n keys: "language_role.original", "language_role.actual", etc.)

*People Object:*
- `name` (required): Person name
- `role` (required): Role i18n key ("role.author", "role.translator", "role.editor", etc.)

**Book vs Content Level**:
- **Book-level**: Physical edition metadata (publisher, ISBN, series, format, pages, edition contributors)
- **Content-level**: Literary work metadata (authors, translators, original year, languages, type)
- A book can contain multiple contents (collections, omnibus editions)
- If `contents` is empty or omitted, only book metadata is imported

**Workflow Integration with Level 3**:
```bash
# Step 1: Extract metadata from EPUBs (Level 3)
ritmo extract-metadata ~/books/*.epub --output metadata.json

# Step 2: Review and edit metadata.json manually
# - Check confidence scores (low-confidence fields)
# - Fix incorrect extractions (wrong authors, series, etc.)
# - Add missing data (tags, notes, missing contributors)
# - Split or merge contents if needed
# - Remove unwanted books from the array

# Step 3: Batch import with reviewed metadata (Level 2)
ritmo add-batch --input metadata.json

# Alternative: Combined one-step workflow for trusted sources
ritmo extract-metadata ~/books/*.epub | ritmo add-batch

# Or with confidence filtering:
ritmo extract-metadata ~/books/*.epub --min-confidence 0.85 | ritmo add-batch
```

**Key Integration Points**:
- Level 3 output = Level 2 input (same JSON schema)
- Confidence scores help identify fields needing manual review
- Manual editing step allows correction before database import
- One-step pipeline available for trusted sources

**Implementation Details**:
- New CLI command: `add-batch` (separate from `add` for clarity)
- Parser: Use `serde_json` to deserialize JSON array of import objects
- Validation: Check required fields before processing:
  - Import level: `file_path`, `book` object
  - Book level: `book.title`
  - Content level: `contents[].title` (if contents present)
- Import service refactoring:
  - Extend or create new service to handle book + contents import
  - Current `book_import_service::import_book()` handles single book without contents
  - Need service to import book + create contents + associate them
- Database operations:
  1. Import book file (hash, storage, create books record)
  2. Create/get book-level entities (publisher, series, people, tags)
  3. For each content:
     - Create contents record
     - Create/get content-level entities (people, languages)
     - Link content to book (x_books_contents)
  4. Link all relationships (x_books_people_roles, x_contents_people_roles, etc.)
- Transaction strategy: per import object (not batch-level) for partial success
- Batch orchestration layer:
  - Progress bar using indicatif crate (N/M books imported)
  - Error collection and reporting (skip failed, continue or abort)
  - Summary: success count, failure count, skipped duplicates
- File path resolution: support both absolute and relative paths
- RitmoReporter integration for consistent output
- Content detection: if `contents` array is empty/missing, create single default content from book metadata

### Level 3 - Automatic Metadata Extraction (PLANNED)
**Status**: ebook_parser crate exists but only as skeleton

**Location**: `ebook_parser/`

**Design Goals**:
- Extract metadata automatically from EPUB files (content.opf)
- Output JSON format compatible with Level 2 batch import
- Achieve 95% automation rate for metadata population
- Handle 12,000+ books without manual intervention
- Provide confidence scores for user review

**Planned Features**:
- Parse EPUB archive (ZIP format with `zip` crate)
- Extract content.opf (package metadata location from container.xml)
- Parse Dublin Core metadata elements:
  - `dc:title` → title
  - `dc:creator` (with role="aut") → authors
  - `dc:contributor` (with role="trl", etc.) → translators/contributors
  - `dc:publisher` → publisher
  - `dc:date` → year (extract year from date)
  - `dc:identifier` (scheme="ISBN") → isbn
  - `dc:language` → languages
  - `dc:subject` → tags
- Series detection:
  - Check for `<meta property="belongs-to-collection">` (EPUB3)
  - Check for `<meta name="calibre:series">` (Calibre format)
  - Fallback: extract from title or filename patterns
- Page count estimation:
  - Parse spine items and estimate from content length
- Confidence scoring for each extracted field (0.0-1.0)
- Output modes:
  - `--output JSON`: Write JSON file (for Level 2 workflow)
  - `--stdout`: Pipe to stdout (for direct Level 2 integration)
  - `--import`: Direct import mode (extract + import in one step)

**Output Format**:
Same JSON format as Level 2 input (see Level 2 section above). The ebook_parser will:
1. Extract book-level metadata (publisher, ISBN, edition year) from EPUB metadata
2. Detect and extract individual contents (works) within the EPUB:
   - Single-work EPUBs: one content entry
   - Collections/omnibus: multiple content entries (detected from spine structure or TOC)
3. Assign people to appropriate level:
   - Authors, translators → content-level people
   - Editors, edition preface/introduction → book-level people
4. Generate confidence scores for all extracted fields

Example commands:

```bash
# Extract metadata only (for review)
ritmo extract-metadata ~/books/*.epub --output metadata.json

# Extract and pipe to batch import
ritmo extract-metadata ~/books/*.epub | ritmo add-batch

# Extract with confidence threshold (filter out low-confidence)
ritmo extract-metadata ~/books/*.epub --min-confidence 0.80 --output metadata.json

# Extract single file
ritmo extract-metadata book.epub
```

Example output structure:
```json
[
  {
    "file_path": "/path/to/book.epub",
    "book": { /* edition metadata */ },
    "contents": [ /* work(s) metadata */ ],
    "confidence": { /* extraction confidence scores */ }
  }
]
```

**Confidence Scoring Logic**:

*Book-level fields:*
- **book.title**: 0.95 if found in dc:title, 0.50 if extracted from filename
- **book.publisher**: 0.85 if found in dc:publisher
- **book.year**: 0.90 if parsed from dc:date, 0.60 if estimated
- **book.isbn**: 0.95 if found in dc:identifier with scheme="ISBN"
- **book.series**: 0.90 if found in metadata (EPUB3 collection or Calibre meta), 0.60 if extracted from title/filename
- **book.people**: 0.75 if contributors found in dc:contributor (edition-level roles)

*Content-level fields:*
- **contents[N].title**: 0.95 if found in dc:title or spine metadata, 0.60 if inferred
- **contents[N].people**: 0.90 if found in dc:creator with role, 0.70 if role missing
- **contents[N].type**: 0.85 if found in dc:type, 0.60 if inferred from content
- **contents[N].year**: 0.90 if found in metadata, 0.60 if estimated
- **contents[N].languages**: 0.90 if found in dc:language

*General scoring ranges:*
- **0.90-1.0**: Found in proper metadata fields
- **0.70-0.89**: Inferred with medium confidence
- **0.50-0.69**: Extracted from filename or estimated

Confidence keys use dot notation for books (`book.*`) and array indexing for contents (`contents[N].*`).

**Dependencies Ready**:
- `zip = "2.2"` - EPUB archive reading
- `quick-xml = "0.36"` - OPF XML parsing
- `serde = { workspace }` - JSON serialization
- `serde_json` - JSON output generation
- `thiserror = "2.0"` - Error handling

**CLI Integration**:
New command in `ritmo_cli`:
```rust
// ritmo_cli/src/commands/metadata.rs
pub async fn cmd_extract_metadata(
    files: Vec<PathBuf>,
    output: Option<PathBuf>,
    min_confidence: Option<f32>,
) -> Result<(), RitmoErr>
```

**Integration Points**:
- **Level 2 Workflow**: Extract to JSON → review → batch import
- **Level 1 Enhancement**: Use ebook_parser as metadata suggestion for single imports
- **Direct Import Mode**: Extract + import in one command for trusted sources
- **GUI Integration**: Show extracted metadata with confidence scores for user review

**Error Handling**:
- Invalid EPUB structure: skip file, log error, continue with next
- Missing OPF file: attempt fallback to filename parsing
- Malformed XML: use regex fallback for critical fields (title, author)
- Invalid metadata: include in output with low confidence scores

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
│   ├── books/            # Book files organized by SHA256 hash
│   │   ├── {hash[0:2]}/  # First level: first 2 chars of hash
│   │   │   └── {hash[2:4]}/  # Second level: chars 3-4 of hash
│   │   │       └── {hash[4:]}.{ext}  # Filename: remaining hash + extension
│   │   # Example: d1/21/b095fd222ac6d4f13eebaba7a3d08fe35fee3189b996d6020b3365c27252.epub
│   ├── originals_opf/    # Original OPF metadata files (EPUB only)
│   │   ├── {hash[0:2]}/  # Same hierarchical structure as books/
│   │   │   └── {hash[2:4]}/
│   │   │       └── {hash[4:]}.opf.xml  # OPF extracted from EPUB
│   │   # Example: d1/21/b095fd222ac6d4f13eebaba7a3d08fe35fee3189b996d6020b3365c27252.opf.xml
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

**Hash-Based Storage**: Book files are stored in a hierarchical structure based on their SHA256 content hash. This provides:
- Content-addressed storage (same file = same location)
- Efficient distribution across 65,536 subdirectories (256×256)
- Automatic duplicate detection at the filesystem level
- Optimal performance with large collections

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

## File Storage System

**Hash-Based Content-Addressed Storage**: Ritmo uses SHA256 content hashing for file organization and duplicate detection.

### Storage Path Structure
```
storage/books/{hash[0:2]}/{hash[2:4]}/{hash[4:]}.{extension}
```

**Example**: File with SHA256 hash `d121b095fd222ac6d4f13eebaba7a3d08fe35fee3189b996d6020b3365c27252`
```
storage/books/d1/21/b095fd222ac6d4f13eebaba7a3d08fe35fee3189b996d6020b3365c27252.epub
```

### Key Features

1. **Content-Addressed**: Same file content always results in the same storage path
   - Automatic deduplication at the filesystem level
   - Identical books are stored only once

2. **Hierarchical Distribution**:
   - First level: 256 directories (00-ff, first 2 hex chars)
   - Second level: 256 subdirectories per first level (256×256 = 65,536 total)
   - With 1 million books: ~15 files per directory (optimal filesystem performance)

3. **Duplicate Detection**: SHA256 hash stored in `books.file_hash` (UNIQUE constraint)
   - Hash calculated from file content during import
   - Database query prevents duplicate imports before file copy
   - No collision risk (SHA256 collision probability: ~1 in 2^256)

4. **Database Integration**:
   - `books.file_link`: Relative path from storage root (e.g., `"books/d1/21/b095fd222...epub"`)
   - `books.file_hash`: Full SHA256 hash (64 hex characters)
   - `books.file_size`: File size in bytes

### Import Workflow

1. Read file content and calculate SHA256 hash
2. Check database for existing `file_hash` (duplicate detection)
3. Generate hierarchical path from hash
4. Create subdirectories if needed (`mkdir -p`)
5. Copy file to storage location
6. Save book record with `file_link` and `file_hash`

### Benefits

- **Performance**: O(1) lookup with known hash, efficient with millions of files
- **Portability**: Relative paths allow library relocation
- **Integrity**: Content hash verifies file integrity
- **Deduplication**: Same content = same location (no duplicates)
- **Scalability**: Excellent distribution prevents filesystem bottlenecks
- **Backup-Friendly**: Stable paths (hash-based, not metadata-based)

### Implementation

**Location**: `ritmo_core/src/service/book_import_service.rs`

```rust
// Hash calculation
let file_content = fs::read(file_path)?;
let file_hash = calculate_hash(&file_content);  // SHA256 -> 64 hex chars

// Path generation
let extension = file_path.extension().unwrap_or("epub");
let relative_path = format!(
    "books/{}/{}/{}.{}",
    &file_hash[0..2],   // First level
    &file_hash[2..4],   // Second level
    &file_hash[4..],    // Filename
    extension
);
```

### OPF Metadata Preservation

**Feature**: Automatic extraction and storage of original EPUB OPF (Open Packaging Format) metadata files.

**Purpose**: The OPF file is the metadata container in EPUB files, containing:
- Book metadata (title, author, publisher, ISBN, language, etc.)
- File structure and manifest
- Table of contents (spine)
- Cover references

**Storage Location**: `storage/originals_opf/` with the same hash-based hierarchy as `books/`:
```
storage/originals_opf/{hash[0:2]}/{hash[2:4]}/{hash[4:]}.opf.xml
```

**Example**:
- Book: `storage/books/d1/21/b095fd222...epub`
- OPF: `storage/originals_opf/d1/21/b095fd222...opf.xml`

**Extraction Process**:
1. Open EPUB as ZIP archive
2. Read `META-INF/container.xml` to find OPF path
3. Extract OPF file from ZIP
4. Save to `originals_opf/` with hash-based path

**Implementation**: `ritmo_core/src/epub_utils.rs` - `extract_opf()` function

**Use Cases**:
- **Metadata Analysis**: Examine original metadata for debugging
- **Level 3 Import**: Future automatic metadata extraction can use these files
- **Validation**: Compare imported metadata with original OPF
- **Bulk Updates**: Re-extract metadata from preserved OPFs without original files
- **ML Training**: Use OPF corpus for training metadata extraction models

**Error Handling**: OPF extraction failures don't block import (graceful degradation)
- Non-standard EPUB structures continue importing
- Only EPUB files attempt OPF extraction (PDFs, MOBI skip)

**Example OPF Content** (24KB typical size):
```xml
<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0">
  <metadata>
    <dc:creator>Julie Adair King</dc:creator>
    <dc:title>Fotografia digitale For Dummies</dc:title>
    <dc:publisher>Hoepli</dc:publisher>
    <dc:identifier opf:scheme="ISBN">9788820360108</dc:identifier>
    <dc:language>it</dc:language>
    <dc:date>2014-04-06</dc:date>
  </metadata>
  <!-- ... manifest, spine, guide ... -->
</package>
```

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
