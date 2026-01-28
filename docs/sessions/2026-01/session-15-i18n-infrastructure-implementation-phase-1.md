## 2026-01-26 - Session 15: i18n Infrastructure Implementation (Phase 1)

**Context**
After completing the i18n preparation for roles and language_role (Session 14) and conducting a comprehensive analysis of ~550 system strings requiring translation, this session implemented the complete i18n infrastructure (Phase 1 of the 5-phase plan). The goal was to establish the foundation with rust-i18n framework, translation files, locale detection utilities, and comprehensive documentation.

**Framework Selection: rust-i18n v3**

Chosen for:
- Simple `t!()` macro for translations
- YAML-based translation files (easy for non-developers)
- Compile-time checking of translation keys
- Built-in pluralization support
- Automatic locale detection
- Active maintenance and good documentation

**Translation Files Structure**

Created `locales/` directory with:
- `en.yml` - English translations (default language)
- `it.yml` - Italian translations
- `README.md` - Contributor guide for translators

**Initial Translation Coverage** (~54 keys):

```yaml
# Namespaces implemented
db.*           # 17 keys: roles, language_role, types, formats
cli.*          #  6 keys: app info, common messages
error.*        # 15 keys: database, book, content, file, validation errors
gui.*          # 13 keys: sidebar, search, empty state
validation.*   #  3 keys: person, language, date format errors
```

**Key Naming Convention**

Established pattern: `{namespace}.{category}.{subcategory}.{key}`

Examples:
- `db.role.author` - Database role display name
- `cli.common.success` - CLI success message
- `error.book.not_found` - Book not found error
- `gui.sidebar.books` - GUI sidebar label
- `validation.date_format` - Date validation error

**Locale Detection Utility** (`ritmo_db/src/i18n_utils.rs`)

Created module with functions:

✅ `detect_locale()` - Auto-detect best locale with priority:
  1. `RITMO_LANG` environment variable (e.g., `RITMO_LANG=it`)
  2. `LANG` environment variable (e.g., `LANG=it_IT.UTF-8` → "it")
  3. Default fallback ("en")

✅ `set_locale(locale: &str)` - Manually set application locale

✅ `get_locale() -> String` - Get current active locale

✅ `init_i18n()` - Initialize i18n with auto-detected locale (call in main())

✅ Constants: `SUPPORTED_LOCALES`, `DEFAULT_LOCALE`

**Model Integration**

Updated existing models to use i18n system:

✅ `Role::display_name()` - Updated implementation:
```rust
// Before: String manipulation fallback with TODO comment
// After: Uses t!() macro for translation
pub fn display_name(&self) -> String {
    use rust_i18n::t;
    let translation_key = format!("db.{}", self.key);
    t!(&translation_key).to_string()
}
```

✅ `RunningLanguages::display_role()` - Updated implementation:
```rust
// Before: Match statement fallback
// After: Uses t!() macro for translation
pub fn display_role(&self) -> String {
    use rust_i18n::t;
    let translation_key = format!("db.{}", self.role);
    t!(&translation_key).to_string()
}
```

**Testing**

Created comprehensive integration tests (`ritmo_db/tests/i18n_integration_test.rs`):

✅ `test_role_display_name_english` - Verify English translations
✅ `test_role_display_name_italian` - Verify Italian translations
✅ `test_all_role_translations` - Test all 6 role keys in both languages
✅ `test_language_role_display_english` - Verify language role English
✅ `test_language_role_display_italian` - Verify language role Italian
✅ `test_all_language_role_translations` - Test all 3 language_role keys
✅ `test_locale_switching` - Verify dynamic locale switching works

**Test Results:**
- 7 integration tests created
- All tests passing with `--test-threads=1` (required for locale state management)
- Tests verify translations work correctly in both English and Italian
- Tests verify locale switching works dynamically

**Documentation**

Created comprehensive documentation:

✅ `docs/i18n.md` (407 lines) - Complete developer guide:
  - Overview of i18n system
  - What to translate vs what not to translate
  - How to use `t!()` macro with examples
  - Locale management functions
  - Translation file format and key naming
  - Complete examples with step-by-step instructions
  - Model display name examples (Role, RunningLanguages)
  - How to add new languages
  - Testing i18n translations
  - Best practices and troubleshooting
  - Future improvements roadmap

✅ `locales/README.md` (135 lines) - Translator contributor guide:
  - Supported languages
  - File structure
  - Key naming convention
  - Variable substitution examples
  - How to add new translations
  - Best practices for translators
  - What to translate vs what not to translate
  - Translation guidelines
  - How to contribute new languages
  - Current translation coverage table

**Configuration**

✅ Updated `Cargo.toml` (workspace):
```toml
[workspace.dependencies]
rust-i18n = "3"
```

