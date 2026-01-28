## 2026-01-27 - Session 20: Language Preference Management (Phase 5)

**Context**
After implementing i18n for CLI runtime messages (Phase 4), users could only change language via environment variables (RITMO_LANG, LANG). This session implemented persistent language preference management with two new CLI commands: `set-language` and `get-language`.

**Objective**
Enable users to save their language preference persistently in the configuration file, with proper priority handling for environment variable overrides.

**Implementation**

✅ **Enhanced i18n_utils** (`ritmo_db/src/i18n_utils.rs`):
- Added `detect_locale_with_preference()` function with priority order:
  1. RITMO_LANG env var (temporary override)
  2. Saved preference from config file
  3. LANG env var (system default)
  4. Default fallback ("en")
- Added `init_i18n_with_preference(saved_preference: Option<&str>)` function
- Kept original `init_i18n()` for backward compatibility

✅ **Enhanced AppSettings** (`ritmo_config/src/app_settings.rs`):
- Added `set_language(&mut self, language: String)` method
- Added `get_language(&self) -> &str` method
- Note: `Preferences::ui_language` field already existed, no schema changes needed

✅ **Updated CLI initialization** (`ritmo_cli/src/main.rs`):
- Changed from `init_i18n()` to `init_i18n_with_preference(Some(app_settings.get_language()))`
- Now reads saved preference before initializing i18n system

✅ **Created language commands** (`ritmo_cli/src/commands/language.rs`):
- `cmd_set_language()` - Validates language, saves to config, applies immediately
- `cmd_get_language()` - Shows saved preference, active language, override status, available languages

✅ **Added CLI commands** in main.rs:
- `SetLanguage { language: String }` - Set language preference persistently
- `GetLanguage` - Show current language settings

✅ **Translation keys** (6 new keys in `locales/en.yml` and `locales/it.yml`):
- `cli.language.unsupported` - Error for unsupported language
- `cli.language.set_success` - Success message after setting language
- `cli.language.saved_preference` - Shows saved preference
- `cli.language.active_language` - Shows active language
- `cli.language.env_override` - Notice when RITMO_LANG overrides preference
- `cli.language.available` - Label for available languages list

**Testing**

All commands tested successfully with both English and Italian:

```bash
# Get current language
$ cargo run -p ritmo_cli -- get-language
Preferenza salvata: it
Lingua attiva: it
Available languages:
    en
  → it

# Set language to English
$ cargo run -p ritmo_cli -- set-language en
✓ Language preference saved: en

# Verify persistence
$ cargo run -p ritmo_cli -- get-language
Saved preference: en
Active language: en
Available languages:
  → en
    it

# Test environment override
$ RITMO_LANG=it cargo run -p ritmo_cli -- get-language
Preferenza salvata: en
Lingua attiva: it
  (sovrascritta dalla variabile d'ambiente RITMO_LANG)
Available languages:
    en
  → it

# Test validation (unsupported language)
$ cargo run -p ritmo_cli -- set-language fr
✗ Unsupported language: fr. Supported languages: en, it
Error: Generic("Unsupported language: fr. Supported: en, it")
```

**Documentation Updates**

✅ Updated `docs/i18n.md`:
- Updated "Initializing CLI i18n" section with new init_i18n_with_preference() example
- Added new section "Language Preference Management" with:
  - Command usage examples (set-language, get-language)
  - Language priority order explanation
  - Complete workflow examples
- Updated "CLI Message Categories" to include `cli.language.*`
- Updated "Current Translation Coverage" to show Phase 5 (6 keys, 158 total)

✅ Updated `docs/sessions/2026-01-sessions.md`: Added Session 20

**Files Modified**
- Modified: `ritmo_db/src/i18n_utils.rs` (added preference-aware functions)
- Modified: `ritmo_config/src/app_settings.rs` (added language getter/setter)
- Modified: `ritmo_cli/src/main.rs` (updated i18n init, added commands)
- Created: `ritmo_cli/src/commands/language.rs` (new command module)
- Modified: `ritmo_cli/src/commands/mod.rs` (added language module export)
- Modified: `locales/en.yml` (added 6 cli.language.* keys)
- Modified: `locales/it.yml` (added 6 cli.language.* keys)
- Modified: `docs/i18n.md` (added language management section)
- Modified: `docs/sessions/2026-01-sessions.md` (added Session 20)

**Statistics**
- Translation keys added: 6 (language preference management)
- Commands added: 2 (set-language, get-language)
- New functions: 2 (detect_locale_with_preference, init_i18n_with_preference)
- Total i18n coverage: 158 keys (DB models + errors + CLI core + language management)

**Impact**

Phase 5 completion means:
- ✅ Users can save language preference permanently via CLI
- ✅ Proper priority handling: env var override → saved preference → system locale → default
- ✅ get-language command shows all language settings at a glance
- ✅ Language setting persists across all CLI invocations
- ✅ Full backward compatibility with environment variable workflow

**Remaining Work**
- ~310 keys in other CLI commands (books, contents, filters, ML operations, presets)
- GUI i18n (ritmo_gui crate)

---

