# Filter Preset System - Usage Guide

## Overview

The preset system allows you to save commonly used filter combinations and reuse them quickly. Presets are stored globally in `~/.config/ritmo/settings.toml` and are available across all your libraries.

## Benefits

- **Save time**: No need to type the same filters repeatedly
- **Consistency**: Use the same filters across sessions
- **Easy sharing**: Export your settings.toml to share presets
- **Combine with CLI**: Override preset values with CLI parameters

## Commands

### Save a Preset

```bash
# Save a preset for books
ritmo save-preset books \
  --name "my_ebooks" \
  --format epub \
  --description "All my EPUB books"

# Save a preset with multiple filters
ritmo save-preset books \
  --name "calvino_novels" \
  --author "Calvino" \
  --format epub \
  --sort year \
  --description "Calvino's novels in EPUB format"

# Save a preset for contents
ritmo save-preset contents \
  --name "italian_novels" \
  --content-type "Romanzo" \
  --description "Italian novels only"

# Complex preset with all options
ritmo save-preset books \
  --name "recent_tech" \
  --author "Martin" \
  --publisher "O'Reilly" \
  --year 2023 \
  --format pdf \
  --sort date_added \
  --limit 20 \
  --description "Recent tech books from O'Reilly"
```

### List Presets

```bash
# List all presets (books and contents)
ritmo list-presets

# List only book presets
ritmo list-presets books

# List only content presets
ritmo list-presets contents
```

**Example Output:**
```
Preset per Libri:
--------------------------------------------------
• my_ebooks
  Descrizione: All my EPUB books
  Filtri: formato=epub, ordina=title

• calvino_novels
  Descrizione: Calvino's novels in EPUB format
  Filtri: autore=Calvino, formato=epub, ordina=year


Preset per Contenuti:
--------------------------------------------------
• italian_novels
  Descrizione: Italian novels only
  Filtri: tipo=Romanzo, ordina=title
```

### Use a Preset

```bash
# Apply a book preset
ritmo list-books --preset my_ebooks

# Apply a book preset with short flag
ritmo list-books -p my_ebooks

# Apply a content preset
ritmo list-contents --preset italian_novels
ritmo list-contents -p italian_novels
```

### Override Preset Values

CLI parameters always override preset values:

```bash
# Use preset but override format
ritmo list-books --preset my_ebooks --format pdf

# Use preset but override sort and limit
ritmo list-books --preset calvino_novels --sort title --limit 5

# Use preset but add additional filter
ritmo list-books --preset my_ebooks --year 2023
```

**Priority Order:**
1. Explicit CLI parameters (highest)
2. Preset values
3. Default values (lowest)

### Delete a Preset

```bash
# Delete a book preset
ritmo delete-preset books my_ebooks

# Delete a content preset
ritmo delete-preset contents italian_novels
```

## Practical Examples

### Example 1: Quick Access to Favorite Collections

```bash
# Create presets for your favorite collections
ritmo save-preset books --name "scifi" --search "fantascienza" --format epub
ritmo save-preset books --name "tech" --publisher "O'Reilly" --format pdf
ritmo save-preset books --name "italian" --search "italiano"

# Quick access
ritmo list-books -p scifi
ritmo list-books -p tech
ritmo list-books -p italian
```

### Example 2: Different Views of Same Data

```bash
# Create different sorting presets
ritmo save-preset books --name "by_author" --sort author --limit 50
ritmo save-preset books --name "by_year" --sort year --limit 50
ritmo save-preset books --name "recent" --sort date_added --limit 20

# Switch views quickly
ritmo list-books -p by_author
ritmo list-books -p by_year
ritmo list-books -p recent
```

### Example 3: Research Workflows

```bash
# Create research-specific presets
ritmo save-preset books \
  --name "ai_research" \
  --search "artificial intelligence" \
  --year 2023 \
  --format pdf

ritmo save-preset books \
  --name "ml_papers" \
  --search "machine learning" \
  --format pdf \
  --sort year

# Use in research
ritmo list-books -p ai_research -o json > ai_research.json
ritmo list-books -p ml_papers --limit 10
```

### Example 4: Combined with Output Formats

```bash
# Export specific collection to JSON
ritmo list-books -p my_ebooks -o json > ebooks.json

# Quick overview in simple format
ritmo list-books -p calvino_novels -o simple

# Detailed table for printing
ritmo list-books -p recent_tech -o table
```

## Preset Storage

