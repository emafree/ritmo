# Command Layer Architecture

**Status**: ✅ Implemented
**Version**: 1.0.0
**Date**: 2026-02-10

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Design Principles](#design-principles)
- [Command Trait](#command-trait)
- [Available Commands](#available-commands)
- [Usage Examples](#usage-examples)
- [Adding New Commands](#adding-new-commands)
- [Testing](#testing)
- [Migration Guide](#migration-guide)

---

## Overview

The command layer (`ritmo_commands` crate) provides a clean separation between business logic and presentation layers. It enables code sharing between CLI and GUI frontends by encapsulating operations as stateless, testable commands.

### Why Command Layer?

**Before** (Direct service calls):
```rust
// CLI tightly coupled to services
let book_id = import_book(&config, &pool, &file, metadata).await?;
println!("Book added: {}", book_id);

// GUI would duplicate this logic
```

**After** (Command pattern):
```rust
// CLI uses command
let command = AddBookCommand;
let result = command.execute(&config, &pool, input).await?;
println!("Book added: {}", result.book_id);

// GUI reuses same command with different UI
let result = command.execute(&config, &pool, input).await?;
show_success_dialog(&result);
```

### Benefits

1. **Code Reuse**: Same commands for CLI, GUI, API
2. **Testability**: Commands testable without UI
3. **Type Safety**: Structured inputs/outputs with Serde
4. **Separation**: Presentation ↔ Logic ↔ Services
5. **Validation**: Centralized in command layer
6. **Maintainability**: Single source of truth for operations

---

## Architecture

### Layer Structure

```
┌─────────────────────────────────────────────┐
│  Presentation Layer (CLI / GUI / API)       │
│  ─────────────────────────────────────────  │
│  • Parse user input                         │
│  • Display results                          │
│  • Handle UI interactions                   │
│  • No business logic                        │
└────────────────┬────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────┐
│  Command Layer (ritmo_commands)             │
│  ─────────────────────────────────────────  │
│  • Command trait                            │
│  • Typed Input/Output structs              │
│  • Input validation                         │
│  • Error handling                           │
│  • Stateless operations                     │
└────────────────┬────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────┐
│  Service Layer (ritmo_core)                 │
│  ─────────────────────────────────────────  │
│  • Business logic implementation            │
│  • Database operations                      │
│  • File system operations                   │
│  • Domain logic                             │
└─────────────────────────────────────────────┘
```

### Crate Organization

```
ritmo_commands/
├── src/
│   ├── lib.rs              # Command trait, error types
│   ├── types/mod.rs        # Output types (results, summaries)
│   ├── books/              # Book commands
│   │   ├── mod.rs
│   │   ├── add.rs          # AddBookCommand
│   │   ├── list.rs         # ListBooksCommand
│   │   ├── update.rs       # UpdateBookCommand
│   │   └── delete.rs       # DeleteBookCommand
│   ├── contents/           # Content commands
│   │   ├── mod.rs
│   │   ├── add.rs          # AddContentCommand
│   │   ├── list.rs         # ListContentsCommand
│   │   ├── update.rs       # UpdateContentCommand
│   │   ├── delete.rs       # DeleteContentCommand
│   │   ├── link.rs         # LinkContentCommand
│   │   └── unlink.rs       # UnlinkContentCommand
│   └── entities/           # Entity list commands
│       ├── mod.rs
│       ├── list_tags.rs    # ListTagsCommand
│       ├── list_publishers.rs  # ListPublishersCommand
│       ├── list_series.rs  # ListSeriesCommand
│       └── list_people.rs  # ListPeopleCommand
└── Cargo.toml
```

---

## Design Principles

### 1. Stateless Commands

Commands have no mutable state. All data flows through input/output.

```rust
// ✅ Good: Stateless
#[derive(Debug, Clone)]
pub struct AddBookCommand;

// ❌ Bad: Stateful
pub struct AddBookCommand {
    cache: HashMap<String, i64>,  // Don't do this!
}
```

### 2. Typed Inputs and Outputs

Every command has strongly-typed Input and Output structs.

```rust
pub struct AddBookInput {
    pub file_path: PathBuf,
    pub title: String,
    pub publisher: Option<String>,
    // ...
}

pub struct AddBookResult {
    pub book_id: i64,
    pub title: String,
    pub file_size: u64,
}
```

### 3. Validation Before Execution

Commands validate input before calling services.

```rust
fn validate(&self, input: &Self::Input) -> CommandResult<()> {
    if input.title.trim().is_empty() {
        return Err(CommandError::Validation(
            "Title cannot be empty".to_string()
        ));
    }
    Ok(())
}
```

### 4. Single Responsibility

Each command does one thing. Bulk operations handled at CLI/GUI level.

```rust
// ✅ Good: Single book operation
UpdateBookCommand::execute(&config, &pool, input).await?;

// CLI/GUI iterates for bulk:
for book in books {
    let input = UpdateBookInput { book_id: book.id, ... };
    command.execute(&config, &pool, input).await?;
}
```

### 5. Serializable Results

All outputs implement `Serialize` for future API/IPC use.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddBookResult {
    pub book_id: i64,
    pub title: String,
}
```

---

## Command Trait

### Definition

```rust
#[async_trait]
pub trait Command: Send + Sync + Debug {
    /// Input parameters for this command
    type Input: Send + Sync;

    /// Output result of this command
    type Output: Send + Sync;

    /// Execute the command
    async fn execute(
        &self,
        config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> Result<Self::Output, CommandError>;

    /// Validate input before execution (optional)
    fn validate(&self, _input: &Self::Input) -> Result<(), CommandError> {
        Ok(())
    }
}
```

### Error Types

```rust
pub enum CommandError {
    Database(sqlx::Error),
    Ritmo(ritmo_errors::RitmoErr),
    Validation(String),
    NotFound(String),
    Io(std::io::Error),
    Other(String),
}
```

---

## Available Commands

### Books (4 commands)

| Command | Input | Output | Purpose |
|---------|-------|--------|---------|
| `AddBookCommand` | File path + metadata | `book_id` + details | Import book with manual metadata |
| `ListBooksCommand` | Filters | `Vec<BookSummary>` | Query books with filters |
| `UpdateBookCommand` | `book_id` + updates | `book_id` | Update single book metadata |
| `DeleteBookCommand` | `book_id` + flags | `book_id` + status | Delete book (DB + optional file) |

### Contents (6 commands)

| Command | Input | Output | Purpose |
|---------|-------|--------|---------|
| `AddContentCommand` | Title + metadata | `content_id` + details | Create new content |
| `ListContentsCommand` | Filters | `Vec<ContentSummary>` | Query contents with filters |
| `UpdateContentCommand` | `content_id` + updates | `content_id` | Update single content metadata |
| `DeleteContentCommand` | `content_id` | `content_id` | Delete content record |
| `LinkContentCommand` | `content_id` + `book_id` | Both IDs | Associate content to book |
| `UnlinkContentCommand` | `content_id` + `book_id` | Both IDs | Remove content-book link |

### Entities (4 commands)

| Command | Input | Output | Purpose |
|---------|-------|--------|---------|
| `ListTagsCommand` | None | `Vec<TagSummary>` | List all tags |
| `ListPublishersCommand` | None | `Vec<PublisherSummary>` | List all publishers |
| `ListSeriesCommand` | None | `Vec<SeriesSummary>` | List all series |
| `ListPeopleCommand` | None | `Vec<PersonSummary>` | List all people (authors, translators, etc.) |

---

## Usage Examples

### CLI Example

```rust
use ritmo_commands::{Command, books::{AddBookCommand, AddBookInput}};

pub async fn handle_books_add(
    config: &LibraryConfig,
    pool: &SqlitePool,
    file: PathBuf,
    title: String,
    // ... other args
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Prepare input
    let input = AddBookInput {
        file_path: file,
        title,
        // ...
    };

    // 2. Execute command
    let command = AddBookCommand;
    let result = command.execute(config, pool, input).await?;

    // 3. Display result (presentation layer)
    println!("✓ Book added! ID: {}", result.book_id);
    println!("  Title: {}", result.title);
    println!("  Size: {} bytes", result.file_size);

    Ok(())
}
```

### GUI Example (Hypothetical)

```rust
use ritmo_commands::{Command, books::{ListBooksCommand, ListBooksInput}};

impl BooksPanel {
    async fn load_books(&mut self) {
        // 1. Prepare input from UI state
        let input = ListBooksInput {
            filters: self.get_filters_from_ui(),
        };

        // 2. Execute command (same as CLI!)
        let command = ListBooksCommand;
        match command.execute(&self.config, &self.pool, input).await {
            Ok(result) => {
                // 3. Update UI with results
                self.display_books(result.books);
                self.status_bar.set_text(
                    format!("Found {} books", result.total_count)
                );
            }
            Err(e) => {
                self.show_error_dialog(&e.to_string());
            }
        }
    }
}
```

### Bulk Operations Pattern

Commands operate on single items. CLI/GUI handle bulk iteration.

```rust
// CLI bulk update example
pub async fn handle_books_update_bulk(
    config: &LibraryConfig,
    pool: &SqlitePool,
    book_ids: Vec<i64>,
    publisher: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let command = UpdateBookCommand;
    let mut success = 0;
    let mut errors = 0;

    for book_id in book_ids {
        let input = UpdateBookInput {
            book_id,
            publisher: publisher.clone(),
            ..Default::default()
        };

        match command.execute(config, pool, input).await {
            Ok(_) => {
                success += 1;
                println!("✓ Updated book [{}]", book_id);
            }
            Err(e) => {
                errors += 1;
                eprintln!("✗ Failed [{}]: {}", book_id, e);
            }
        }
    }

    println!("\nSummary: {} success, {} errors", success, errors);
    Ok(())
}
```

---

## Adding New Commands

### Step-by-Step Guide

Let's add a hypothetical `ExportBookCommand` as an example.

#### 1. Create Command File

```rust
// ritmo_commands/src/books/export.rs

use crate::{Command, CommandResult, ExportBookResult};
use async_trait::async_trait;
use ritmo_core::service::export_book;  // Assume this exists
use ritmo_db_core::LibraryConfig;
use sqlx::SqlitePool;
use std::path::PathBuf;

/// Command to export a book
#[derive(Debug, Clone)]
pub struct ExportBookCommand;

/// Input parameters for exporting a book
#[derive(Debug, Clone)]
pub struct ExportBookInput {
    pub book_id: i64,
    pub output_path: PathBuf,
    pub format: String,  // e.g., "pdf", "epub", "mobi"
}

#[async_trait]
impl Command for ExportBookCommand {
    type Input = ExportBookInput;
    type Output = ExportBookResult;

    fn validate(&self, input: &Self::Input) -> CommandResult<()> {
        // Validate output directory exists
        if let Some(parent) = input.output_path.parent() {
            if !parent.exists() {
                return Err(CommandError::Validation(
                    format!("Output directory does not exist: {}", parent.display())
                ));
            }
        }

        // Validate format
        if !["pdf", "epub", "mobi"].contains(&input.format.as_str()) {
            return Err(CommandError::Validation(
                format!("Unsupported format: {}", input.format)
            ));
        }

        Ok(())
    }

    async fn execute(
        &self,
        config: &LibraryConfig,
        pool: &SqlitePool,
        input: Self::Input,
    ) -> CommandResult<Self::Output> {
        // Validate input
        self.validate(&input)?;

        // Call service layer
        export_book(
            config,
            pool,
            input.book_id,
            &input.output_path,
            &input.format
        ).await?;

        // Return structured result
        Ok(ExportBookResult {
            book_id: input.book_id,
            output_path: input.output_path.display().to_string(),
            format: input.format,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_invalid_format() {
        let command = ExportBookCommand;
        let input = ExportBookInput {
            book_id: 1,
            output_path: PathBuf::from("/tmp/book.txt"),
            format: "invalid".to_string(),
        };

        let result = command.validate(&input);
        assert!(result.is_err());
    }
}
```

#### 2. Add Output Type

```rust
// ritmo_commands/src/types/mod.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportBookResult {
    pub book_id: i64,
    pub output_path: String,
    pub format: String,
}
```

#### 3. Export from Module

```rust
// ritmo_commands/src/books/mod.rs

mod add;
mod list;
mod update;
mod delete;
mod export;  // Add this

pub use add::{AddBookCommand, AddBookInput};
pub use list::{ListBooksCommand, ListBooksInput};
pub use update::{UpdateBookCommand, UpdateBookInput};
pub use delete::{DeleteBookCommand, DeleteBookInput};
pub use export::{ExportBookCommand, ExportBookInput};  // Add this
```

#### 4. Implement CLI Handler

```rust
// ritmo_cli/src/handlers/books.rs

pub async fn handle_books_export(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    book_id: i64,
    output: PathBuf,
    format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use super::common::get_library_and_pool;

    let (config, pool) = get_library_and_pool(cli_library, app_settings).await?;

    println!("Exporting book {} to {}...", book_id, output.display());

    let input = ExportBookInput {
        book_id,
        output_path: output,
        format,
    };

    let command = ExportBookCommand;
    match command.execute(&config, &pool, input).await {
        Ok(result) => {
            println!("✓ Book exported successfully!");
            println!("  Output: {}", result.output_path);
            println!("  Format: {}", result.format);
        }
        Err(e) => {
            println!("✗ Export failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
```

#### 5. Add CLI Argument Parsing

```rust
// ritmo_cli/src/main.rs

#[derive(Subcommand)]
enum BooksCommand {
    Add { /* ... */ },
    List { /* ... */ },
    Update { /* ... */ },
    Delete { /* ... */ },
    Export {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        output: PathBuf,
        #[arg(long, default_value = "epub")]
        format: String,
    },
}

// In main handler:
BooksCommand::Export { id, output, format } => {
    handle_books_export(&cli_library, &app_settings, id, output, format).await?
}
```

---

## Testing

### Unit Tests

Test commands without database using mocks:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_empty_title() {
        let command = AddBookCommand;
        let input = AddBookInput {
            file_path: PathBuf::from("book.epub"),
            title: "".to_string(),
            // ...
        };

        let result = command.validate(&input);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CommandError::Validation(_)
        ));
    }

    #[test]
    fn test_default_input() {
        let input = UpdateBookInput::default();
        assert_eq!(input.book_id, 0);
        assert!(input.title.is_none());
    }
}
```

### Integration Tests

Test with real database (future work):

```rust
#[tokio::test]
async fn test_add_book_integration() {
    // Setup test database
    let pool = create_test_pool().await;
    let config = test_library_config();

    // Execute command
    let input = AddBookInput {
        file_path: PathBuf::from("tests/fixtures/test.epub"),
        title: "Test Book".to_string(),
        // ...
    };

    let command = AddBookCommand;
    let result = command.execute(&config, &pool, input).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.book_id > 0);
    assert_eq!(result.title, "Test Book");
}
```

---

## Migration Guide

### From Direct Service Calls to Commands

**Before**:
```rust
// Direct service call
let book_id = import_book(&config, &pool, &file, metadata).await?;
println!("Book ID: {}", book_id);
```

**After**:
```rust
// 1. Prepare input
let input = AddBookInput {
    file_path: file,
    title: metadata.title,
    // ...
};

// 2. Execute command
let command = AddBookCommand;
let result = command.execute(&config, &pool, input).await?;

// 3. Use structured result
println!("Book ID: {}", result.book_id);
println!("Title: {}", result.title);
```

### Benefits of Migration

1. **Type Safety**: Structured outputs instead of primitive types
2. **Validation**: Centralized input validation
3. **Reusability**: Same command for CLI/GUI/API
4. **Testing**: Commands testable independently
5. **Documentation**: Input/Output types self-document
6. **Evolution**: Easy to add fields to results without breaking existing code

---

## Best Practices

### Do's ✅

1. **Keep commands stateless** - No mutable state
2. **Validate input** - Check all required fields and constraints
3. **Use structured types** - Don't return primitives
4. **Document thoroughly** - Examples in doc comments
5. **Test validation** - Unit tests for all validation rules
6. **Handle errors gracefully** - Convert service errors to CommandError
7. **Make outputs serializable** - Derive Serialize/Deserialize

### Don'ts ❌

1. **Don't add UI logic** - Commands should be presentation-agnostic
2. **Don't do bulk operations** - Commands work on single items
3. **Don't cache state** - Each execution should be independent
4. **Don't skip validation** - Always validate before calling services
5. **Don't use println** - Return structured results instead
6. **Don't couple to CLI** - Commands should work in any frontend
7. **Don't return service types directly** - Wrap in command-specific types

---

## Performance Considerations

### Command Overhead

Commands add minimal overhead:
- Struct creation: ~100ns
- Validation: ~1-10μs
- Service call: variable (ms-seconds)

The service layer dominates execution time, so command layer overhead is negligible.

### Bulk Operations

For bulk operations, iterate at the presentation layer:

```rust
// Good: Iterate in CLI
for book_id in book_ids {
    command.execute(&config, &pool, input).await?;
}

// Bad: Don't create bulk commands
// BulkUpdateBooksCommand { book_ids: Vec<i64> }  // Don't do this
```

This keeps commands simple and testable while allowing CLI/GUI to optimize (progress bars, parallelization, transactions).

---

## Future Enhancements

### Planned Features

1. **Transactions** - Multi-command transactions
2. **Middleware** - Logging, metrics, authorization
3. **Async Batching** - Efficient bulk operations
4. **Command History** - Undo/redo support
5. **Progress Reporting** - Streaming progress updates
6. **Validation Rules** - Declarative validation framework

### API/IPC Integration

Commands are already serializable, making them API-ready:

```rust
// HTTP API endpoint (future)
#[post("/books")]
async fn add_book(
    Json(input): Json<AddBookInput>,
    State(state): State<AppState>,
) -> Result<Json<AddBookResult>, ApiError> {
    let command = AddBookCommand;
    let result = command.execute(&state.config, &state.pool, input).await?;
    Ok(Json(result))
}
```

---

## Summary

The command layer provides:

- ✅ **10 commands** (4 books + 6 contents)
- ✅ **Clean separation** between presentation and logic
- ✅ **Type-safe** inputs and outputs
- ✅ **Testable** without UI dependencies
- ✅ **Reusable** across CLI, GUI, API
- ✅ **Well-documented** with examples

This architecture scales to hundreds of commands while maintaining consistency and clarity.

---

## References

- [Architecture Documentation](./architecture.md)
- [Development Guide](./development.md)
- [Session 30 History](./sessions/2026-02-sessions.md#session-30)
- [ritmo_commands source](../ritmo_commands/src/)
