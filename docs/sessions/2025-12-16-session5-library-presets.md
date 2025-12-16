# Session 5: Library-Specific Preset System (Phase 2)
**Date**: 2025-12-16  
**Focus**: Implementation of library-specific presets with portable filters.toml

## Summary

This session completed Phase 2 of the Filter Preset System, implementing library-specific presets that are stored in `library/config/filters.toml` and travel with the library. This makes presets fully portable and enables per-library customization while maintaining global presets for cross-library use.

## Goals Achieved

### ✅ Primary Objectives
1. **LibraryPresets structure** - Preset storage in library/config/filters.toml
2. **Preset resolution system** - Priority order: library > global
3. **CLI enhancements** - --in-library flag, set-default-filter command
4. **Auto-create examples** - Example presets on library initialization
5. **Full testing** - End-to-end verification with real library
6. **Documentation** - Updated CLAUDE.md and session docs

## Implementation Details

### 1. LibraryPresets Structure (`ritmo_db_core/src/library_presets.rs`)

**Main Structure:**
```rust
pub struct LibraryPresets {
    pub books: HashMap<String, NamedPreset<BookFilterPreset>>,
    pub contents: HashMap<String, NamedPreset<ContentFilterPreset>>,
    pub default_books_preset: Option<String>,
    pub default_contents_preset: Option<String>,
}
```

**Key Features:**
- Stores presets in `library/config/filters.toml` (portable!)
- Supports default preset selection per library
- Auto-creates example presets on initialization:
  - `epub_only`: Filter for EPUB books
  - `pdf_only`: Filter for PDF books
  - `novels`: Filter for novel-type contents

**Example filters.toml:**
```toml
default_books_preset = "epub_only"

[books.epub_only]
name = "epub_only"
description = "Solo libri in formato EPUB"

[books.epub_only.filters]
format = "epub"
sort = "title"
offset = 0

[contents.novels]
name = "novels"
description = "Solo romanzi"

[contents.novels.filters]
content_type = "Romanzo"
sort = "title"
offset = 0
```

**Key Methods:**
```rust
impl LibraryPresets {
    // Load from file or create with examples
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Result<Self>
    
    // Create with example presets
    pub fn with_examples() -> Self
    
    // Save to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()>
    
    // Preset management
    pub fn add_book_preset(&mut self, preset: NamedPreset<BookFilterPreset>)
    pub fn get_book_preset(&self, name: &str) -> Option<&NamedPreset<BookFilterPreset>>
    pub fn remove_book_preset(&mut self, name: &str) -> Option<NamedPreset<BookFilterPreset>>
    
    // Default preset management
    pub fn set_default_books_preset(&mut self, name: Option<String>)
    pub fn get_default_books_preset(&self) -> Option<&str>
}
```

### 2. Preset Resolution System (`ritmo_config/src/preset_resolver.rs`)

**PresetResolver:**
```rust
pub struct PresetResolver {
    global_presets: GlobalPresets,
    library_presets: Option<LibraryPresetsHolder>,
}

impl PresetResolver {
    // Create with only global presets
    pub fn new(global_presets: GlobalPresets) -> Self
    
    // Create with both global and library presets
    pub fn with_library(global_presets: GlobalPresets, library_presets: LibraryPresetsHolder) -> Self
    
    // Resolve preset with priority: library > global
    pub fn resolve_book_preset(&self, name: &str) -> Option<&NamedPreset<BookFilterPreset>>
    pub fn resolve_content_preset(&self, name: &str) -> Option<&NamedPreset<ContentFilterPreset>>
    
    // Get default presets from library
    pub fn get_default_books_preset(&self) -> Option<&str>
    pub fn get_default_contents_preset(&self) -> Option<&str>
    
    // List all available presets with their source
    pub fn list_all_book_presets(&self) -> Vec<(String, PresetSource)>
    pub fn list_all_content_presets(&self) -> Vec<(String, PresetSource)>
}
```

**PresetSource:**
```rust
pub enum PresetSource {
    Library,  // From library/config/filters.toml
    Global,   // From ~/.config/ritmo/settings.toml
}
```

**LibraryPresetsHolder:**
Bridge structure to avoid circular dependencies between `ritmo_db_core` and `ritmo_config`.

### 3. LibraryConfig Integration (`ritmo_db_core/src/lib.rs`)

**New Methods:**
```rust
impl LibraryConfig {
    // Get path to filters.toml
    pub fn filters_file(&self) -> PathBuf
    
    // Load library presets (creates examples if not exists)
    pub fn load_library_presets(&self) -> Result<LibraryPresets>
    
    // Save library presets
    pub fn save_library_presets(&self, presets: &LibraryPresets) -> Result<()>
}
```

