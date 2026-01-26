# Ritmo Translations

This directory contains translation files for the Ritmo application.

## Supported Languages

- **en** - English (default)
- **it** - Italian

## File Structure

```
locales/
├── en.yml       # English translations
├── it.yml       # Italian translations
└── README.md    # This file
```

## Key Naming Convention

All translation keys follow this pattern:

```
{namespace}.{category}.{subcategory}.{key}
```

### Namespaces

- `cli.*` - Command-line interface strings
- `db.*` - Database system strings (roles, types, formats)
- `error.*` - Error messages
- `gui.*` - GUI interface strings
- `validation.*` - Input validation messages

### Examples

```yaml
cli.app.description: "Ritmo - Library Management System"
db.role.author: "Author"
error.book.not_found: "Book with ID %{id} not found"
gui.sidebar.books: "Books"
validation.date_format: "Invalid date format: '%{input}'. Use YYYY-MM-DD"
```

## Variable Substitution

Translation strings can include variables using `%{variable_name}` (rust-i18n v3 syntax):

```yaml
# English
error.book.not_found: "Book with ID %{id} not found"

# Italian
error.book.not_found: "Libro con ID %{id} non trovato"
```

Usage in code:

```rust
use rust_i18n::t;

let message = t!("error.book.not_found", id = 42);
// English: "Book with ID 42 not found"
// Italian: "Libro con ID 42 non trovato"
```

**Important**: rust-i18n v3 uses `%{variable}` syntax (not `{variable}`).

## Adding New Translations

1. Add the key to **all** language files (en.yml, it.yml)
2. Follow the naming convention
3. Use descriptive keys (not abbreviated)
4. Keep translations consistent across files
5. Use `%{variable}` syntax for variable substitution

### Example - Adding a new error message

```yaml
# en.yml
error:
  import:
    failed: "Import failed: %{reason}"

# it.yml
error:
  import:
    failed: "Importazione fallita: %{reason}"
```

## Best Practices

1. **Always provide both translations** - Never leave a key missing in any language file
2. **Use consistent terminology** - Keep technical terms uniform across translations
3. **Context matters** - Same English word may need different translations in different contexts
4. **Test both languages** - Verify translations work in actual UI/CLI context
5. **Keep formatting** - Preserve emojis, punctuation, and special characters

## Translation Guidelines

### What to Translate

✅ User-facing messages and prompts
✅ CLI help text and descriptions
✅ Error messages
✅ GUI labels and buttons
✅ System value display names (roles, types, formats)

### What NOT to Translate

❌ Code comments
❌ API documentation
❌ Variable/function names
❌ Log messages (debug/trace)
❌ Translation keys themselves

## Contributing Translations

To add a new language:

1. Copy `en.yml` to `{language_code}.yml` (e.g., `fr.yml` for French)
2. Translate all values (keep keys in English)
3. Update this README with the new language
4. Test the translations in the application

## Current Translation Coverage

| Namespace | Keys | Status | Phase |
|-----------|------|--------|-------|
| db.role.* | 6 | ✅ Complete | Phase 1 |
| db.language_role.* | 3 | ✅ Complete | Phase 1 |
| db.type.* | 5 | ✅ Complete | Phase 2 |
| db.format.* | 5 | ✅ Complete | Phase 2 |
| cli.app.* | 2 | ✅ Complete | Phase 1 |
| cli.common.* | 4 | ✅ Complete | Phase 1 |
| error.database.* | 17 | ✅ Complete | Phase 3 |
| error.book.* | 3 | ✅ Complete | Phase 1 |
| error.content.* | 3 | ✅ Complete | Phase 1 |
| error.file.* | 3 | ✅ Complete | Phase 3 |
| error.import.* | 2 | ✅ Complete | Phase 3 |
| error.export.* | 2 | ✅ Complete | Phase 3 |
| error.config.* | 2 | ✅ Complete | Phase 3 |
| error.ml.* | 3 | ✅ Complete | Phase 3 |
| error.validation.* | 4 | ✅ Complete | Phase 3 |
| error.search.* | 2 | ✅ Complete | Phase 3 |
| error.record.* | 2 | ✅ Complete | Phase 3 |
| error.generic.* | 5 | ✅ Complete | Phase 3 |
| gui.* | 13 | ✅ Complete | Phase 1 |

**Phase 1**: 54 keys (Infrastructure, Role, RunningLanguages)
**Phase 2**: 10 keys (Type, Format models)
**Phase 3**: 48 keys (Error messages - all RitmoErr variants)

**Total**: 112 translation keys
**Remaining**: ~350 keys (CLI commands, help text, service messages)
