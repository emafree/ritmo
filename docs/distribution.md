# Ritmo Distribution Model

## Overview

Ritmo uses a **library-centric distribution model** where the primary distribution method is through pre-configured library bundles with binaries included in the `bootstrap/portable_app/` directory. This approach provides users with ready-to-use libraries that can be easily duplicated when additional libraries are needed.

## Portable Library Structure

A portable Ritmo library has the following structure:

```
MyRitmoLibrary/
├── database/           # SQLite database files
│   └── ritmo.db
├── storage/           # Content storage
│   ├── books/        # EPUB and other book files
│   ├── covers/       # Book cover images
│   └── temp/         # Temporary files
├── config/           # Library configuration
│   ├── ritmo.toml   # Library settings
│   └── filters.toml # Filter presets
└── bootstrap/        # Distribution files
    └── portable_app/ # Contains ritmo_cli and ritmo_gui executables
```

## Portable Mode Detection

When Ritmo executables are located in `bootstrap/portable_app/`, the system automatically detects it's running in **portable mode**. The library root is identified as two levels up from the executable location.

**Example:**
- Executable path: `/path/to/MyLibrary/bootstrap/portable_app/ritmo_cli`
- Library root: `/path/to/MyLibrary`

## Commands

### `ritmo libraries init`

Initialize a new library from scratch.

**Usage:**
```bash
ritmo libraries init [PATH]
```

**Default path:** `~/RitmoLibrary`

**Portable Mode Restriction:**
- ❌ **Cannot** be used when running from portable mode
- Returns error message directing user to use `duplicate` instead
- This prevents confusion and ensures users understand the proper workflow

**Example:**
```bash
# From a system-installed Ritmo
$ ritmo libraries init ~/MyNewLibrary
Initializing library: ~/MyNewLibrary
✓ Directories created
✓ Database initialized
...
✓ Library initialized successfully!
```

### `ritmo libraries duplicate`

Duplicate the current portable library to a new location.

**Usage:**
```bash
ritmo libraries duplicate <OUTPUT_PATH>
```

**Required:**
- Must be running from portable mode (from `bootstrap/portable_app/`)
- Output path must not already exist or be empty

**What it does:**
1. Copies entire library structure to output path
2. Resets database to clean template (empty but initialized)
3. Updates library configuration
4. Creates example filter presets
5. Sets new library as current in AppSettings

**Example:**
```bash
# From within a portable library
$ cd MyLibrary/bootstrap/portable_app
$ ./ritmo_cli libraries duplicate ~/MySecondLibrary
Duplicating portable library to: ~/MySecondLibrary
✓ Copying library files...
✓ Resetting database to template...
✓ Updating library configuration...
✓ Library duplicated successfully!
```

## Typical Workflows

### Workflow 1: Using Distributed Portable Library

Most users will receive a pre-configured portable library:

1. **Receive** portable library bundle (e.g., `RitmoLibrary.zip`)
2. **Extract** to desired location
3. **Use** executables in `bootstrap/portable_app/`
4. All data stays within the library directory (fully portable)

### Workflow 2: Creating Additional Libraries

When users need multiple libraries:

1. **Navigate** to portable library: `cd MyLibrary/bootstrap/portable_app`
2. **Run duplicate**: `./ritmo_cli libraries duplicate ~/MySecondLibrary`
3. **Use new library** at `~/MySecondLibrary`
4. Each library maintains its own database and configuration

### Workflow 3: Developer/System Installation

For developers or system-wide installations:

1. **Install** Ritmo system-wide (e.g., `cargo install ritmo_cli`)
2. **Initialize** library: `ritmo libraries init ~/MyLibrary`
3. **Use** from any location: `ritmo --library ~/MyLibrary books list`

## Error Messages

### Init from Portable Mode

```
✗ Cannot initialize a new library while running in portable mode
  Use 'ritmo libraries duplicate <path>' to create a copy of the current portable library
Error: "Cannot initialize library from portable mode (running from /path/to/library)"
```

### Duplicate from Non-Portable Mode

```
✗ Not running in portable mode. Duplicate command only works from portable libraries.
  Use 'ritmo libraries init <path>' to create a new library
Error: "Not running in portable mode"
```

## Technical Details

### Database Reset

When duplicating, the database is reset to ensure a clean state:
- Source database (with user data) is **not** copied
- Fresh template database (empty schema) is written instead
- This ensures each duplicated library starts with clean data
- Template is embedded in the binary as `DB_TEMPLATE`

### Configuration Update

After duplication:
- Library paths in `ritmo.toml` are updated to match new location
- Filter presets are recreated with default examples
- AppSettings is updated to track the new library

### Localization

Both commands support English and Italian locales:
- Messages automatically display in user's preferred language
- Set via `RITMO_LANG` environment variable or AppSettings
- Language detection follows rust_i18n conventions

## Benefits of This Model

1. **Portability**: Entire library is self-contained and movable
2. **Simplicity**: Users receive ready-to-use bundles
3. **Isolation**: Each library is independent with its own data and config
4. **Clarity**: Clear distinction between init (new) and duplicate (copy)
5. **Safety**: Prevents accidental nested library creation

## See Also

- [Architecture](architecture.md) - Overall system design
- [Development Guide](development.md) - Development setup
- [Command Layer](command-layer.md) - CLI command structure
