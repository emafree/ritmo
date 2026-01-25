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
- **Delete**: Remove books with optional file deletion and cleanup of orphaned entities

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

# Delete book
cargo run -p ritmo_cli -- delete-book 1 --delete-file

# Cleanup orphaned entities
cargo run -p ritmo_cli -- cleanup --dry-run
```

### ML Deduplication Operations
```bash
# Find duplicate authors (dry-run by default)
cargo run -p ritmo_cli -- deduplicate-authors --dry-run

# Merge duplicate authors with custom threshold
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

For complete command reference, see [Development Guide](docs/development.md).

## Rust Version

Required: **stable** (currently 1.91+) as specified in `rust-toolchain.toml`
- Edition 2024 features available
- Supports Slint GUI framework

## Recent Changes

### 2026-01-25 - Session 12: ML CLI Integration - COMPLETED
Integrated ritmo_ml deduplication system into CLI with 4 new commands.
- `deduplicate-authors` - Find and merge duplicate authors using ML
- `deduplicate-publishers` - Find and merge duplicate publishers
- `deduplicate-series` - Find and merge duplicate series
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
2. **ebook_parser Integration**: Extract EPUB metadata automatically (goal: 95% automation)

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
