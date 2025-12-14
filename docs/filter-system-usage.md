# Filter System Usage Guide

## Overview

The filter system allows you to query and list books and contents from your Ritmo library with powerful filtering, sorting, and output formatting options.

## Commands

### List Books

```bash
# List all books (table format - default)
ritmo list-books

# Filter by author
ritmo list-books --author "Calvino"

# Filter by publisher
ritmo list-books --publisher "Einaudi"

# Filter by format
ritmo list-books --format "epub"

# Filter by year
ritmo list-books --year 2020

# Filter by ISBN
ritmo list-books --isbn "978-88"

# Full-text search (searches in title, original title, and notes)
ritmo list-books --search "barone"

# Multiple filters
ritmo list-books --author "Calvino" --format "epub" --year 2020

# Sort options: title (default), author, year, date_added
ritmo list-books --sort author
ritmo list-books --sort year
ritmo list-books --sort date_added

# Pagination
ritmo list-books --limit 10
ritmo list-books --limit 10 --offset 20

# Output formats
ritmo list-books --output table    # Default: formatted table
ritmo list-books --output json     # JSON format for scripting
ritmo list-books --output simple   # Simple bullet list

# Short form for output
ritmo list-books -o json
```

### List Contents

```bash
# List all contents
ritmo list-contents

# Filter by author
ritmo list-contents --author "Calvino"

# Filter by content type (Romanzo, Racconto, Saggio, etc.)
ritmo list-contents --content-type "Romanzo"

# Filter by year
ritmo list-contents --year 1957

# Full-text search
ritmo list-contents --search "cavaliere"

# Multiple filters
ritmo list-contents --author "Calvino" --content-type "Romanzo"

# Sort options: title (default), author, year, type
ritmo list-contents --sort type
ritmo list-contents --sort year

# Pagination
ritmo list-contents --limit 20
ritmo list-contents --limit 20 --offset 40

# Output formats (same as list-books)
ritmo list-contents -o json
ritmo list-contents -o simple
```

## Output Formats

### Table Format (Default)

Displays results in a formatted table with columns:
- **Books**: ID, Title, Publisher, Format, Year
- **Contents**: ID, Title, Type, Year

```
ID    Titolo                                   Editore              Formato         Anno
-----------------------------------------------------------------------------------------------
1     Il barone rampante                       Einaudi              EPUB            1957
2     Il cavaliere inesistente                 Mondadori            PDF             1959

Totale: 2 libri
```

### JSON Format

Machine-readable JSON array for scripting and integration:

```json
[
  {
    "id": 1,
    "name": "Il barone rampante",
    "original_title": "Il barone rampante",
    "publisher_name": "Einaudi",
    "format_name": "EPUB",
    "series_name": "I nostri antenati",
    "series_index": 2,
    "publication_date": 1262304000,
    "isbn": "978-88-06-20000-0",
    "pages": 320,
    "file_link": "/path/to/book.epub",
    "created_at": 1609459200
  }
]
```

### Simple Format

Quick reading format with bullet points:

```
• Il barone rampante (Einaudi) [1957] - EPUB
• Il cavaliere inesistente (Mondadori) [1959] - PDF

Totale: 2 libri
```

## Practical Examples

### Find all EPUBs by an author
```bash
ritmo list-books --author "Calvino" --format "epub"
```

### Export library catalog to JSON
```bash
ritmo list-books -o json > my_library.json
```

### Search for books with specific term
```bash
ritmo list-books --search "cavaliere"
```

### List recent additions (last 10)
```bash
ritmo list-books --sort date_added --limit 10
```

### Paginate through large collection
```bash
# First page
ritmo list-books --limit 20

# Second page
ritmo list-books --limit 20 --offset 20

# Third page
ritmo list-books --limit 20 --offset 40
```

### Get all science fiction books from a specific year
```bash
ritmo list-books --search "fantascienza" --year 2020
```

### List all novels (contents)
```bash
ritmo list-contents --content-type "Romanzo"
```

## Using with Different Libraries

All commands support the `--library` flag to specify a library:

```bash
# Use specific library
ritmo list-books --library /path/to/library

# Works with filters
ritmo list-books --library /media/usb/MyLibrary --author "Calvino"
```

## Technical Details

### Filter Architecture

- **Location**: `ritmo_db_core/src/filters.rs` and `query_builder.rs`
- **Result Types**: `BookResult` and `ContentResult` in `ritmo_db_core/src/results.rs`
- **Query Builder**: Constructs parameterized SQL queries to prevent injection
- **Formatter**: `ritmo_cli/src/formatter.rs` handles output formatting

### Performance

- Uses SQLx connection pooling for efficient database access
- Parameterized queries for security
- Supports pagination for large datasets
- All filters applied at database level (no post-processing)

### Testing

- 8 unit tests in `ritmo_db_core` (filters, query builder, results)
- 2 unit tests in `ritmo_cli` (formatter)
- End-to-end integration tested
- All output formats validated

## Future Enhancements

### Filter Presets (Planned)

Save and reuse filter combinations:

```bash
# Save a filter preset
ritmo save-preset books --name "my_ebooks" --format epub --sort date_added

# Use a preset
ritmo list-books --preset my_ebooks

# Library-specific presets (portable!)
ritmo save-preset books --name "default_view" --format epub --library
```

See CLAUDE.md "Filter Preset System" section for complete architecture.