Presets are stored in your global settings file:
- **Linux/Mac**: `~/.config/ritmo/settings.toml`
- **Windows**: `%APPDATA%/ritmo/settings.toml`

### Example settings.toml with Presets

```toml
last_library_path = "/home/user/RitmoLibrary"
recent_libraries = ["/home/user/RitmoLibrary", "/media/usb/Books"]

[preferences]
ui_language = "it"
ui_theme = "light"

[presets.books.my_ebooks]
name = "my_ebooks"
description = "All my EPUB books"

[presets.books.my_ebooks.filters]
format = "epub"
sort = "title"
offset = 0

[presets.books.calvino_novels]
name = "calvino_novels"
description = "Calvino's novels in EPUB format"

[presets.books.calvino_novels.filters]
author = "Calvino"
format = "epub"
sort = "year"
offset = 0

[presets.contents.italian_novels]
name = "italian_novels"
description = "Italian novels only"

[presets.contents.italian_novels.filters]
content_type = "Romanzo"
sort = "title"
offset = 0
```

## Tips and Best Practices

### 1. Use Descriptive Names
```bash
# Good names
ritmo save-preset books --name "unread_scifi" --search "fantascienza"
ritmo save-preset books --name "tech_2023" --year 2023

# Avoid generic names
ritmo save-preset books --name "preset1" --format epub  # Too vague
```

### 2. Add Descriptions
```bash
# Always add descriptions for complex presets
ritmo save-preset books \
  --name "research_ai" \
  --search "artificial intelligence" \
  --year 2023 \
  --description "AI research papers from 2023"
```

### 3. Create Specific and General Presets
```bash
# General preset for all EPUBs
ritmo save-preset books --name "all_epub" --format epub

# Specific preset for recent EPUBs
ritmo save-preset books --name "recent_epub" --format epub --sort date_added --limit 20
```

### 4. Use Presets with Pagination
```bash
# Create a preset with limit
ritmo save-preset books --name "browse" --limit 20

# Page through results
ritmo list-books -p browse --offset 0
ritmo list-books -p browse --offset 20
ritmo list-books -p browse --offset 40
```

### 5. Export and Share Presets

```bash
# Export your presets
cp ~/.config/ritmo/settings.toml ~/my_ritmo_presets.toml

# Share with colleague (they can merge into their settings.toml)
```

## Common Workflows

### Daily Reading List
```bash
# Setup
ritmo save-preset books --name "to_read" --format epub --limit 10

# Daily use
ritmo list-books -p to_read
```

### Research Organization
```bash
# By topic
ritmo save-preset books --name "ai" --search "artificial intelligence"
ritmo save-preset books --name "ml" --search "machine learning"
ritmo save-preset books --name "nlp" --search "natural language"

# Quick access
ritmo list-books -p ai
ritmo list-books -p ml
ritmo list-books -p nlp
```

### Format-Specific Views
```bash
# Different devices
ritmo save-preset books --name "kindle" --format mobi
ritmo save-preset books --name "tablet" --format epub
ritmo save-preset books --name "desktop" --format pdf

# Transfer to device
ritmo list-books -p kindle -o json > kindle_books.json
```

## Future Enhancements (Planned)

### Library-Specific Presets (Phase 2)
In the future, presets will also be stored per-library in `library/config/filters.toml`:
- **Portable**: Travel with the library when copied
- **Library-specific defaults**: Each library can have its own default view
- **Resolution order**: Library presets override global presets

### Auto-Save Last Filter (Phase 3)
- Automatically save the last used filter combination
- Quickly restore with `--use-last` flag
- Clear saved filter with `--clear-filters`

## Troubleshooting

### Preset Not Found
```bash
$ ritmo list-books -p nonexistent
✗ Preset 'nonexistent' non trovato

# Solution: Check available presets
$ ritmo list-presets books
```

### Wrong Preset Type
```bash
$ ritmo list-books -p italian_novels  # This is a contents preset!
✗ Preset 'italian_novels' non trovato

# Solution: Use correct command
$ ritmo list-contents -p italian_novels
```

### Overwriting Existing Preset
When you save a preset with an existing name, it will be overwritten:
```bash
$ ritmo save-preset books --name "my_ebooks" --format pdf
✓ Preset 'my_ebooks' salvato per libri

# Previous "my_ebooks" preset is now replaced
```

## See Also

- [Filter System Usage Guide](filter-system-usage.md) - Complete filter documentation
- [CLAUDE.md](../CLAUDE.md) - Full project documentation
- Preset architecture in CLAUDE.md "Filter Preset System" section
