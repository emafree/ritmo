# Book Metadata JSON Format

This document describes the JSON format used for book metadata in ritmo's batch import system (Level 2) and metadata extraction output (Level 3).

## Overview

The format is a JSON array of import objects, where each object represents a physical book file and its associated contents. This structure reflects ritmo's database architecture:
- **Book**: Physical book file with metadata (publisher, ISBN, format, etc.)
- **Contents**: Literary works contained in the book (novels, stories, essays, etc.)
- **People**: Contributors associated with either books (editors, preface writers) or contents (authors, translators)

This format is used by:
- **Level 2** (batch import): **✅ IMPLEMENTED** - Read this format and import multiple books with their contents
- **Level 3** (ebook_parser): **Planned** - Extract metadata from EPUBs and output to this format

**Workflow**: Extract (Level 3) → Review/Edit → Import (Level 2)

**Implementation Status**:
- Level 2 batch import is fully functional with validation, error handling, and comprehensive testing
- Level 3 ebook_parser extraction is planned for future implementation

## Format Specification

### Root Structure
```json
[
  { /* import object 1 */ },
  { /* import object 2 */ },
  ...
]
```

### Import Object

Each import object represents one physical book file to import.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `file_path` | string | ✅ Yes | Absolute or relative path to book file |
| `book` | object | ✅ Yes | Book-level metadata (see Book Object below) |
| `contents` | array | ❌ No | Array of content objects (see Content Object below) |
| `confidence` | object | ❌ No | Confidence scores for extracted fields (see Confidence Object below) |

### Book Object

Represents the physical book (the file being imported).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | ✅ Yes | Book title (physical edition title) |
| `original_title` | string | ❌ No | Original title if different from current title |
| `people` | array | ❌ No | Array of people objects for book-level contributors (editors, preface writers, etc.) |
| `publisher` | string | ❌ No | Publisher name |
| `year` | integer | ❌ No | Publication year of this edition |
| `isbn` | string | ❌ No | ISBN identifier |
| `format` | string | ❌ No | File format (auto-detected if omitted) |
| `series` | string | ❌ No | Series name |
| `series_index` | integer | ❌ No | Position in series |
| `pages` | integer | ❌ No | Page count |
| `notes` | string | ❌ No | Free-text notes |
| `tags` | array | ❌ No | Array of tag strings |

### Content Object

Represents a literary work contained in the book (novel, story, essay, etc.). A book can contain multiple contents (e.g., a collection, omnibus edition).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | ✅ Yes | Content title (work title) |
| `original_title` | string | ❌ No | Original title if different |
| `people` | array | ❌ No | Array of people objects for content creators (authors, translators, etc.) |
| `type` | string | ❌ No | Content type (i18n key, e.g., "type.novel", "type.short_story") |
| `year` | integer | ❌ No | Original publication year of this work |
| `languages` | array | ❌ No | Array of language objects (see below) |

### People Object

Used for both book-level and content-level contributors.

```json
{
  "name": "Person Name",
  "role": "role.author"
}
```

**Role Values** (i18n keys):
- `role.author` - Author (typically content-level)
- `role.translator` - Translator (typically content-level)
- `role.editor` - Editor (typically book-level)
- `role.illustrator` - Illustrator (can be both levels)
- `role.narrator` - Narrator (typically content-level)
- `role.introduction` - Introduction writer (typically book-level)
- `role.preface` - Preface writer (typically book-level)
- `role.afterword` - Afterword writer (can be both levels)

**Usage Guidelines**:
- **Book-level people**: Contributors to the physical edition (editors, edition preface/introduction writers)
- **Content-level people**: Creators of the literary work (authors, translators, original work contributors)

### Language Object
```json
{
  "code": "en",
  "role": "language_role.original"
}
```

**Language Codes**: ISO 639-1 codes (e.g., "en", "it", "fr", "de", "es")

**Role Values** (i18n keys):
- `language_role.original` - Original language of the content
- `language_role.source` - Source language for translation
- `language_role.actual` - Actual language of this book/edition

### Confidence Object (Level 3 Output Only)

The confidence object contains scores for extracted fields, using dot notation for nested fields and array indexing for contents.

```json
{
  "book.title": 0.95,
  "book.publisher": 0.85,
  "book.year": 0.90,
  "book.isbn": 0.95,
  "book.series": 0.85,
  "book.people": 0.75,
  "contents[0].title": 0.95,
  "contents[0].people": 0.90,
  "contents[1].title": 0.92,
  "contents[1].people": 0.88
}
```

