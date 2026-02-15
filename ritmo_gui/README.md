# ritmo_gui - egui-based GUI

The GUI for ritmo built with the egui framework. Provides a contents-centric interface for managing your library.

## Architecture

### Contents-Centric Design

The GUI follows a contents-centric paradigm:
- **TAB BOOKS**: Browse books and see their contents
- **TAB CONTENTS**: Browse contents directly - same content can appear in multiple books

### Structure

```
ritmo_gui/
├── src/
│   ├── main.rs              # Entry point + eframe setup
│   ├── app.rs               # App state + main logic
│   ├── ui/
│   │   ├── main_window.rs    # Main layout + tab selector
│   │   ├── tabs/
│   │   │   ├── books_tab.rs
│   │   │   └── contents_tab.rs
│   │   ├── filters_panel.rs
│   │   ├── menu.rs
│   │   └── widgets/
│   ├── state/
│   │   ├── library_state.rs      # Manages library data
│   │   ├── books_filter_state.rs
│   │   └── contents_filter_state.rs
│   ├── config/
│   │   ├── theme.rs              # Dark/Light themes
│   │   └── settings.rs           # Settings persistence
│   └── events/
│       └── message.rs            # UI messages
```

## Features

### Filter System

3-level filtering:
1. Click filter field button (Author, Publisher, etc.)
2. Select filter value:
   - **NESSUNO**: Exclude items without this field
   - **ALMENO UNO**: Include items that have this field  
   - **Specific value**: e.g., "Tolkien", "epub"

### Settings Persistence

Settings are saved to `~/.ritmo/gui_config.toml`:
- Last active tab (Books or Contents)
- Last used filters for each tab
- Theme preference (Dark/Light)
- Window size

## Building

```bash
cargo build -p ritmo_gui
```

## Running

```bash
cargo run -p ritmo_gui
```

## Testing

```bash
cargo test -p ritmo_gui
```

## Integration

Uses ritmo_commands for all business logic:
- `ListBooksCommand` - List books with filters
- `ListContentsCommand` - List contents with filters

## Technologies

- **egui 0.27**: Immediate mode GUI framework
- **eframe 0.27**: Application framework
- **Tokio**: Asynchronous runtime
- **ritmo_commands**: Business logic layer
