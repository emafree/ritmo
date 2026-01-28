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