**Confidence Score Ranges** (0.0 to 1.0):
- **0.90-1.0**: High confidence (found in proper metadata fields)
- **0.70-0.89**: Medium confidence (inferred or parsed with some uncertainty)
- **0.50-0.69**: Low confidence (extracted from filename or estimated)

**Confidence Keys**:
- `book.*` - Book-level fields
- `contents[N].*` - Content-level fields (N = array index)

The `confidence` object is produced by Level 3 (ebook_parser) and ignored during Level 2 import. It's included for user review and filtering purposes.

## Example Files

See [book_metadata_format.json](book_metadata_format.json) for a complete example with two import objects:
1. A collected edition with 3 contents (2 novels + 1 short story collection)
2. A single-content book (history essay)

## Usage Examples

### Level 3: Extract Metadata (Planned)
```bash
# Extract metadata from EPUBs to JSON file (PLANNED)
ritmo extract-metadata ~/books/*.epub --output metadata.json

# Extract and filter by confidence threshold (PLANNED)
ritmo extract-metadata ~/books/*.epub --min-confidence 0.80 --output metadata.json

# Extract to stdout (for piping) (PLANNED)
ritmo extract-metadata ~/books/*.epub
```

### Level 2: Batch Import (✅ Implemented)
```bash
# Import from JSON file
ritmo add-batch --input metadata.json

# Import from stdin
cat metadata.json | ritmo add-batch

# Dry run (validate without importing)
ritmo add-batch --input metadata.json --dry-run

# Continue on errors
ritmo add-batch --input metadata.json --continue-on-error
```

### Integrated Workflow
```bash
# Step 1: Extract metadata (Level 3 - PLANNED)
ritmo extract-metadata ~/books/*.epub --output metadata.json

# Step 2: Review and edit metadata.json
# - Check low-confidence fields
# - Fix incorrect extractions
# - Add missing metadata
# - Remove unwanted books

# Step 3: Batch import (Level 2 - ✅ IMPLEMENTED)
ritmo add-batch --input metadata.json
ritmo add-batch --input metadata.json --dry-run           # Validation only
ritmo add-batch --input metadata.json --continue-on-error # Continue on errors
cat metadata.json | ritmo add-batch                       # Via stdin
```

**Current Status**: Level 2 is fully implemented and tested. You can manually create JSON files following this format and use `ritmo add-batch` to import them. Level 3 (automatic extraction) will be implemented in the future.

## Field Validation

During Level 2 import, the following validations are performed:

### Import Object Level
1. **file_path**: Must exist and be readable
2. **book**: Must be present (required object)
3. **book.title**: Must be non-empty string

### Book Object Level
4. **book.year**: If provided, must be valid integer (1000-2100)
5. **book.series_index**: If provided, must be positive integer
6. **book.pages**: If provided, must be positive integer
7. **book.isbn**: If provided, basic format validation (10 or 13 digits)
8. **book.people[].name**: Must be non-empty string
9. **book.people[].role**: Must be valid i18n key (starts with "role.")

### Content Object Level
10. **contents[].title**: Must be non-empty string (if content is present)
11. **contents[].year**: If provided, must be valid integer (1000-2100)
12. **contents[].type**: If provided, must be valid i18n key (starts with "type.")
13. **contents[].people[].name**: Must be non-empty string
14. **contents[].people[].role**: Must be valid i18n key (starts with "role.")
15. **contents[].languages[].role**: Must be valid i18n key (starts with "language_role.")
16. **contents[].languages[].code**: Must be valid ISO 639-1 code (2 letters)

## Error Handling

### Level 3 (Extract)
- **Invalid EPUB**: Skip file, log error, continue with next
- **Missing metadata**: Output with low confidence scores
- **Parse errors**: Use fallback extraction (filename, regex)

### Level 2 (Import)
- **Invalid JSON**: Abort with error message
- **Missing required fields**: Skip book, log error
- **Duplicate detection**: Skip books with existing SHA256 hash
- **File not found**: Skip book, log error
- **Validation errors**: Skip book, log detailed error

Error handling modes:
- `--stop-on-error`: Abort on first failure (default)
- `--continue-on-error`: Skip failed books, report summary at end

## Future Enhancements

1. **Multiple formats**: Support YAML or TOML in addition to JSON
2. **Partial updates**: Support updating existing books instead of only creating new ones
3. **Merge strategies**: Handle conflicts when book already exists
4. **Validation schemas**: JSON Schema for format validation
5. **Extended metadata**: Support for more fields (editions, awards, ratings, etc.)
