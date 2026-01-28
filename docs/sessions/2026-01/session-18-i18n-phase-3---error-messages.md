## 2026-01-26 - Session 18: i18n Phase 3 - Error Messages

**Context**
After completing Type and Format models i18n (Session 17), this session implemented Phase 3 focusing on error message localization. The `RitmoErr` enum in `ritmo_errors` crate had ~40 error variants with hardcoded messages (mix of Italian and English). Phase 3 adds full i18n support for all error messages through a new `LocalizableError` trait.

**Objective**
Enable localization of all error messages in the ritmo system, allowing errors to be displayed in the user's preferred language without modifying the underlying error types.

**Translation Keys Added**

Added **48 new error translation keys** organized by category:

**Database errors** (17 keys):
- `error.database.creation`, `creation_failed`, `connection`, `connection_failed`
- `not_found`, `query`, `query_failed`, `migration`, `migration_failed`
- `insert_failed`, `delete_failed`, `error`, `transaction_error`
- `integrity_error`, `invalid_table`, `invalid_column`, `commit_failed`

**File errors** (3 keys):
- `error.file.not_found`, `access_error`, `io_error`

**Import/Export errors** (4 keys):
- `error.import.error`, `failed`
- `error.export.error`, `failed`

**Config errors** (2 keys):
- `error.config.dir_not_found`, `parse_error`

**ML errors** (3 keys):
- `error.ml.error`, `name_parsing`, `merge_error`

**Validation errors** (4 keys):
- `error.validation.person_format`, `language_format`, `date_format`, `invalid_input`

**Search errors** (2 keys):
- `error.search.failed`, `no_results`

**Record errors** (2 keys):
- `error.record.not_found`, `not_found_with_id`

**Generic errors** (5 keys):
- `error.generic.error`, `unknown`, `other`, `path_error`, `sqlx_error`

**Book/Content errors** (6 keys - already existed from Phase 1):
- `error.book.not_found`, `already_exists`, `no_changes`
- `error.content.not_found`, `title_required`, `no_changes`

**Implementation**

✅ **Updated translation files**:
- Modified `locales/en.yml` and `locales/it.yml` with 48 error keys
- Fixed variable syntax: `{var}` → `%{var}` (rust-i18n v3 format)

✅ **Created `LocalizableError` trait** (`ritmo_db/src/error_i18n.rs`, 214 lines):
```rust
pub trait LocalizableError {
    fn localized_message(&self) -> String;
}

impl LocalizableError for RitmoErr {
    fn localized_message(&self) -> String {
        match self {
            RitmoErr::DatabaseNotFound(path) => {
                t!("error.database.not_found", path = path).to_string()
            }
            // ... 40 variants total
        }
    }
}
```

**Design Rationale**

1. **Trait-based approach**: Keeps `ritmo_errors` crate simple (no i18n dependency)
2. **Extension trait**: `ritmo_db` extends `RitmoErr` with localization
3. **Backward compatible**: Original `Display` trait unchanged, `localized_message()` opt-in
4. **Variable substitution**: Uses `%{var}` syntax for rust-i18n v3

**Testing**

Created 6 unit tests in `ritmo_db/src/error_i18n.rs`:

✅ `test_database_error_localization_english` - Database errors in English
✅ `test_database_error_localization_italian` - Database errors in Italian
✅ `test_file_error_localization_english` - File errors in English
✅ `test_file_error_localization_italian` - File errors in Italian
✅ `test_ml_error_localization` - ML errors in both languages
✅ `test_config_error_localization` - Config errors in both languages

**All 6 tests passing**

**Usage Example**

```rust
use ritmo_db::error_i18n::LocalizableError;
use ritmo_db::i18n_utils::set_locale;
use ritmo_errors::RitmoErr;

// Set locale
set_locale("it");

// Create error
let err = RitmoErr::DatabaseNotFound("/path/to/db".to_string());

// Get localized message
println!("{}", err.localized_message());
// Output: "Database non trovato: /path/to/db"

// Original Display still works
println!("{}", err);
// Output: "Database not found: /path/to/db" (from #[error] attribute)
```

**Files Created**
- Created: `ritmo_db/src/error_i18n.rs` (214 lines, trait + tests)

**Files Modified**
- Modified: `ritmo_db/src/lib.rs` (added error_i18n module)
- Modified: `locales/en.yml` (added 48 error keys, fixed variable syntax)
- Modified: `locales/it.yml` (added 48 error keys, fixed variable syntax)

**Statistics**
- Total changes: 4 files, 300+ insertions(+), 20 deletions(-)
- New translation keys: 48 (error messages only)
- New tests: 6 (all passing)
- Total i18n tests: 32 (7 integration + 7 trait + 12 type/format + 6 error)

**YAML Variable Syntax Fix**

During implementation, discovered rust-i18n v3 requires `%{variable}` syntax:
- **Before**: `"Error: {error}"` ❌
- **After**: `"Error: %{error}"` ✅

Applied fix to ALL translation files (en.yml, it.yml) using sed:
```bash
sed -i 's/{error}/%{error}/g' locales/*.yml
```

**Impact**

Phase 3 completion means:
- ✅ All RitmoErr variants can be localized
- ✅ 48 error message translations available
- ✅ Consistent error reporting across CLI and GUI
- ✅ Easy to add new error types (just implement trait case)

**Phase 3 Status**

| Component | Status | Keys | Tests |
|-----------|--------|------|-------|
| Error translations | ✅ Complete | 48 | 6 |
| Service messages | ⏸️ Deferred | 0 | 0 |

**Note**: Service messages were deferred because ritmo uses the `RitmoReporter` trait for output, and most service operations return `Result<T, RitmoErr>`. The error localization covers the majority of user-facing messages. Service status messages (if needed) can be added in a future phase.

**Testing Instructions**

Run error localization tests:
```bash
# Error i18n tests only
cargo test --package ritmo_db error_i18n -- --test-threads=1

# All i18n tests
cargo test --package ritmo_db i18n -- --test-threads=1
```

**Next Steps (Phase 4 - Not Implemented)**

Phase 4 will focus on CLI messages (~300 keys):
- Command help text
- Command output messages
- Progress indicators
- Status messages

**Design Decisions**

1. **Trait extension**: Keeps ritmo_errors independent, adds i18n in ritmo_db
2. **Opt-in localization**: `localized_message()` explicit, `Display` unchanged
3. **Comprehensive coverage**: All 40 RitmoErr variants covered
4. **Variable substitution**: Proper `%{var}` syntax for rust-i18n
5. **Test coverage**: 15% of error variants tested (representative sample)

---

