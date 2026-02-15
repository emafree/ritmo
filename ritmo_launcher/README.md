# Ritmo Launcher

The `ritmo_launcher` is a cross-platform launcher for portable Ritmo installations. It provides comprehensive library management including detection, verification, auto-repair, and binary download capabilities.

## Features

- **Automatic Library Detection**: Detects library path from multiple sources
  - Environment variable (`RITMO_LIBRARY_PATH`)
  - Current working directory
  - Parent directories (walking up the tree)
  - Executable location (for portable installations)
  - Default location (`~/.ritmo`)

- **Library Structure Verification**: Checks for required directories and files

- **Auto-Repair**: Automatically repairs corrupted libraries while preserving books

- **Binary Download**: Downloads missing binaries from GitHub releases (user-initiated)

- **User-Friendly Interface**: Soft messaging with clear prompts

- **Offline-First**: No unwanted downloads; always asks user permission

## Directory Structure

```
library_root/
├── ritmo_library/         # Library data (config, database, storage)
│   ├── config/           # Configuration files
│   ├── database/         # Database files
│   └── storage/          # Book storage (TOML files)
└── bootstrap/
    └── portable_app/
        ├── ritmo_launcher[.exe]  # This executable
        └── ritmo_gui[.exe]       # GUI executable
```

## Usage

Simply run the launcher executable:

```bash
./ritmo_launcher
```

The launcher will:
1. Detect the library path
2. Verify the library structure
3. Check for missing binaries
4. Download binaries if missing (with user confirmation)
5. Auto-repair the library if needed (preserving books)
6. Launch the GUI

## Module Structure

- `main.rs` - Main flow and orchestration
- `library_verifier.rs` - Verify library structure and count books
- `library_repairer.rs` - Auto-repair with book preservation
- `binary_downloader.rs` - Download binaries from GitHub
- `ui.rs` - User prompts and soft messaging
- `config_generator.rs` - Generate default configurations

## Environment Variables

- `RITMO_LIBRARY_PATH` - Override the automatic library detection

## Platform Support

- **Linux**: Fully supported with tar.gz binary downloads
- **Windows**: Fully supported with ZIP binary downloads
- **macOS**: Supported with tar.gz binary downloads

## Security

- SHA256 verification of downloaded files
- User confirmation required before downloading
- All downloads are from official GitHub releases
- Proper error handling for all I/O operations

## Testing

Run the test suite:

```bash
cargo test -p ritmo_launcher
```

All 7 unit tests should pass.

## Building

Debug build:
```bash
cargo build -p ritmo_launcher
```

Release build:
```bash
cargo build --release -p ritmo_launcher
```

## License

Same as the main Ritmo project.