### 4. CLI Commands Enhanced

#### save-preset with --in-library flag
```bash
# Save globally (default)
ritmo save-preset books --name my_preset --format epub

# Save in library (portable!)
ritmo save-preset books --name my_preset --format epub --in-library
```

**Implementation:**
- Added `in_library: bool` parameter to `SavePreset` command
- Detects library location via `get_library_path()`
- Loads library presets, adds new preset, saves back
- Clear feedback: "salvato nella libreria" vs "salvato globalmente"

#### set-default-filter command (NEW)
```bash
# Set default preset for books
ritmo set-default-filter books epub_only

# Set default for contents
ritmo set-default-filter contents novels

# Remove default
ritmo set-default-filter books none
```

**Implementation:**
- Validates preset exists in library before setting as default
- Updates `default_books_preset` or `default_contents_preset`
- Saves back to library's filters.toml
- Provides clear error messages

#### list-presets enhanced
Shows library and global presets separately with clear labels:

```
Preset per Libri (Libreria):
--------------------------------------------------
• epub_only
  Descrizione: Solo libri in formato EPUB
  Filtri: formato=epub, ordina=title

• pdf_only
  Descrizione: Solo libri in formato PDF
  Filtri: formato=pdf, ordina=title

Default: epub_only

Preset per Libri (Globali):
--------------------------------------------------
• my_ebooks
  Descrizione: All my EPUB books
  Filtri: formato=epub, ordina=title
```

**Implementation:**
- Loads library presets if available
- Shows library presets first with "(Libreria)" label
- Shows global presets with "(Globali)" label
- Displays default preset if set

#### list-books/list-contents with preset resolution

**Automatic Resolution:**
```rust
// Load library presets
let library_presets = config.load_library_presets().ok();

// Resolve preset: library > global
let preset = if let Some(ref lib_presets) = library_presets {
    lib_presets.get_book_preset(&preset_name)
        .or_else(|| app_settings.presets.get_book_preset(&preset_name))
} else {
    app_settings.presets.get_book_preset(&preset_name)
}
.ok_or_else(|| format!("Preset '{}' non trovato", preset_name))?;
```

**Priority Order:**
1. Library preset (if exists)
2. Global preset (fallback)
3. Error if not found

#### init command enhanced
Automatically creates example presets:
```rust
// During library initialization
let _library_presets = config.load_library_presets()?;
println!("✓ Preset di esempio creati (epub_only, pdf_only, novels)");
```

## Files Created

1. **ritmo_db_core/src/library_presets.rs** (~280 lines)
   - LibraryPresets struct
   - Example preset creation
   - Load/save functionality
   - Full test coverage (7/7 tests)

2. **ritmo_config/src/preset_resolver.rs** (~240 lines)
   - PresetResolver with priority resolution
   - LibraryPresetsHolder bridge struct
   - PresetSource enum
   - Full test coverage (4/4 tests)

## Files Modified

1. **ritmo_db_core/src/lib.rs**
   - Exported LibraryPresets
   - Added `filters_file()`, `load_library_presets()`, `save_library_presets()`

2. **ritmo_db_core/Cargo.toml**
   - Added `ritmo_config` dependency

3. **ritmo_config/src/lib.rs**
   - Made `presets` module public
   - Exported PresetResolver, LibraryPresetsHolder, PresetSource

4. **ritmo_cli/src/main.rs** (~200 lines added)
   - Added `in_library` flag to SavePreset
   - New SetDefaultFilter command
   - Enhanced cmd_list_presets with library/global separation
   - Updated cmd_list_books/cmd_list_contents with preset resolution
   - Enhanced cmd_init to create example presets

## Testing

### Unit Tests
- **ritmo_db_core**: 7/7 library_presets tests passing
- **ritmo_config**: 4/4 preset_resolver tests passing
- **Total workspace**: 23/23 tests passing

### End-to-End Testing

**Test Scenario:**
```bash
# 1. Initialize library
ritmo init /tmp/test_ritmo_library
# ✓ Creates filters.toml with example presets

# 2. Add books
ritmo --library /tmp/test_ritmo_library add test.epub --title "Test EPUB"
ritmo --library /tmp/test_ritmo_library add test.pdf --title "Test PDF"

# 3. Test presets
ritmo --library /tmp/test_ritmo_library list-books --preset epub_only
# Result: Shows only EPUB book

ritmo --library /tmp/test_ritmo_library list-books --preset pdf_only
# Result: Shows only PDF book

# 4. Create custom preset
ritmo --library /tmp/test_ritmo_library save-preset books \
  --name test_preset --in-library --format mobi

# 5. Set default
ritmo --library /tmp/test_ritmo_library set-default-filter books epub_only

# 6. Verify preset resolution (library > global)
# Created global preset "epub_only" with different settings
# Confirmed library preset takes priority
```

