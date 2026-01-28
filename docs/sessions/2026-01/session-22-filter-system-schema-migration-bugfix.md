## 2026-01-27 - Session 22: Filter System Schema Migration Bugfix

**Context**
After Session 17 (i18n Phase 2), the database schema was changed to use canonical i18n keys instead of translated strings for `formats` and `types` tables (changing `name` columns to `key` columns). However, the filter system queries and result structures were not updated, causing SQL errors when listing books or contents.

**Problem**
- Commands `list-books` and `list-contents` failed with: `Error: Database(SqliteError { code: 1, message: "no such column: formats.name" })`
- The filter system was still referencing the old column names (`formats.name`, `types.name`) instead of the new ones (`formats.key`, `types.key`)

**Objective**
Update the filter system to align with the new i18n-based schema, ensuring all queries and result structures use the correct column names.

**Implementation**

✅ **ritmo_db_core/src/filters/builder.rs**:
- Updated `build_books_query()` to use `formats.key as format_key` instead of `formats.name as format_name`
- Updated format filter clause to use `formats.key` instead of `formats.name`
- Updated `build_contents_query()` to use `types.key as type_key` instead of `types.name as type_name`
- Updated content type filter clause to use `types.key` instead of `types.name`

✅ **ritmo_db_core/src/filters/types.rs**:
- Changed `BookResult.format_name` → `format_key`
- Changed `ContentResult.type_name` → `type_key`
- Updated `ContentSortField::to_sql()` to return `types.key` instead of `types.name` for Type sorting
- Updated `ContentResult::to_short_string()` to use `type_key`
- Updated test data in `test_book_result_formatting()` and `test_content_result_formatting()` to use key-based values

✅ **ritmo_cli/src/formatter.rs**:
- Updated `format_books_table()` to use `book.format_key` instead of `book.format_name`
- Updated `format_books_simple()` to use `book.format_key` instead of `book.format_name`
- Updated `format_contents_table()` to use `content.type_key` instead of `content.type_name`
- Updated `format_contents_simple()` to use `content.type_key` instead of `content.type_name`

**Testing**:
- ✅ `cargo build --workspace` - successful compilation
- ✅ `cargo test -p ritmo_db_core --lib filters` - all 20 filter tests passing
- ✅ `list-books` command - displays books with `format_key` (e.g., "epub")
- ✅ `list-contents` command - displays contents with `type_key`
- ✅ `list-books --output json` - correct JSON output with `format_key` field
- ✅ `list-contents --output json` - correct JSON output with `type_key` field

**Files Modified**:
- `ritmo_db_core/src/filters/builder.rs` (4 changes)
- `ritmo_db_core/src/filters/types.rs` (6 changes)
- `ritmo_cli/src/formatter.rs` (4 changes)

**Outcome**:
Both `list-books` and `list-contents` commands now work correctly with all output formats (table, JSON, simple). The filter system is fully aligned with the i18n schema using canonical keys instead of translated strings.

**Related Sessions**:
- Session 17 (2026-01-26): i18n Phase 2 - Type and Format Models (introduced the schema change)
- Session 15 (2026-01-26): i18n Infrastructure Implementation (foundation)

---

