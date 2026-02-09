# Development Guide

This document contains all the commands needed for building, testing, and running ritmo.

## Building

### Build Entire Workspace
```bash
cargo build --workspace
```

### Build Specific Crate
```bash
cargo build -p ritmo_cli
cargo build -p ritmo_core
cargo build -p ritmo_gui
```

### Build in Release Mode
```bash
cargo build --workspace --release
```

## Testing

### Run All Tests
```bash
cargo test --workspace
```

### Run Tests for Specific Crate
```bash
cargo test -p ritmo_db_core
cargo test -p ritmo_core
cargo test -p ritmo_ml
```

### Run Specific Test by Name
```bash
cargo test test_name
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Testing Notes
- Many crates use `serial_test` for tests that access shared resources (database)
- Dev dependencies include `tempfile` for temporary test directories
- Use `tokio-test` for async test utilities

## Code Quality

### Format All Code
```bash
cargo fmt --all
```

### Run Linter
```bash
cargo clippy --all -- -D warnings
```

### Check Code Without Building
```bash
cargo check --workspace
```

## Running the Application

### CLI Commands

#### Library Management
```bash
# Initialize a new library (default: ~/RitmoLibrary)
cargo run -p ritmo_cli -- libraries init
cargo run -p ritmo_cli -- libraries init /path/to/library

# Show current library info
cargo run -p ritmo_cli -- libraries info

# List all recent libraries
cargo run -p ritmo_cli -- libraries list

# Set current library
cargo run -p ritmo_cli -- libraries set /path/to/library

# Use specific library temporarily (doesn't change default)
cargo run -p ritmo_cli -- --library /path/to/library libraries info
```

#### Book Management
```bash
# Import a book into the library
cargo run -p ritmo_cli -- books add /path/to/book.epub --title "Book Title" --author "Author Name"

# Import with full metadata
cargo run -p ritmo_cli -- books add book.pdf \
  --title "My Book" \
  --author "John Doe" \
  --publisher "Publisher" \
  --year 2024 \
  --isbn "978-1234567890" \
  --series "Series Name" \
  --series-index 1 \
  --notes "Some notes"

# Update book metadata (single book by ID)
cargo run -p ritmo_cli -- books update --id 1 --set-title "New Title" --set-year 2024
cargo run -p ritmo_cli -- books update --id 1 --set-author "New Author" --set-notes "Updated notes"

# Update multiple books by filters (bulk operation with confirmation)
cargo run -p ritmo_cli -- books update --filter-author "King" --set-publisher "New Publisher"
cargo run -p ritmo_cli -- books update --filter-format epub --filter-year 2020 --set-notes "Reviewed" --yes
```

#### Book Deletion and Cleanup

Ritmo provides comprehensive deletion with automatic cascade cleanup of relationships and manual cleanup of orphaned entities.

```bash
# Delete book (database record only, keeps physical file)
cargo run -p ritmo_cli -- books delete --id 1

# Delete book AND physical file from storage
cargo run -p ritmo_cli -- books delete --id 1 --delete-file

# Force deletion even if file is missing or can't be deleted
cargo run -p ritmo_cli -- books delete --id 1 --delete-file --force

# Delete multiple books by filters (bulk operation with confirmation)
cargo run -p ritmo_cli -- books delete --filter-format epub --filter-year 2020
cargo run -p ritmo_cli -- books delete --filter-publisher "Old Pub" --delete-file --yes

# Dry-run mode: preview what would be deleted without executing
cargo run -p ritmo_cli -- books delete --filter-author "Test" --dry-run
```

**What happens when deleting a book:**

1. **Automatic CASCADE deletion** (immediate):
   - All book-content associations (`x_books_contents`)
   - All book-author/contributor associations (`x_books_people_roles`)
   - All book-tag associations (`x_books_tags`)

2. **Referenced entities are kept** (become orphaned):
   - People (authors, translators, etc.)
   - Publishers
   - Series
   - Formats
   - Tags
   - Contents

3. **Optional file deletion**:
   - With `--delete-file`: removes physical file from `storage/` directory
   - With `--force`: continues even if file doesn't exist or can't be deleted

**Cleanup orphaned entities:**

After deleting books, you can remove entities that are no longer referenced:

```bash
# Preview what would be removed (dry-run mode - NOT YET IMPLEMENTED)
cargo run -p ritmo_cli -- books cleanup --dry-run

# Remove orphaned entities
cargo run -p ritmo_cli -- books cleanup
```

> **Note**: The `--dry-run` flag is not yet fully implemented and will only display a message without showing preview.

**Entities cleaned up:**
- **People**: Not associated with any book or content
- **Publishers**: Not referenced by any book
- **Series**: Not referenced by any book
- **Formats**: Not used by any book
- **Types**: Not used by any content
- **Tags**: Not associated with any book or content

**Recommended workflow:**
```bash
# 1. Delete one or more books
cargo run -p ritmo_cli -- books delete --id 1 --delete-file
cargo run -p ritmo_cli -- books delete --id 2 --delete-file

