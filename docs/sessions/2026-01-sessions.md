# Session History - January 2026

This document contains all development sessions from January 2026.

---

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

## 2026-01-26 - Session 16: I18nDisplayable Trait Implementation

**Context**
After completing the i18n infrastructure (Session 15), the `Role` and `RunningLanguages` models had duplicate translation code. Both models implemented nearly identical translation logic: formatting the key, calling `t!()`, and converting to `String`. This session implemented the `I18nDisplayable` trait to eliminate code duplication, improve maintainability, and enable generic functions that work with any translatable model.

**Problem Analysis**

Before trait implementation, both models had 10+ lines of duplicate translation code:

```rust
// Role model
pub fn display_name(&self) -> String {
    use rust_i18n::t;
    let translation_key = format!("db.{}", self.key);
    t!(&translation_key).to_string()
}

// RunningLanguages model
pub fn display_role(&self) -> String {
    use rust_i18n::t;
    let translation_key = format!("db.{}", self.role);
    t!(&translation_key).to_string()
}
```

With 4+ models needing translation (Role, RunningLanguages, Types, Formats), this would result in 40+ lines of duplicated code and maintenance challenges.

**Trait Design**

Created the `I18nDisplayable` trait in `ritmo_db/src/i18n_trait.rs`:

```rust
pub trait I18nDisplayable {
    /// Returns the canonical i18n key (e.g., "role.author")
    fn i18n_key(&self) -> &str;

    /// Returns the namespace prefix (default: "db")
    fn i18n_namespace(&self) -> &str {
        "db"
    }

    /// Translates the key to a localized string
    fn translate(&self) -> String {
        use rust_i18n::t;
        let translation_key = format!("{}.{}", self.i18n_namespace(), self.i18n_key());
        t!(&translation_key).to_string()
    }
}
```

**Benefits of the Trait**

1. **Eliminates Duplication**: 10 lines per model → 3 lines
2. **Type Safety**: Compile-time checks for translation methods
3. **Consistency**: All models translate the same way
4. **Generic Functions**: Write code that works with any translatable model
5. **Maintainability**: Change translation logic in one place
6. **Future-Proof**: Easy to add new translatable models

**Implementation**

✅ **Created trait module** (`ritmo_db/src/i18n_trait.rs`, 66 lines):
- `I18nDisplayable` trait with documentation
- Default implementation for `i18n_namespace()` and `translate()`
- Comprehensive doc comments with examples

✅ **Implemented trait for Role**:
```rust
impl I18nDisplayable for Role {
    fn i18n_key(&self) -> &str {
        &self.key  // "role.author"
    }
}
```

✅ **Implemented trait for RunningLanguages**:
```rust
impl I18nDisplayable for RunningLanguages {
    fn i18n_key(&self) -> &str {
        &self.role  // "language_role.original"
    }
}
```

✅ **Updated display methods** to delegate to trait:
```rust
// Role::display_name()
pub fn display_name(&self) -> String {
    self.translate()  // Delegates to I18nDisplayable
}

// RunningLanguages::display_role()
pub fn display_role(&self) -> String {
    self.translate()  // Delegates to I18nDisplayable
}
```

**Testing**

Created comprehensive generic tests (`ritmo_db/tests/i18n_trait_test.rs`, 194 lines):

✅ `test_trait_translate_role` - Verify trait works for Role
✅ `test_trait_translate_running_languages` - Verify trait works for RunningLanguages
✅ `test_trait_multiple_instances` - Test with multiple instances and locales
✅ `test_trait_i18n_key` - Verify i18n_key() returns correct key
✅ `test_trait_i18n_namespace` - Verify namespace is "db"
✅ `test_trait_generic_function` - Test generic function accepting any I18nDisplayable
✅ `test_trait_missing_translation` - Verify graceful handling of missing keys

**Test Results**:
- 7 new trait tests created
- All 14 i18n tests passing (7 integration + 7 trait tests)
- Tests verify trait methods work correctly in both English and Italian
- Tests verify display methods delegate to trait correctly
- Tests verify generic functions work with any I18nDisplayable type

**Documentation Updates**

