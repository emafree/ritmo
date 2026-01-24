# Filter System Documentation

This document describes the comprehensive filter system used in ritmo for querying books and contents.

## Overview

The filter system is organized as a modular, isolated subsystem in `ritmo_db_core/src/filters/`:

```
filters/
├── mod.rs          # Public API and documentation
├── types.rs        # Filter structures and result types
├── builder.rs      # SQL query construction
├── executor.rs     # Query execution
└── validator.rs    # Input validation
```

## Filter Types

### BookFilters
- `authors: Vec<String>` - Multiple authors (OR logic)
- `publishers: Vec<String>` - Multiple publishers (OR logic)
- `formats: Vec<String>` - Multiple formats (OR logic)
- `series_list: Vec<String>` - Multiple series (OR logic)
- `year: Option<i32>` - Single year (exact match)
- `isbn: Option<String>` - ISBN search
- `search: Option<String>` - Full-text search (title, author, publisher)
- `acquired_after: Option<i64>` - Filter by acquisition date (Unix timestamp)
- `acquired_before: Option<i64>` - Filter by acquisition date (Unix timestamp)
- `sort: BookSortField` - Sort field (title, author, year, date_added)
- `limit: Option<i64>` - Result limit
- `offset: Option<i64>` - Result offset (pagination)

### ContentFilters
- `authors: Vec<String>` - Multiple authors (OR logic)
- `content_types: Vec<String>` - Multiple content types (OR logic)
- `year: Option<i32>` - Single year (exact match)
- `search: Option<String>` - Full-text search
- `sort: ContentSortField` - Sort field
- `limit: Option<i64>` - Result limit
- `offset: Option<i64>` - Result offset

## OR Logic Support

Multiple values for the same filter type use OR logic:

```rust
let filters = BookFilters::default()
    .with_author("King")
    .with_author("Tolkien")
    .with_format("epub");

// SQL: (author LIKE '%King%' OR author LIKE '%Tolkien%') AND format LIKE '%epub%'
```

Different filter types are combined with AND logic:
- `authors AND publishers AND formats AND year`

## Builder Pattern

Fluent API for constructing filters:

```rust
use ritmo_db_core::filters::BookFilters;

let filters = BookFilters::default()
    .with_author("Calvino")
    .with_format("epub")
    .with_format("pdf")
    .with_year(2024);
```

### Builder Methods

#### BookFilters
- `with_author(author: &str)` - Add author filter
- `with_publisher(publisher: &str)` - Add publisher filter
- `with_format(format: &str)` - Add format filter
- `with_series(series: &str)` - Add series filter
- `with_year(year: i32)` - Set year filter
- `with_isbn(isbn: &str)` - Set ISBN filter
- `with_search(query: &str)` - Set search query
- `with_acquired_after(timestamp: i64)` - Set acquired after date
- `with_acquired_before(timestamp: i64)` - Set acquired before date
- `with_limit(limit: i64)` - Set result limit
- `with_offset(offset: i64)` - Set result offset

#### ContentFilters
- `with_author(author: &str)` - Add author filter
- `with_content_type(content_type: &str)` - Add content type filter
- `with_year(year: i32)` - Set year filter
- `with_search(query: &str)` - Set search query
- `with_limit(limit: i64)` - Set result limit
- `with_offset(offset: i64)` - Set result offset

## Backward Compatibility

Old API methods still work:

```rust
// Old way (still works)
filters.set_author_opt(Some("King".to_string()));

// New way (preferred)
filters.with_author("King");
```

## Validation

The validator module ensures data integrity:

- Maximum 50 values per filter (performance limit)
- No negative offsets
- Positive limits only
- Valid date ranges (after < before)
- No empty filter values

```rust
use ritmo_db_core::filters::{validate_book_filters, BookFilters};

let filters = BookFilters::default().with_author("King");

// Validate before executing
validate_book_filters(&filters)?;
```

## Query Execution

Execute filters using the executor module:

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

## Preset System

Filters can be saved as presets for reuse.

### Two-Level Storage

**1. Global Presets** (`~/.config/ritmo/settings.toml`):
- User's personal filter preferences
- Not portable with library
- Used across all libraries

**2. Library Presets** (`library/config/filters.toml`):
- Library-specific filter configuration
- Portable with library (critical for portable mode!)
- Travels with library when copied/shared

### Preset Resolution Order

When executing `ritmo list-books`:
1. Explicit CLI options (`--author`, `--format`, etc.) - highest priority
2. `--preset <name>` (searches library first, then global)
3. Library default filter (from `library/config/filters.toml`)
4. Last used filter (global `~/.config/ritmo/last_filters`)
5. No filter (list all) - lowest priority

### CLI Commands

```bash
# Global presets
ritmo save-preset books --name "my_ebooks" --format epub
ritmo list-presets
ritmo delete-preset books "my_ebooks"

# Library presets (portable!)
ritmo save-preset books --name "default_view" --format epub --in-library
ritmo set-default-filter books default_view
ritmo list-presets --library-only

# Usage
ritmo list-books --preset default_view  # Searches library first, then global
ritmo list-books                        # Uses library default if set
```

### Portable Library Workflow

```bash
# Create library with useful presets
ritmo init /media/usb/SharedLibrary
ritmo save-preset books --name "epub_only" --format epub --in-library
ritmo set-default-filter books epub_only

# Copy to USB and share
# Colleague opens portable library
cd /media/usb/SharedLibrary/bootstrap/portable_app
./ritmo_gui    # Opens with "epub_only" filter already active!
./ritmo_cli list-books  # Automatically uses "epub_only" preset
```

## Implementation History

- **Session 2 (2025-12-14)**: Initial filter system implementation
- **Session 2 - Phase 1**: Global preset system
- **Session 5 (2025-12-16)**: Library-specific presets
- **Session 6 (2025-12-16)**: Relative date filters (--last-days, --last-months, --recent-count)
- **Session 7 - Phase 1 (2025-12-17)**: Modular architecture refactoring
- **Session 7 - Phase 2 (2025-12-17)**: OR logic support and validation

For detailed session history, see [Session Documentation](sessions/).
