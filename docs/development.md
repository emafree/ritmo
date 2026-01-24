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
```

#### Book Management
```bash
# Import a book into the library
cargo run -p ritmo_cli -- add /path/to/book.epub --title "Book Title" --author "Author Name"

# Import with full metadata
cargo run -p ritmo_cli -- add book.pdf \
  --title "My Book" \
  --author "John Doe" \
  --publisher "Publisher" \
  --year 2024 \
  --isbn "978-1234567890" \
  --series "Series Name" \
  --series-index 1 \
  --notes "Some notes"

# Update book metadata
cargo run -p ritmo_cli -- update-book 1 --title "New Title" --year 2024
cargo run -p ritmo_cli -- update-book 1 --author "New Author" --notes "Updated notes"

# Delete book
cargo run -p ritmo_cli -- delete-book 1                   # Delete record only
cargo run -p ritmo_cli -- delete-book 1 --delete-file     # Delete record + file
cargo run -p ritmo_cli -- delete-book 1 --delete-file --force  # Force deletion
```

#### Listing and Filtering
```bash
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
```

For more filter options, see [Filter System Documentation](filters.md).

#### Content Management
```bash
# Update content metadata
cargo run -p ritmo_cli -- update-content 1 --title "New Title" --author "New Author"
cargo run -p ritmo_cli -- update-content 1 --content-type "Romanzo" --year 2024

# Delete content
cargo run -p ritmo_cli -- delete-content 1
```

#### Cleanup
```bash
# Cleanup orphaned entities (authors, publishers, series not referenced)
cargo run -p ritmo_cli -- cleanup
cargo run -p ritmo_cli -- cleanup --dry-run               # Preview without changes
```

#### Help
```bash
# Show help
cargo run -p ritmo_cli -- --help
cargo run -p ritmo_cli -- add --help
cargo run -p ritmo_cli -- update-book --help
```

### GUI Application
```bash
# Run GUI application
cargo run -p ritmo_gui

# Build GUI in release mode (smaller and faster)
cargo build -p ritmo_gui --release
./target/release/ritmo_gui
```