✅ Updated `docs/i18n.md`:
- Added "I18nDisplayable Trait" section with complete examples
- Added "Benefits of the Trait" subsection
- Added "Using the Trait" section showing model implementations
- Added "Generic Functions" section with examples
- Updated "Model Display Names Example" to show delegation
- Added trait usage to "Using i18n in Code" section

✅ Updated `docs/architecture.md`:
- Added `i18n_trait` to ritmo_db description
- Updated "Model Integration" to mention trait delegation
- Changed description from "Models use t!() macro" to "Models implement I18nDisplayable trait"

**Files Created**
- Created: `ritmo_db/src/i18n_trait.rs` (66 lines)
- Created: `ritmo_db/tests/i18n_trait_test.rs` (194 lines)

**Files Modified**
- Modified: `ritmo_db/src/lib.rs` (added i18n_trait module)
- Modified: `ritmo_db/src/models/roles.rs` (implemented trait, updated display_name())
- Modified: `ritmo_db/src/models/languages.rs` (implemented trait, updated display_role())
- Modified: `ritmo_db/tests/i18n_integration_test.rs` (removed unused import)
- Modified: `docs/i18n.md` (added trait documentation, 100+ new lines)
- Modified: `docs/architecture.md` (updated i18n system description)

**Code Reduction**

Before trait:
- Role: 10 lines of translation code
- RunningLanguages: 10 lines of translation code
- Total: 20 lines (for 2 models)

After trait:
- Trait definition: 24 lines (shared)
- Role implementation: 5 lines
- RunningLanguages implementation: 5 lines
- Total: 34 lines (but scales much better)

For 4+ models:
- Before: 40+ lines of duplicated code
- After: 24 + (4 × 5) = 44 lines (no duplication)

**Impact**

The trait system provides:
- ✅ Consistent translation interface across all models
- ✅ Type-safe generic functions for i18n operations
- ✅ Elimination of code duplication
- ✅ Simplified future model integration
- ✅ Maintainable translation logic in one place
- ✅ All tests passing (14 total: 7 integration + 7 trait tests)

**Generic Function Example**

The trait enables powerful generic code:

```rust
fn get_translation<T: I18nDisplayable>(item: &T) -> String {
    item.translate()
}

// Works with any I18nDisplayable type
let role = Role { key: "role.author".to_string(), ... };
let lang = RunningLanguages { role: "language_role.original".to_string(), ... };

println!("{}", get_translation(&role));  // "Author" / "Autore"
println!("{}", get_translation(&lang));  // "Original Language" / "Lingua Originale"
```

**Statistics**
- Total changes: 7 files, 260+ insertions(+), 30 deletions(-)
- New trait tests: 7 tests (all passing)
- Documentation: 100+ new lines in i18n.md
- Code reduction: Eliminated 10+ lines of duplication per model

**Next Steps (Not Implemented)**

Future models can implement the trait with minimal code:
```rust
// Future: Types model
impl I18nDisplayable for Types {
    fn i18n_key(&self) -> &str {
        &self.key  // "type.book", "type.magazine", etc.
    }
}

// Future: Formats model
impl I18nDisplayable for Formats {
    fn i18n_key(&self) -> &str {
        &self.key  // "format.epub", "format.pdf", etc.
    }
}
```

**Testing Instructions**

Run all i18n tests (sequential to avoid race conditions):
```bash
# All i18n integration tests
cargo test --package ritmo_db --test i18n_integration_test -- --test-threads=1

# All i18n trait tests
cargo test --package ritmo_db --test i18n_trait_test -- --test-threads=1

# Both test files together
cargo test --package ritmo_db --test i18n_integration_test --test i18n_trait_test -- --test-threads=1
```

**Design Decisions**

1. **Single Required Method**: Only `i18n_key()` must be implemented, reducing boilerplate
2. **Default Namespace**: "db" is sensible default for all database models
3. **Default Implementation**: `translate()` works for 99% of cases without override
4. **Delegation Pattern**: Existing display methods delegate to trait for backward compatibility
5. **Comprehensive Tests**: Generic tests ensure trait works consistently across types

---

## 2026-01-26 - Session 14: Roles & Language Roles i18n Integration

**Context**
The `roles` model and `language_role` field in `running_languages` used plain text strings (e.g., "Autore", "Author" for roles; "Original", "Source", "Actual" for language roles), which prevented internationalization (i18n) and made ML deduplication difficult for roles. This session refactored both systems to use canonical i18n keys instead of translated strings, preparing the foundation for future i18n support while integrating roles into the ML deduplication system.