# Or delete multiple books by filters
cargo run -p ritmo_cli -- books delete --filter-year 2020 --delete-file --yes

# 2. Preview orphaned entities
cargo run -p ritmo_cli -- books cleanup --dry-run

# 3. Remove orphaned entities if desired
cargo run -p ritmo_cli -- books cleanup
```

#### Listing and Filtering
```bash
# List books with filters
cargo run -p ritmo_cli -- books list
cargo run -p ritmo_cli -- books list --author "Stephen King"
cargo run -p ritmo_cli -- books list --format epub --year 2024

# Filter by acquisition date (absolute)
cargo run -p ritmo_cli -- books list --acquired-after 2024-01-01
cargo run -p ritmo_cli -- books list --acquired-before 2024-12-31

# Filter by acquisition date (relative)
cargo run -p ritmo_cli -- books list --last-days 7        # Last 7 days
cargo run -p ritmo_cli -- books list --last-months 1      # Last month
cargo run -p ritmo_cli -- books list --recent-count 10    # 10 most recent books
```

For more filter options, see [Filter System Documentation](filters.md).

#### Content Management
```bash
# Create new content
cargo run -p ritmo_cli -- contents add --title "Content Title" --author "Author Name"
cargo run -p ritmo_cli -- contents add --title "Short Story" --author "Author" --content-type "Racconto" --year 2024

# Create content and associate to a book
cargo run -p ritmo_cli -- contents add --title "Novel" --author "Author" --book-id 1

# Update content metadata (single content by ID)
cargo run -p ritmo_cli -- contents update --id 1 --set-title "New Title" --set-author "New Author"
cargo run -p ritmo_cli -- contents update --id 1 --set-content-type "Romanzo" --set-year 2024

# Update multiple contents by filters (bulk operation with confirmation)
cargo run -p ritmo_cli -- contents update --filter-author "King" --set-content-type "Romanzo"
cargo run -p ritmo_cli -- contents update --filter-content-type "Racconto" --set-content-type "Novella" --yes

# Delete content
cargo run -p ritmo_cli -- contents delete --id 1

# Delete multiple contents by filters (bulk operation with confirmation)
cargo run -p ritmo_cli -- contents delete --filter-content-type "Test"

# Associate content to book
cargo run -p ritmo_cli -- contents link --content-id 1 --book-id 1

# Remove content-book association
cargo run -p ritmo_cli -- contents unlink --content-id 1 --book-id 1

# List contents with filters
cargo run -p ritmo_cli -- contents list --author "King" --content-type "Romanzo"
```

#### Cleanup
```bash
# Cleanup orphaned entities (authors, publishers, series not referenced)
cargo run -p ritmo_cli -- books cleanup
cargo run -p ritmo_cli -- books cleanup --dry-run               # Preview without changes
```

#### ML Deduplication
```bash
# Find duplicate people (authors, translators, etc.) using ML (dry-run mode)
cargo run -p ritmo_cli -- deduplicate people --dry-run
cargo run -p ritmo_cli -- deduplicate people --threshold 0.90 --dry-run
cargo run -p ritmo_cli -- deduplicate people --threshold 0.90 --auto-merge  # Actually merge

# Find duplicate publishers
cargo run -p ritmo_cli -- deduplicate publishers --dry-run
cargo run -p ritmo_cli -- deduplicate publishers --threshold 0.85 --auto-merge

# Find duplicate series
cargo run -p ritmo_cli -- deduplicate series --dry-run

# Find duplicate tags
cargo run -p ritmo_cli -- deduplicate tags --dry-run
cargo run -p ritmo_cli -- deduplicate tags --threshold 0.85 --auto-merge

# Find and merge all duplicate entities (people, publishers, series, tags, roles)
cargo run -p ritmo_cli -- deduplicate all --dry-run
cargo run -p ritmo_cli -- deduplicate all --threshold 0.92 --auto-merge
```

#### Help
```bash
# Show help
cargo run -p ritmo_cli -- --help
cargo run -p ritmo_cli -- books --help
cargo run -p ritmo_cli -- books add --help
cargo run -p ritmo_cli -- books update --help
cargo run -p ritmo_cli -- contents --help
cargo run -p ritmo_cli -- deduplicate --help
```

### GUI Application
```bash
# Run GUI application
cargo run -p ritmo_gui

# Build GUI in release mode (smaller and faster)
cargo build -p ritmo_gui --release
./target/release/ritmo_gui
```