✅ Updated `ritmo_db/Cargo.toml`:
```toml
[dependencies]
rust-i18n = { workspace = true }
```

✅ Initialized in `ritmo_db/src/lib.rs`:
```rust
rust_i18n::i18n!("../locales", fallback = "en");

pub mod i18n_utils;
```

**Usage Examples**

Basic translation:
```rust
use rust_i18n::t;

let message = t!("cli.common.success");
// English: "✓ Operation completed successfully"
// Italian: "✓ Operazione completata con successo"
```

With variables:
```rust
let message = t!("error.book.not_found", id = 42);
// English: "Book with ID 42 not found"
// Italian: "Libro con ID 42 non trovato"
```

Locale management:
```rust
use ritmo_db::i18n_utils::{init_i18n, set_locale};

// Initialize with auto-detection
init_i18n();

// Or set manually
set_locale("it");  // Switch to Italian
```

Command-line usage:
```bash
# English (default)
cargo run -p ritmo_cli -- list-books

# Italian
RITMO_LANG=it cargo run -p ritmo_cli -- list-books

# Use system locale
LANG=it_IT.UTF-8 cargo run -p ritmo_cli -- list-books
```

**Files Created**
- Created: `locales/en.yml` (106 lines)
- Created: `locales/it.yml` (106 lines)
- Created: `locales/README.md` (135 lines)
- Created: `ritmo_db/src/i18n_utils.rs` (135 lines)
- Created: `ritmo_db/tests/i18n_integration_test.rs` (186 lines)
- Created: `docs/i18n.md` (407 lines)

**Files Modified**
- Modified: `Cargo.toml` (added rust-i18n workspace dependency)
- Modified: `Cargo.lock` (238 new lines from dependencies)
- Modified: `ritmo_db/Cargo.toml` (added rust-i18n dependency)
- Modified: `ritmo_db/src/lib.rs` (added i18n initialization and module)
- Modified: `ritmo_db/src/models/roles.rs` (updated display_name() to use t!())
- Modified: `ritmo_db/src/models/languages.rs` (updated display_role() to use t!())

**Statistics**
- Total changes: 12 files, 1324 insertions(+), 53 deletions(-)
- Translation keys: 54 initial keys (10% of ~550 total)
- Test coverage: 7 integration tests, all passing
- Documentation: 677 lines of comprehensive guides

**Impact**

The i18n infrastructure is now:
- ✅ Fully operational and ready for use
- ✅ Role and RunningLanguages models translate automatically
- ✅ Easy to add new translation keys (just edit YAML files)
- ✅ Simple to use in code (t!() macro)
- ✅ Locale detection works automatically
- ✅ Can switch languages at runtime
- ✅ Comprehensive documentation for developers and translators
- ✅ All tests passing

**Design Decisions**

1. **YAML over JSON/TOML**: Easier for non-technical translators
2. **Workspace dependency**: Centralized version management
3. **Initialized in ritmo_db**: Most fundamental crate, used everywhere
4. **Priority detection**: RITMO_LANG > LANG > default (explicit wins)
5. **Key naming convention**: Hierarchical, descriptive, not abbreviated
6. **Test with --test-threads=1**: Avoids race conditions in locale state

**Migration Strategy**

For existing code:
1. Phase 1 (completed): Infrastructure + ~54 base keys
2. Phase 2 (future): Database Models (types, formats, languages) - ~60 keys
3. Phase 3 (future): Errors & Services - ~70 keys
4. Phase 4 (future): CLI (commands, help, messages) - ~300 keys
5. Phase 5 (future): GUI (Slint + backend) - ~30 keys

**Known Limitations**

- Tests require `--test-threads=1` due to global locale state
- `t!()` returns `Cow<str>`, need `.to_string()` for String return types
- Only 2 languages initially (en, it) - more can be added easily
- ~500 strings still need translation (90% remaining)

**Next Steps (Not Implemented)**

- Add more translation keys progressively (Phases 2-5)
- Implement CLI commands to use i18n
- Add GUI language switcher
- Implement error messages with i18n
- Add service layer messages with i18n
- Consider pluralization for count-based messages
- Add date/time localization
- Create translation completeness validation tool

**Testing Instructions**

Run i18n tests:
```bash
# All i18n tests (sequential to avoid race conditions)
cargo test --package ritmo_db --test i18n_integration_test -- --test-threads=1

# Specific test
cargo test --package ritmo_db --test i18n_integration_test test_role_display_name_english -- --test-threads=1
```

Verify manual translation:
```rust
use ritmo_db::{Role, i18n_utils::set_locale};

let role = Role {
    id: Some(1),
    key: "role.author".to_string(),
    created_at: 1234567890,
};

set_locale("en");
assert_eq!(role.display_name(), "Author");

set_locale("it");
assert_eq!(role.display_name(), "Autore");
```

---