Language roles are fixed system values (only 3 possible values) and don't require ML deduplication, but still benefit from i18n support for display purposes.

**Database Schema Changes**
✅ Updated `roles` table in `ritmo_db_core/assets/template.db`:
  - Changed column `name TEXT` → `key TEXT NOT NULL UNIQUE`
  - The `key` field now stores canonical i18n keys (e.g., "role.author", "role.translator")
  - Added `created_at` field to schema (was already in CREATE statement)
  - Maintains UNIQUE constraint via index `idx_roles_key`

**ritmo_db Model Updates**
✅ Updated `ritmo_db/src/models/roles.rs`:
  - Changed struct field: `name: String` → `key: String`
  - Added `display_name()` method for UI display (currently returns fallback, ready for future i18n)
  - Added new methods:
    - `get_all()` - Retrieve all roles ordered by key
    - `get_by_key()` - Find role by canonical key
    - `get_or_create_by_key()` - Get or create role by key (replaces deprecated method)
  - Deprecated methods for backward compatibility:
    - `get_by_name()` → use `get_by_key()` instead
    - `get_or_create_by_name()` → use `get_or_create_by_key()` instead
  - Updated `save()` to insert `key` and `created_at`
  - Updated `update()` to modify `key` instead of `name`

**ritmo_core Service Updates**
✅ Updated 4 service files to use new `get_or_create_by_key()` method:
  - `book_import_service.rs:146` - Import books with people and roles
  - `book_update_service.rs:108` - Update book people and roles
  - `content_create_service.rs:76` - Create content with people and roles
  - `content_update_service.rs:92` - Update content people and roles

**ritmo_ml Integration**
✅ Updated `ritmo_ml/src/db_loaders.rs`:
  - Changed SQL query to select `key` column instead of `name`
  - RoleRecord now receives canonical keys for ML processing

✅ Updated `ritmo_ml/src/test_helpers.rs`:
  - Updated `CREATE TABLE roles` to use `key TEXT NOT NULL UNIQUE`
  - Updated `populate_test_roles()` with realistic i18n key data:
    - Test duplicates: "role.author" / "role.autor" (typo)
    - Test duplicates: "role.translator" / "role.traduttore" (language variant)
    - Test duplicates: "role.illustrator" / "role.ilustrator" (typo)
  - All test data now uses canonical keys instead of translated names

✅ Updated test assertion in `db_loaders.rs:313`:
  - Changed expected value from `"Autore"` to `"role.author"`

**CLI Integration**
✅ The `deduplicate-roles` command was already implemented in Session 12
✅ The `deduplicate-all` command already included roles
✅ No CLI changes needed - commands work with new schema

**Testing**
✅ All `ritmo_ml` tests pass (20 tests total):
  - `test_load_roles_from_db` - Loads roles with new `key` column
  - `test_merge_roles` - Merges duplicate role keys
  - `test_deduplicate_roles` - End-to-end deduplication with new schema
  - All other tests updated to work with new schema

✅ Full workspace build successful:
  - Zero compilation errors
  - Only 2 minor warnings (unused `is_valid_library` function, unused `CliReporter` struct)
  - All tests passing

**Migration Strategy**
The changes are **breaking** for existing databases:
- Old databases with `name` column will not work with new code
- Existing libraries need database migration (manual or scripted)
- Template database updated for new library initializations
- Future consideration: Add migration script for existing libraries

**Design Rationale**
1. **i18n Foundation**: Using canonical keys (e.g., "role.author") allows future translation without database changes
2. **ML Compatibility**: Canonical keys are more suitable for ML deduplication than translated strings
3. **Backward Compatibility**: Deprecated methods allow gradual migration of calling code
4. **Display Flexibility**: `display_name()` method provides abstraction for future i18n implementation

**Language Roles Integration**
✅ Updated `ritmo_db/schema/schema.sql`:
  - Changed CHECK constraint in `running_languages` table:
    - Old: `CHECK("language_role" IN ('Original', 'Source', 'Actual'))`
    - New: `CHECK("language_role" IN ('language_role.original', 'language_role.source', 'language_role.actual'))`
  - No need for ML deduplication (only 3 fixed system values)