**Results:**
- ✅ Library initialization creates filters.toml
- ✅ Example presets work correctly
- ✅ Custom presets save to library
- ✅ Default preset setting works
- ✅ Preset resolution priority verified
- ✅ filters.toml is portable with library

## Portable Workflow

### Complete Example

**Setup:**
```bash
# Initialize portable library on USB
ritmo init /media/usb/MyLibrary

# Import books
ritmo --library /media/usb/MyLibrary add book1.epub --title "Book 1"
ritmo --library /media/usb/MyLibrary add book2.pdf --title "Book 2"

# Create custom presets for this library
ritmo --library /media/usb/MyLibrary save-preset books \
  --name my_collection --in-library --author "Calvino" --format epub \
  --description "Libri di Calvino in EPUB"

# Set as default for this library
ritmo --library /media/usb/MyLibrary set-default-filter books my_collection
```

**Share:**
```bash
# Copy USB to another computer
# The library is fully portable with:
# - Database (ritmo.db)
# - Books (storage/books/)
# - Presets (config/filters.toml) ← NEW!
# - Config (config/ritmo.toml)
```

**Use on Another Computer:**
```bash
# No setup needed - presets are already there!
ritmo --library /media/usb/MyLibrary list-books
# Uses "my_collection" preset automatically (it's the default)

ritmo --library /media/usb/MyLibrary list-presets
# Shows all library presets including "my_collection"
```

## Benefits

### 1. Full Portability
- Presets travel with the library
- No need to recreate presets on each computer
- Perfect for USB libraries or shared libraries

### 2. Per-Library Customization
- Different presets for different libraries
- Each library can have its own defaults
- No conflicts between libraries

### 3. Seamless Integration
- Works transparently with existing global presets
- Library presets take priority automatically
- Clear separation in UI (list-presets)

### 4. Zero Configuration
- Example presets created automatically
- Ready to use immediately after init
- Sensible defaults (epub_only, pdf_only, novels)

## Architecture Decisions

### 1. Preset Storage Location
**Decision**: Store in `library/config/filters.toml`  
**Rationale**: 
- Lives alongside library data
- Portable with library
- Separate from global settings
- Easy to version control

### 2. Resolution Priority
**Decision**: Library > Global  
**Rationale**:
- Library-specific customization overrides global defaults
- Intuitive behavior for users
- Allows both general and specific presets

### 3. Dependency Management
**Decision**: Created LibraryPresetsHolder bridge struct  
**Rationale**:
- Avoids circular dependency between ritmo_db_core and ritmo_config
- Clean separation of concerns
- Minimal coupling

### 4. Example Presets
**Decision**: Auto-create on library init  
**Rationale**:
- Immediate value for new users
- Demonstrates preset functionality
- Covers common use cases (epub, pdf, novels)

## Next Steps

### Phase 3: Enhanced Preset Features (Future)
- Auto-save last used filter
- `--use-last` flag to reuse previous filter
- Interactive preset editing
- Preset templates

### Integration Opportunities
- GUI preset management
- Export/import preset collections
- Preset validation and suggestions

## Lessons Learned

1. **Bridge Patterns**: LibraryPresetsHolder successfully avoided circular dependencies
2. **User Feedback**: Clear labeling (Libreria/Globali) improves UX significantly
3. **Testing Strategy**: End-to-end tests with real library caught integration issues
4. **Documentation**: Example workflows in docs are invaluable for understanding

## Commits

1. **462a878**: Session 2025-12-16: Library-specific preset system (Phase 2)
   - Created LibraryPresets and PresetResolver
   - Enhanced CLI commands
   - Full implementation with tests

2. **a25ada7**: Update CLAUDE.md: Document Session 5
   - Updated Recent Changes section
   - Updated TODO/Next Steps
   - Documented portable workflow

## Statistics

- **Lines Added**: ~875
- **Lines Modified**: ~60
- **Files Created**: 2
- **Files Modified**: 5
- **Tests Added**: 11
- **Tests Passing**: 23/23 (100%)
- **Session Duration**: ~3 hours
- **Commits**: 2

## Conclusion

Phase 2 of the Filter Preset System is now complete. Library-specific presets provide a powerful way to customize filter behavior per-library while maintaining global presets for cross-library use. The implementation is fully tested, documented, and ready for production use.

The portable nature of `filters.toml` makes this feature particularly valuable for USB-based or shared libraries, where presets can be configured once and travel with the library to any computer.

Next priority: Integrate ebook_parser for automatic metadata extraction from EPUB files.
