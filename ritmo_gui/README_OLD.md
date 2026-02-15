# Ritmo GUI

Modern and minimalist graphical interface for Ritmo, built with [Slint](https://slint.dev/).

## Features

- **Minimalist Design**: Clean and modern interface focused on simplicity
- **Lightweight**: Native build without heavy dependencies, works completely offline
- **Cross-platform**: Works on Linux, Windows, and macOS
- **Async**: Full integration with Tokio for non-blocking database operations

## Structure

```
ritmo_gui/
├── src/
│   └── main.rs          # Entry point and application logic
├── ui/
│   └── main_window.slint # UI definition in Slint language
├── build.rs             # Build script to compile .slint files
└── Cargo.toml           # Dependencies
```

## Building

```bash
# Build in debug mode
cargo build -p ritmo_gui

# Optimized build (release)
cargo build -p ritmo_gui --release
```

## Running

```bash
# Run directly
cargo run -p ritmo_gui

# Or run the compiled binary
./target/release/ritmo_gui
```

## Interface

### Sidebar
- 📖 **Books**: Main view with list of all books
- ✍️ **Authors**: Author management (in development)
- 🏢 **Publishers**: Publisher management (in development)
- 📚 **Series**: Book series management (in development)
- ⚙️ **Settings**: Application configuration (in development)

### Main Area
- **Search bar**: Search books, authors, publishers in real-time
- **Book list**: Card view of books with title, author, publisher, year
- **Add button**: To add new books (in development)
- **Status messages**: Visual feedback for operations

## Initialization

On startup, the application:
1. Automatically creates the library directory in `~/RitmoLibrary`
2. Initializes the SQLite database if it doesn't exist
3. Creates the necessary directory structure (database, storage, config, bootstrap)
4. Loads books from the library (currently sample data)

## Technologies

- **Slint 1.7.2**: Native and performant UI framework
- **Tokio**: Asynchronous runtime for I/O operations
- **SQLx**: Asynchronous database access layer
- **Rust**: Safe and performant language

## Current Status

✅ Base interface implemented
✅ Sidebar navigation working
✅ Book search with real-time filtering
✅ Integration with LibraryConfig
✅ Automatic library initialization

🚧 In development:
- Real database integration (SQL queries)
- Dialogs to add/edit books
- Book detail view
- Author, publisher, series management
- EPUB file import
- Cover management

## Dependencies

Main dependencies are:
- `slint = "1.7.2"` - UI framework
- `slint-build = "1.7.2"` - Build script for .slint files
- `tokio` - Async runtime
- `dirs` - To find user home directory
- `ritmo_db_core` - Database and config management
- `ritmo_db` - Database models
- `ritmo_core` - Business logic

## Development Notes

### .slint Files
`.slint` files define the graphical interface using a declarative language similar to QML. During build, `slint-build` compiles these files into Rust code.

### Async/Sync Bridge
The application uses `tokio::runtime::Runtime` to handle async operations from the synchronous UI thread. Database operations are executed via `runtime.block_on()`.

### Callbacks
UI callbacks are defined in Slint and implemented in Rust:
- `initialize-library`: Initialize the library
- `refresh-books`: Reload the book list
- `search-books`: Filter books based on search text
- `add-new-book`: Open dialog to add book (TODO)