✅ Updated `ritmo_db/src/models/languages.rs`:
  - Added `language_role` constants module:
    - `language_role::ORIGINAL = "language_role.original"`
    - `language_role::SOURCE = "language_role.source"`
    - `language_role::ACTUAL = "language_role.actual"`
  - Added `display_role()` method to RunningLanguages struct:
    - Returns human-readable fallback for now ("Original", "Source", "Actual")
    - Ready for future i18n integration with `t!(&self.role)` macro
  - No changes needed to existing methods (`get_or_create_by_iso_and_role()` etc.)
  - Service layer continues to work as-is (receives role string from CLI/GUI)

✅ Regenerated template database:
  - Updated `ritmo_db_core/assets/template.db` from schema.sql
  - Both `roles` and `running_languages` tables updated simultaneously
  - Empty database (no pre-populated data for language roles - created on demand)

**Files Modified**
- Modified: `ritmo_db/schema/schema.sql` (2 tables: roles + running_languages)
- Modified: `ritmo_db/src/models/roles.rs` (115 lines changed: +68/-47)
- Modified: `ritmo_db/src/models/languages.rs` (37 lines added: constants + display_role())
- Modified: `ritmo_core/src/service/book_import_service.rs` (1 line)
- Modified: `ritmo_core/src/service/book_update_service.rs` (1 line)
- Modified: `ritmo_core/src/service/content_create_service.rs` (1 line)
- Modified: `ritmo_core/src/service/content_update_service.rs` (1 line)
- Modified: `ritmo_ml/src/db_loaders.rs` (6 lines)
- Modified: `ritmo_ml/src/test_helpers.rs` (20 lines)
- Modified: `ritmo_db_core/assets/template.db` (binary, schema changes)

**Impact**
Both roles and language_role systems now:
- Use canonical i18n keys instead of translated strings
- Are ready for future internationalization
- Roles integrate seamlessly with ML deduplication
- Language roles are system-only values (no ML needed, no user input)
- Roles maintain backward compatibility through deprecated methods
- All tests passing (20 ML tests + workspace build successful)

**Next Steps (Future Considerations)**
- Implement actual i18n system with translation files
- Create database migration script for existing libraries
- Remove deprecated methods in future major version
- Add GUI integration for role management with i18n

---

## 2026-01-25 - Session 12: ML CLI Integration Complete (+ Tags Support)

**Context**
The `ritmo_ml` crate was fully implemented and tested (Session 11), but lacked CLI commands for end users to perform deduplication. This session integrated the ML deduplication system into `ritmo_cli` with 5 new commands. Initially implemented for authors, publishers, and series, then extended to include tags support.

**New CLI Commands**
✅ `deduplicate-authors` - Find and merge duplicate authors using ML
✅ `deduplicate-publishers` - Find and merge duplicate publishers
✅ `deduplicate-series` - Find and merge duplicate series
✅ `deduplicate-tags` - Find and merge duplicate tags using ML
✅ `deduplicate-all` - Run deduplication for all entity types (authors, publishers, series, tags)

**Command Options (All Commands)**
- `--threshold <0.0-1.0>` - Minimum confidence threshold (default: 0.85)
- `--auto-merge` - Automatically merge high-confidence duplicates
- `--dry-run` - Preview mode without database changes (default: true)

**Implementation**
✅ Created `ritmo_cli/src/commands/deduplication.rs` (~330 lines):
  - `cmd_deduplicate_authors()` - Author deduplication command
  - `cmd_deduplicate_publishers()` - Publisher deduplication command
  - `cmd_deduplicate_series()` - Series deduplication command
  - `cmd_deduplicate_tags()` - Tags deduplication command
  - `cmd_deduplicate_all()` - All entity types command (includes tags)
  - `print_deduplication_results()` - User-friendly output formatter
  - Safety logic: dry-run defaults to true, only disabled with `--auto-merge`

✅ Extended `ritmo_ml/src/merge.rs`:
  - Added `merge_tags()` function for safe tag merging with transactions
  - Helper functions: `validate_tags_exist()`, `update_books_tags()`, `update_contents_tags()`, `delete_tags()`
  - Updates both `x_books_tags` and `x_contents_tags` junction tables

