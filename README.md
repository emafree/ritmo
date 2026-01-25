# ritmo

A Rust-based library management system inspired by Calibre, focused solely on cataloging books and their metadata.

## Overview

**ritmo** is designed to catalog books, their contents, and contributors (authors, translators, illustrators, editors) without the complexity of editing, reading, or converting ebooks. It's a pure library management tool that does one thing well: organize your book collection.

### Key Features

- **Multi-library support** with global configuration
- **Portable mode** for USB/external drive usage
- **Complete CRUD operations** for books and metadata
- **Advanced filter system** with OR logic and presets
- **ML-powered deduplication** for cleaning up duplicate entities
- **SQLite database** - no external server required
- **CLI and GUI interfaces** - choose your preferred workflow

## Quick Start

```bash
# Build the entire workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Initialize a new library
cargo run -p ritmo_cli -- init

# Import a book
cargo run -p ritmo_cli -- add book.epub --title "Book Title" --author "Author Name"

# List books with filters
cargo run -p ritmo_cli -- list-books --author "King" --format epub
```

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[Architecture](docs/architecture.md)** - Workspace crates, database schema, directory structure
- **[Development Guide](docs/development.md)** - Build, test, and run commands
- **[Filter System](docs/filters.md)** - Filter types, presets, and usage
- **[ML System](docs/ml-system.md)** - Entity deduplication documentation
- **[Session History](docs/sessions/)** - Development changelog

## Features

### Library Management
- Multi-library support with global configuration (`~/.config/ritmo/settings.toml`)
- Portable mode: auto-detect when running from `bootstrap/portable_app/`
- Library initialization with directory structure and template database

### Book Management (CRUD Complete)
- **Create**: Import books with manual metadata, SHA256 hash for duplicate detection
- **Read**: List and filter books with comprehensive query system
- **Update**: Modify book metadata with optional field updates
- **Delete**: Remove books with optional file deletion and cleanup of orphaned entities

### Filter System
- Multiple filter types: author, publisher, series, format, year, ISBN, dates
- OR logic for multiple values within same filter type
- Preset system: global and library-specific filter presets
- Three output formats: table, JSON, simple
- Relative date filters: `--last-days`, `--last-months`, `--recent-count`

### ML Deduplication
- Pattern classification system (7 pattern types)
- Jaro-Winkler similarity clustering
- Safe database merging with transactions
- Configurable confidence thresholds
- Dry-run mode for preview

## Architecture

The project is organized as a Rust workspace with specialized crates:

- **ritmo_core** - Core logic and ebook management
- **ritmo_cli** - Command-line interface
- **ritmo_db** - Database models and schema
- **ritmo_db_core** - Database operations and metadata management
- **ritmo_ml** - Machine learning for entity deduplication
- **ritmo_config** - Configuration management
- **ritmo_mapping** - Metadata mapping
- **ritmo_errors** - Shared error types
- **ritmo_gui** - Graphical interface (Slint-based)
- **ebook_parser** - EPUB metadata extraction

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

# Delete book
cargo run -p ritmo_cli -- delete-book 1 --delete-file

# Cleanup orphaned entities
cargo run -p ritmo_cli -- cleanup --dry-run
```

### ML Deduplication Operations
```bash
# Find duplicate authors (dry-run mode)
cargo run -p ritmo_cli -- deduplicate-authors --dry-run

# Merge duplicate authors with auto-merge
cargo run -p ritmo_cli -- deduplicate-authors --threshold 0.90 --auto-merge

# Find duplicate publishers
cargo run -p ritmo_cli -- deduplicate-publishers --dry-run

# Find duplicate series
cargo run -p ritmo_cli -- deduplicate-series --dry-run

# Find duplicate tags
cargo run -p ritmo_cli -- deduplicate-tags --dry-run

# Run deduplication for all entity types (authors, publishers, series, tags)
cargo run -p ritmo_cli -- deduplicate-all --threshold 0.85 --dry-run
```

## Development

### Requirements
- Rust **stable** (1.91+) as specified in `rust-toolchain.toml`
- Edition 2024 features available

### Building and Testing

```bash
# Build entire workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Format code
cargo fmt --all

# Run linter
cargo clippy --all -- -D warnings

# Run CLI
cargo run -p ritmo_cli -- [command]

# Run GUI
cargo run -p ritmo_gui
```

For detailed development instructions, see [Development Guide](docs/development.md).

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Roadmap

### High Priority
1. **Portable Bootstrap**: Automatic binary copying to bootstrap/portable_app/
2. **ebook_parser Integration**: Extract EPUB metadata automatically (goal: 95% automation)

### Medium Priority
3. **Advanced Filters**: SQL-like query DSL for complex queries
4. **Preset System Phase 3**: Auto-save last filter, interactive editing
5. **Documentation**: Comprehensive user documentation

### Low Priority
6. **GUI Integration**: Update `ritmo_gui` to use `ritmo_config`
7. **Integrate ebook_parser**: Extract EPUB metadata automatically (goal: 95% automation)

## License

[License information to be added]

## Acknowledgments

Inspired by **Calibre** - the comprehensive ebook management solution.
