## 2026-01-28 - Session 25: EPUB OPF Metadata Modification

**Context**
After implementing OPF metadata preservation (Session 24), EPUBs were stored as-is without updating their internal metadata. The user requested that imported EPUBs should have their OPF metadata updated with user-provided data, ensuring consistency between database and EPUB file contents.

**Objective**
Modify EPUB OPF metadata during import to reflect user-provided data (title, authors, publisher, year, ISBN, tags, series, languages) from both manual entry (Level 1) and batch import JSON (Level 2).

**User Requirements** (confirmed via AskUserQuestion):
1. Update **ALL** available metadata fields in OPF
2. None values → preserve original OPF elements (don't remove)
3. Level 2 batch import → aggregate **ALL** people and languages from **ALL** contents
4. Preserve rest of EPUB (cover, manifest, spine, guide)
5. Graceful degradation if modification fails

**Implementation**

✅ **Created OPF Modification Module** (`ritmo_core/src/epub_opf_modifier.rs`, ~500 lines):

**Structures**:
```rust
pub struct OPFMetadata {
    pub title: Option<String>,
    pub creators: Vec<OPFPerson>,       // dc:creator (authors)
    pub contributors: Vec<OPFPerson>,   // dc:contributor (translators, editors)
    pub publisher: Option<String>,
    pub date: Option<String>,            // ISO YYYY-MM-DD
    pub identifiers: Vec<OPFIdentifier>, // ISBN
    pub subjects: Vec<String>,           // tags
    pub languages: Vec<String>,          // ISO 639-1
    pub series: Option<String>,
    pub series_index: Option<f64>,
    pub pages: Option<i64>,
    pub notes: Option<String>,
}

pub struct OPFPerson {
    pub name: String,
    pub role: String,  // OPF MARC code (aut, trl, edt, ill)
}
```

**Core Functions**:

1. **`build_opf_metadata(book_metadata, contents) -> OPFMetadata`**:
   - Aggregates metadata from BookImportMetadata + ALL ContentInputs
   - Maps Ritmo roles to OPF MARC codes:
     - "role.author" → "aut" (creator)
     - "role.translator" → "trl" (contributor)
     - "role.editor" → "edt" (contributor)
     - "role.illustrator" → "ill" (contributor)
   - Deduplicates people by (name, role)
   - Deduplicates languages by code
   - Converts year → ISO date format (YYYY-01-01)

2. **`modify_opf_xml(original_opf: &str, metadata: &OPFMetadata) -> Result<String>`**:
   - Parses original OPF XML (string-based for simplicity)
   - Locates `<metadata>` section
   - Replaces Dublin Core elements:
     - `<dc:title>`
     - `<dc:creator opf:role="aut">` (multiple)
     - `<dc:contributor opf:role="trl|edt|ill">` (multiple)
     - `<dc:publisher>`
     - `<dc:date>`
     - `<dc:identifier opf:scheme="ISBN">`
     - `<dc:subject>` (multiple)
     - `<dc:language>` (multiple)
   - Adds Calibre meta tags:
     - `<meta name="calibre:series">`
     - `<meta name="calibre:series_index">`
   - Preserves manifest, spine, guide sections
   - XML escaping for special characters

3. **`modify_epub_metadata(epub_path, output_path, metadata) -> Result<()>`**:
   - Opens EPUB as ZIP archive
   - Finds OPF path from META-INF/container.xml
   - Extracts original OPF content
   - Modifies OPF XML
   - Creates new ZIP file:
     - Copies all files except OPF
     - Writes modified OPF with same path
     - Preserves compression and permissions
   - Atomic operation (temp file → rename)

✅ **Integration in `book_import_service.rs`**:

**Changes**:
- Renamed `import_book()` → `import_book_with_contents(contents: &[ContentInput])`
- Added imports: `use crate::epub_opf_modifier;` and `use crate::dto::ContentInput;`
- Modified import workflow (lines 120-226):

```rust
// Step 1: Build OPF metadata BEFORE creating Book struct (to avoid move issues)
let opf_metadata = epub_opf_modifier::build_opf_metadata(&metadata, contents);

// ... create Book struct, save to database ...

// Step 8: Extract and save original OPF (backup)
// ... existing code ...

// Step 9: Modify EPUB with user metadata
if extension == "epub" {
    let temp_epub = storage_path.with_extension("epub.tmp");

    match epub_opf_modifier::modify_epub_metadata(file_path, &temp_epub, &opf_metadata) {
        Ok(_) => fs::rename(&temp_epub, &storage_path)?,
        Err(e) => {
            eprintln!("Warning: Could not modify EPUB metadata: {:?}", e);
            let _ = fs::remove_file(&temp_epub);
            fs::copy(file_path, &storage_path)?;
        }
    }
} else {
    fs::copy(file_path, &storage_path)?;
}
```

**Backward Compatibility**:
- Created wrapper `import_book()` that calls `import_book_with_contents(&[])`
- Level 1 (manual import) uses wrapper → empty contents array
- Level 2 (batch import) uses `import_book_with_contents()` directly

✅ **Integration in `batch_import_service.rs`**:

**Changes**:
- Updated imports: `use crate::service::book_import_service::import_book_with_contents;`
- Modified `import_single()` to pass contents:

```rust
let book_id = import_book_with_contents(
    config,
    pool,
    &file_path,
    book_metadata,
    &import_obj.contents,  // Pass ALL contents for OPF aggregation
)
.await?;
```

✅ **Dependencies**:
- Added `quick-xml = "0.36"` to `ritmo_core/Cargo.toml`

**Testing**

✅ **Level 1 Import Test**:
- Imported EPUB with custom metadata:
  - Title: "Fotografia Digitale Test" (instead of original "Fotografia digitale For Dummies")
  - Author: "Julie King" (instead of "Julie Adair King")
  - Publisher: "Test Publisher" (instead of "Hoepli")
  - Year: 2024
  - Tags: "fotografia,test"

✅ **Verification**:
- **Modified EPUB** (`storage/books/.../...epub`):
  ```xml
  <dc:title>Fotografia Digitale Test</dc:title>
  <dc:creator opf:role="aut">Julie King</dc:creator>
  <dc:publisher>Test Publisher</dc:publisher>
  <dc:date>2024-01-01</dc:date>
  <dc:subject>fotografia,test</dc:subject>
  ```

- **Original OPF Backup** (`storage/originals_opf/.../...opf.xml`):
  ```xml
  <dc:title>Fotografia digitale For Dummies (Foto, cinema e televisione) (Italian Edition)</dc:title>
  <dc:creator opf:file-as="King, Julie Adair">Julie Adair King</dc:creator>
  <dc:publisher>Hoepli</dc:publisher>
  ```

✅ **Compilation**: Full workspace builds successfully without errors

**Metadata Mapping**

| Ritmo Field | OPF Element | OPF Role | Element Type |
|-------------|-------------|----------|--------------|
| `title` | `<dc:title>` | - | Single |
| `people[role.author]` | `<dc:creator>` | `opf:role="aut"` | Multiple |
| `people[role.translator]` | `<dc:contributor>` | `opf:role="trl"` | Multiple |
| `people[role.editor]` | `<dc:contributor>` | `opf:role="edt"` | Multiple |
| `people[role.illustrator]` | `<dc:contributor>` | `opf:role="ill"` | Multiple |
| `publisher` | `<dc:publisher>` | - | Single |
| `year` | `<dc:date>` | - | YYYY-01-01 |
| `isbn` | `<dc:identifier>` | `opf:scheme="ISBN"` | Single |
| `tags[]` | `<dc:subject>` | - | Multiple |
| `contents[].languages[]` | `<dc:language>` | - | Multiple (ISO 639-1) |
| `series` | `<meta name="calibre:series">` | - | Calibre meta |
| `series_index` | `<meta name="calibre:series_index">` | - | Calibre meta |

**Error Handling**

| Scenario | Behavior | Impact |
|----------|----------|---------|
| XML parsing error | Log warning, copy original EPUB | Database correct, EPUB unmodified |
| ZIP corruption | Log warning, copy original EPUB | Database correct, EPUB unmodified |
| OPF not found | Log warning, copy original EPUB | Database correct, EPUB unmodified |
| Write permission error | Return error, abort import | Import fails (expected) |

**Graceful Degradation**:
- Modification failure doesn't abort import
- Original EPUB copied as fallback
- Database metadata always correct (source of truth)
- Original OPF always preserved for recovery

**Benefits**

1. **Consistency**: EPUBs in storage match database metadata
2. **Correctness**: User-verified data replaces potentially incorrect original metadata
3. **Level 2 Aggregation**: Batch import with contents automatically aggregates all contributors and languages into EPUB
4. **Recovery**: Original OPF preserved for comparison/recovery
5. **Future-Ready**: Modified EPUBs can be exported with correct metadata

**Files Created**
- Created: `ritmo_core/src/epub_opf_modifier.rs` (~500 lines)

**Files Modified**
- Modified: `ritmo_core/Cargo.toml` (added quick-xml dependency)
- Modified: `ritmo_core/src/lib.rs` (exported epub_opf_modifier module)
- Modified: `ritmo_core/src/service/book_import_service.rs` (renamed function, added OPF modification workflow, wrapper for backward compatibility)
- Modified: `ritmo_core/src/service/batch_import_service.rs` (updated to use import_book_with_contents)
- Modified: `docs/architecture.md` (added "EPUB OPF Metadata Modification" section)
- Modified: `CLAUDE.md` (added Session 25 summary)

**Statistics**
- Total changes: 5 files modified, 1 file created
- New module: epub_opf_modifier.rs (~500 lines)
- New structures: 3 (OPFMetadata, OPFPerson, OPFIdentifier)
- New functions: 6 (build_opf_metadata, modify_opf_xml, modify_epub_metadata, role mapping, etc.)
- Test coverage: Unit tests for role mapping, XML escaping, metadata building
- Dependency added: quick-xml 0.36

**Impact**

Session 25 completion means:
- ✅ EPUBs stored in ritmo have correct, user-verified metadata
- ✅ Database and EPUB contents are consistent
- ✅ Level 2 batch import automatically aggregates all content metadata
- ✅ Original OPF preserved for recovery and comparison
- ✅ Graceful degradation ensures import reliability
- ✅ Foundation ready for Level 3 auto-extraction validation

**Design Decisions**

1. **String-based XML modification**: Simpler than full DOM parsing, sufficient for metadata section
2. **Atomic operations**: Temp file + rename prevents corrupted EPUBs
3. **Build metadata early**: Before Book struct creation to avoid ownership issues
4. **Wrapper for backward compatibility**: Level 1 continues working without changes
5. **Graceful degradation**: Modification failures don't abort import (database is source of truth)
6. **MARC relator codes**: Standard role codes for interoperability (aut, trl, edt, ill)

**Known Limitations**

- Only modifies EPUB files (PDF, MOBI unchanged)
- String-based XML parsing (less robust than DOM but simpler)
- No validation of modified EPUB against EPUB spec
- OPF files not tracked in database (must derive path from hash)

**Related Sessions**
- Session 24 (2026-01-28): OPF Metadata Preservation (foundation for backup)
- Session 23 (2026-01-27): Hash-Based Storage System (file path structure)
- Session 21 (2026-01-27): Batch Import Implementation (Level 2 integration)
- Future: Level 3 Automatic Metadata Extraction (will validate against modified OPF)

---