✅ Extended `ritmo_ml/src/deduplication.rs`:
  - Added `deduplicate_tags()` function for complete tags workflow
  - Added `merge_duplicate_tags()` helper function
  - Integrated tags into the deduplication pipeline

✅ Updated `ritmo_cli/src/commands/mod.rs`:
  - Added `deduplication` module
  - Re-exported all 4 command functions

✅ Updated `ritmo_cli/src/main.rs`:
  - Added 4 new enum variants to `Commands`
  - Added command routing in main match statement
  - All commands use same option pattern

✅ Updated `ritmo_cli/Cargo.toml`:
  - Added `ritmo_ml = { path = "../ritmo_ml" }` dependency

**Output Format**
User-friendly output includes:
- Total entities processed
- Number of duplicate groups found
- Detailed group breakdown with confidence scores
- Primary entity and all duplicates with IDs
- Merge statistics (if auto-merge executed)
- Clear dry-run vs actual merge indicators

**Testing**
Created test library `/tmp/ritmo_ml_test` with intentional duplicates:
- 10 books added with duplicate authors and publishers
- **Authors**: Stephen King (4 variants), J.K. Rowling (3 variants), George R.R. Martin (3 variants)
- **Publishers**: Penguin (4 variants), Bloomsbury (3 variants), Harper Collins (3 variants)
- **Tags**: Fantasy (3 variants: "Fantasy", "fantasy", "FANTASY"), Sci-Fi (3 variants: "Sci-Fi", "Science Fiction", "SciFi")

Test results:
- **deduplicate-publishers** (dry-run): Found 2 duplicate groups with 90.38% and 99.05% confidence
- **deduplicate-publishers** (auto-merge): Successfully merged 2 groups, updated 3 books
- **deduplicate-tags** (dry-run): Found 1 duplicate group (Sci-Fi variants) with 88.85% confidence
- **deduplicate-tags** (auto-merge): Successfully merged 1 group, 6 tags → 4 tags
- **deduplicate-all**: Confirmed no duplicates after merge, now processes 4 entity types
- Database integrity verified: foreign keys updated correctly

**Bug Fixes**
- Fixed `MergeStats` field access error (used correct field names: `books_updated`, `contents_updated`)
- Changed `--dry-run` from default value to flag to allow proper override
- Added safety logic to prevent accidental merges (dry-run=true unless explicitly disabled with auto-merge)

**Files Modified/Created**
- Created: `ritmo_cli/src/commands/deduplication.rs` (~330 lines)
- Modified: `ritmo_cli/src/commands/mod.rs`
- Modified: `ritmo_cli/src/main.rs` (added DeduplicateTags enum + routing)
- Modified: `ritmo_cli/Cargo.toml`
- Modified: `ritmo_ml/src/merge.rs` (added merge_tags + helpers)
- Modified: `ritmo_ml/src/deduplication.rs` (added deduplicate_tags + helper)

**Documentation Updates**
- Updated `CLAUDE.md`: Added Session 12, moved ML CLI from TODO to Recent Changes
- Updated `README.md`: Added ML Deduplication commands section, updated Roadmap
- Updated `docs/ml-system.md`: New "CLI Usage" section with examples and safety recommendations
- Updated `docs/development.md`: Added ML Deduplication commands to reference

**Known Limitations**
- Author deduplication has low detection rate due to complex name parsing (different normalized keys for variants)
- Publisher deduplication works very well with simple normalization (lowercase + trim)
- Series deduplication works well with title normalization
- Tags deduplication works very well (simple normalization: lowercase + alphanumeric only)
- Recommended to start with publishers/series/tags before attempting author deduplication
- Future improvement: Better name normalization for author variants (e.g., "S. King" → "stephen king")

**Next Steps (Not in TODO - for future consideration)**
- Improve author name normalization for better duplicate detection
- Add interactive mode for manual duplicate review
- Export/import deduplication patterns for reuse across libraries
- GUI integration for visual duplicate management

---

## 2026-01-25 - Session 11: ritmo_ml Test Coverage Complete

**Context**
Previously, the `ritmo_ml` crate had 8 test functions that were marked with `#[ignore]` and had empty bodies - they passed only because they did nothing. This session implemented comprehensive, realistic tests for all ML functionality.

**Test Infrastructure Created**
✅ Created `ritmo_ml/src/test_helpers.rs` (270 lines):
  - `create_test_db()`: In-memory SQLite database setup
  - `populate_test_people()`: 12 people with realistic duplicates
    - Stephen King variants: "Stephen King", "Stephen Edwin King", "King, Stephen", "S. King"
    - J.K. Rowling variants: "J.K. Rowling", "Joanne K. Rowling", "J. K. Rowling"
    - George R.R. Martin variants: "George R.R. Martin", "George R. R. Martin", "G.R.R. Martin"
  - `populate_test_publishers()`: 9 publishers with duplicates
    - Penguin variants, HarperCollins variants, Simon & Schuster variants
  - `populate_test_series()`: 8 series with duplicates
    - "The Dark Tower" / "Dark Tower", "Harry Potter" / "Harry Potter Series"
  - `populate_test_tags()`: 8 tags with case duplicates
  - `populate_test_books_with_people()`: Books linked to people for relationship testing
  - `create_full_test_db()`: One-call setup for all entities

**Database Loader Tests (4 tests)**
✅ `test_load_people_from_db`:
  - Loads 12 people from test database
  - Verifies person parsing and normalization
  - Checks all IDs and names are valid

✅ `test_load_publishers_from_db`:
  - Loads 9 publishers
  - Verifies normalization is working

✅ `test_load_series_from_db`:
  - Loads 8 series
  - Verifies title normalization

✅ `test_load_tags_from_db`:
  - Loads 8 tags
  - Verifies label normalization

**Merge Operation Tests (4 tests)**
✅ `test_merge_people`:
  - Merges Stephen King variants (IDs 2, 3, 4 → 1)
  - Verifies people count reduced from 12 to 9
  - Verifies all book relationships point to primary ID
  - Checks duplicate IDs are deleted

✅ `test_merge_publishers`:
  - Merges Penguin/Random House variants
  - Verifies publisher count reduction
  - Checks book publisher_id updates

✅ `test_merge_series`:
  - Merges Dark Tower variants
  - Verifies series count reduction
  - Checks book series_id updates

✅ `test_merge_people_validation_errors`:
  - Tests empty duplicate IDs error
  - Tests primary ID in duplicate list error
  - Tests non-existent ID error

**Deduplication Workflow Tests (2 tests)**
✅ `test_deduplicate_people`:
  - End-to-end deduplication in dry-run mode
  - Verifies duplicate groups are found
  - Checks confidence scores >= threshold
  - Confirms database unchanged in dry-run

✅ `test_deduplicate_people_with_auto_merge`:
  - End-to-end with auto-merge enabled
  - Verifies merges happen automatically
  - Checks people count decreases
  - Validates merge statistics

**Bug Fixes**
- Fixed unused `mut` warning in `deduplication.rs:91`
- Fixed unused import warnings in `db_loaders.rs` and `merge.rs`
- Added missing `x_contents_people_roles` table to test database schema
- Corrected test assertions to match actual struct field names

**Test Results**
- **Before**: 7 real tests + 8 empty/ignored tests = 15 total
- **After**: 17 fully implemented tests, all passing
- **Coverage**: Complete coverage of db_loaders, merge, and deduplication modules
- **Runtime**: All tests run in ~10ms using in-memory databases

**Files Modified/Created**
- Created: `ritmo_ml/src/test_helpers.rs` (270 lines)
- Modified: `ritmo_ml/src/lib.rs` (added test_helpers module)
- Modified: `ritmo_ml/src/db_loaders.rs` (replaced 4 empty tests with real ones)
- Modified: `ritmo_ml/src/merge.rs` (replaced 3 empty tests + added validation test)
- Modified: `ritmo_ml/src/deduplication.rs` (replaced 1 empty test + added auto-merge test)

**Documentation Updates**
- Updated `docs/ml-system.md` with comprehensive Testing section
- Added test infrastructure details and examples
- Documented test data and how to run tests
- Updated implementation history

**Impact**
The `ritmo_ml` crate now has production-ready test coverage with realistic scenarios. All database operations, merge logic, and deduplication workflows are thoroughly tested and verified.

---

_For December 2025 sessions, see [2025-12-sessions.md](2025-12-sessions.md)_
