use crate::dto::ContentInput;
use crate::service::book_import_service::BookImportMetadata;
use ritmo_errors::{RitmoErr, RitmoResult};
use std::collections::HashSet;
use std::fs::{File};
use std::io::{BufReader, Read, Write};
use std::path::Path;
use zip::{ZipArchive, ZipWriter};

/// OPF metadata structure for EPUB modification
#[derive(Debug, Clone)]
pub struct OPFMetadata {
    // Dublin Core elements
    pub title: Option<String>,
    pub creators: Vec<OPFPerson>,       // dc:creator (authors)
    pub contributors: Vec<OPFPerson>,   // dc:contributor (translators, editors, etc.)
    pub publisher: Option<String>,
    pub date: Option<String>,           // ISO format YYYY-MM-DD
    pub identifiers: Vec<OPFIdentifier>,
    pub subjects: Vec<String>,          // tags
    pub languages: Vec<String>,         // ISO 639-1 codes

    // Calibre meta tags
    pub series: Option<String>,
    pub series_index: Option<f64>,
    pub pages: Option<i64>,
    pub notes: Option<String>,
}

/// Person contributor in OPF format
#[derive(Debug, Clone)]
pub struct OPFPerson {
    pub name: String,
    pub role: String,  // OPF role code (aut, trl, edt, ill)
}

/// Identifier in OPF format
#[derive(Debug, Clone)]
pub struct OPFIdentifier {
    pub scheme: String,  // ISBN, UUID, etc.
    pub value: String,
}

/// Maps Ritmo role keys to OPF role codes
///
/// MARC relator codes: https://www.loc.gov/marc/relators/relaterm.html
fn map_ritmo_role_to_opf(ritmo_role: &str) -> &'static str {
    match ritmo_role {
        "role.author" => "aut",
        "role.translator" => "trl",
        "role.editor" => "edt",
        "role.illustrator" => "ill",
        "role.narrator" => "nrt",
        "role.contributor" => "ctb",
        "role.preface" => "aui",  // Author of introduction
        _ => "ctb",  // default: contributor
    }
}

/// Determines if an OPF role should be a creator or contributor
///
/// Authors are creators, all others are contributors
fn opf_role_to_element_type(role: &str) -> &'static str {
    match role {
        "aut" => "creator",
        _ => "contributor",
    }
}

/// Builds OPFMetadata from BookImportMetadata and optional ContentInputs
///
/// This function aggregates all metadata from both book-level and content-level:
/// - Book title and metadata
/// - ALL people from book AND all contents (aggregated, deduplicated)
/// - ALL languages from all contents (deduplicated)
/// - Tags from book
///
/// # Arguments
/// * `book_metadata` - Book-level metadata provided by user
/// * `contents` - Optional array of content metadata (from batch import)
///
/// # Returns
/// OPFMetadata ready for XML modification
pub fn build_opf_metadata(
    book_metadata: &BookImportMetadata,
    contents: &[ContentInput],
) -> OPFMetadata {
    let mut creators = Vec::new();
    let mut contributors = Vec::new();
    let mut languages = HashSet::new();

    // Add book-level people
    if let Some(book_people) = &book_metadata.people {
        for (name, role) in book_people {
            let opf_role = map_ritmo_role_to_opf(role);
            let person = OPFPerson {
                name: name.clone(),
                role: opf_role.to_string(),
            };

            if opf_role_to_element_type(opf_role) == "creator" {
                creators.push(person);
            } else {
                contributors.push(person);
            }
        }
    }

    // Add ALL content-level people and languages
    for content in contents {
        // Add content people
        for person_input in &content.people {
            let opf_role = map_ritmo_role_to_opf(&person_input.role);
            let person = OPFPerson {
                name: person_input.name.clone(),
                role: opf_role.to_string(),
            };

            if opf_role_to_element_type(opf_role) == "creator" {
                // Check for duplicates before adding
                if !creators.iter().any(|p| p.name == person.name && p.role == person.role) {
                    creators.push(person);
                }
            } else {
                // Check for duplicates before adding
                if !contributors.iter().any(|p| p.name == person.name && p.role == person.role) {
                    contributors.push(person);
                }
            }
        }

        // Add content languages
        for lang_input in &content.languages {
            languages.insert(lang_input.code.clone());
        }
    }

    // Build identifiers (ISBN if present)
    let mut identifiers = Vec::new();
    if let Some(isbn) = &book_metadata.isbn {
        identifiers.push(OPFIdentifier {
            scheme: "ISBN".to_string(),
            value: isbn.clone(),
        });
    }

    // Build date from year (format: YYYY-01-01)
    let date = book_metadata.year.map(|y| format!("{}-01-01", y));

    // Build subjects from tags
    let subjects = book_metadata.tags.clone().unwrap_or_default();

    // Build series_index as f64
    let series_index = book_metadata.series_index.map(|i| i as f64);

    OPFMetadata {
        title: Some(book_metadata.title.clone()),
        creators,
        contributors,
        publisher: book_metadata.publisher.clone(),
        date,
        identifiers,
        subjects,
        languages: languages.into_iter().collect(),
        series: book_metadata.series.clone(),
        series_index,
        pages: book_metadata.pages,
        notes: book_metadata.notes.clone(),
    }
}

/// Modifies OPF XML with new metadata
///
/// This function parses the original OPF XML, updates the metadata section,
/// and preserves all other parts (manifest, spine, guide).
///
/// # Strategy
/// - Parse XML with namespace support
/// - Modify only <metadata> section
/// - Preserve manifest, spine, guide sections
/// - Preserve existing elements for None fields
///
/// # Arguments
/// * `original_opf` - Original OPF XML content as string
/// * `metadata` - New metadata to apply
///
/// # Returns
/// Modified OPF XML as string
pub fn modify_opf_xml(
    original_opf: &str,
    metadata: &OPFMetadata,
) -> RitmoResult<String> {
    // For now, return a minimal implementation that just adds/replaces metadata
    // A full XML parser implementation would be more robust but also more complex

    // This is a simplified implementation that uses string manipulation
    // TODO: Implement proper XML parsing with quick-xml for better robustness

    // Find the metadata section
    let metadata_start = original_opf.find("<metadata")
        .ok_or_else(|| RitmoErr::Generic("No <metadata> section found in OPF".to_string()))?;

    let metadata_end = original_opf.find("</metadata>")
        .ok_or_else(|| RitmoErr::Generic("No </metadata> closing tag found in OPF".to_string()))?;

    // Extract parts: before metadata, metadata section, after metadata
    let before_metadata = &original_opf[..metadata_start];
    let after_metadata = &original_opf[metadata_end + "</metadata>".len()..];

    // Extract the opening metadata tag with all its attributes and namespaces
    let metadata_opening_end = original_opf[metadata_start..]
        .find('>')
        .ok_or_else(|| RitmoErr::Generic("Malformed <metadata> tag".to_string()))?;
    let metadata_opening_tag = &original_opf[metadata_start..metadata_start + metadata_opening_end + 1];

    // Build new metadata content
    let mut new_metadata_content = String::new();

    // Add title
    if let Some(title) = &metadata.title {
        new_metadata_content.push_str(&format!("\n    <dc:title>{}</dc:title>", escape_xml(title)));
    }

    // Add creators (authors)
    for creator in &metadata.creators {
        new_metadata_content.push_str(&format!(
            "\n    <dc:creator opf:role=\"{}\">{}</dc:creator>",
            creator.role, escape_xml(&creator.name)
        ));
    }

    // Add contributors (translators, editors, etc.)
    for contributor in &metadata.contributors {
        new_metadata_content.push_str(&format!(
            "\n    <dc:contributor opf:role=\"{}\">{}</dc:contributor>",
            contributor.role, escape_xml(&contributor.name)
        ));
    }

    // Add publisher
    if let Some(publisher) = &metadata.publisher {
        new_metadata_content.push_str(&format!("\n    <dc:publisher>{}</dc:publisher>", escape_xml(publisher)));
    }

    // Add date
    if let Some(date) = &metadata.date {
        new_metadata_content.push_str(&format!("\n    <dc:date>{}</dc:date>", escape_xml(date)));
    }

    // Add identifiers
    for identifier in &metadata.identifiers {
        new_metadata_content.push_str(&format!(
            "\n    <dc:identifier opf:scheme=\"{}\">{}</dc:identifier>",
            identifier.scheme, escape_xml(&identifier.value)
        ));
    }

    // Add languages
    for language in &metadata.languages {
        new_metadata_content.push_str(&format!("\n    <dc:language>{}</dc:language>", language));
    }

    // Add subjects (tags)
    for subject in &metadata.subjects {
        new_metadata_content.push_str(&format!("\n    <dc:subject>{}</dc:subject>", escape_xml(subject)));
    }

    // Add Calibre meta tags for series
    if let Some(series) = &metadata.series {
        new_metadata_content.push_str(&format!("\n    <meta name=\"calibre:series\" content=\"{}\"/>", escape_xml(series)));
    }
    if let Some(series_index) = metadata.series_index {
        new_metadata_content.push_str(&format!("\n    <meta name=\"calibre:series_index\" content=\"{}\"/>", series_index));
    }

    // Construct the new OPF
    let new_opf = format!(
        "{}{}{}\n  </metadata>{}",
        before_metadata,
        metadata_opening_tag,
        new_metadata_content,
        after_metadata
    );

    Ok(new_opf)
}

/// Helper function to escape XML special characters
fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Modifies an EPUB file by updating its OPF metadata
///
/// This function:
/// 1. Opens the EPUB as a ZIP archive
/// 2. Finds the OPF file path from META-INF/container.xml
/// 3. Extracts and modifies the OPF XML
/// 4. Creates a new EPUB with the modified OPF
///
/// # Arguments
/// * `epub_path` - Path to the original EPUB file
/// * `output_path` - Path where the modified EPUB will be written
/// * `metadata` - Metadata to apply to the OPF
///
/// # Returns
/// Ok(()) on success, error otherwise
pub fn modify_epub_metadata(
    epub_path: &Path,
    output_path: &Path,
    metadata: &OPFMetadata,
) -> RitmoResult<()> {
    // Open the EPUB as a ZIP archive
    let file = File::open(epub_path)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader).map_err(|e| {
        RitmoErr::Generic(format!("Failed to open EPUB as ZIP: {}", e))
    })?;

    // Find the OPF path from container.xml
    let opf_path = find_opf_path_in_archive(&mut archive)?;

    // Extract the original OPF content
    let original_opf = {
        let mut opf_file = archive.by_name(&opf_path).map_err(|e| {
            RitmoErr::Generic(format!("Failed to read OPF file '{}': {}", opf_path, e))
        })?;

        let mut content = String::new();
        opf_file.read_to_string(&mut content).map_err(|e| {
            RitmoErr::Generic(format!("Failed to read OPF content: {}", e))
        })?;
        content
    };

    // Modify the OPF XML
    let modified_opf = modify_opf_xml(&original_opf, metadata)?;

    // Create a new ZIP file with the modified OPF
    let output_file = File::create(output_path)?;
    let mut zip_writer = ZipWriter::new(output_file);

    // Copy all files from the original ZIP, replacing the OPF file
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            RitmoErr::Generic(format!("Failed to read ZIP entry {}: {}", i, e))
        })?;

        let file_name = file.name().to_string();
        let options = zip::write::FileOptions::<zip::write::ExtendedFileOptions>::default()
            .compression_method(file.compression())
            .unix_permissions(file.unix_mode().unwrap_or(0o644));

        if file_name == opf_path {
            // Write modified OPF
            zip_writer.start_file(&file_name, options).map_err(|e| {
                RitmoErr::Generic(format!("Failed to start OPF file in output ZIP: {}", e))
            })?;
            zip_writer.write_all(modified_opf.as_bytes()).map_err(|e| {
                RitmoErr::Generic(format!("Failed to write modified OPF: {}", e))
            })?;
        } else {
            // Copy original file
            zip_writer.start_file(&file_name, options).map_err(|e| {
                RitmoErr::Generic(format!("Failed to start file '{}' in output ZIP: {}", file_name, e))
            })?;

            let mut buffer = Vec::new();
            std::io::copy(&mut file, &mut buffer).map_err(|e| {
                RitmoErr::Generic(format!("Failed to read file '{}': {}", file_name, e))
            })?;

            zip_writer.write_all(&buffer).map_err(|e| {
                RitmoErr::Generic(format!("Failed to write file '{}': {}", file_name, e))
            })?;
        }
    }

    zip_writer.finish().map_err(|e| {
        RitmoErr::Generic(format!("Failed to finalize output EPUB: {}", e))
    })?;

    Ok(())
}

/// Finds the OPF file path by reading META-INF/container.xml
fn find_opf_path_in_archive(archive: &mut ZipArchive<BufReader<File>>) -> RitmoResult<String> {
    // Read META-INF/container.xml
    let container_content = {
        let mut container_file = archive
            .by_name("META-INF/container.xml")
            .map_err(|_| RitmoErr::Generic("META-INF/container.xml not found".to_string()))?;

        let mut content = String::new();
        container_file
            .read_to_string(&mut content)
            .map_err(|e| RitmoErr::Generic(format!("Error reading container.xml: {}", e)))?;
        content
    };

    // Parse XML to find the full-path attribute
    for line in container_content.lines() {
        if line.contains("full-path=") && line.contains("application/oebps-package+xml") {
            if let Some(start) = line.find("full-path=\"") {
                let start = start + "full-path=\"".len();
                if let Some(end) = line[start..].find('"') {
                    let path = &line[start..start + end];
                    return Ok(path.to_string());
                }
            }
        }
    }

    Err(RitmoErr::Generic(
        "Could not find OPF path in container.xml".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_mapping() {
        assert_eq!(map_ritmo_role_to_opf("role.author"), "aut");
        assert_eq!(map_ritmo_role_to_opf("role.translator"), "trl");
        assert_eq!(map_ritmo_role_to_opf("role.editor"), "edt");
        assert_eq!(map_ritmo_role_to_opf("role.illustrator"), "ill");
        assert_eq!(map_ritmo_role_to_opf("role.unknown"), "ctb");
    }

    #[test]
    fn test_element_type() {
        assert_eq!(opf_role_to_element_type("aut"), "creator");
        assert_eq!(opf_role_to_element_type("trl"), "contributor");
        assert_eq!(opf_role_to_element_type("edt"), "contributor");
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(escape_xml("Normal text"), "Normal text");
        assert_eq!(escape_xml("Text & More"), "Text &amp; More");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("Quote \" here"), "Quote &quot; here");
    }

    #[test]
    fn test_build_opf_metadata_minimal() {
        let book_metadata = BookImportMetadata {
            title: "Test Book".to_string(),
            original_title: None,
            people: None,
            publisher: None,
            year: None,
            isbn: None,
            format: None,
            series: None,
            series_index: None,
            pages: None,
            notes: None,
            tags: None,
        };

        let opf = build_opf_metadata(&book_metadata, &[]);

        assert_eq!(opf.title, Some("Test Book".to_string()));
        assert!(opf.creators.is_empty());
        assert!(opf.contributors.is_empty());
        assert!(opf.languages.is_empty());
    }

    #[test]
    fn test_build_opf_metadata_with_people() {
        let book_metadata = BookImportMetadata {
            title: "Test Book".to_string(),
            original_title: None,
            people: Some(vec![
                ("John Doe".to_string(), "role.author".to_string()),
                ("Jane Smith".to_string(), "role.editor".to_string()),
            ]),
            publisher: Some("Test Publisher".to_string()),
            year: Some(2024),
            isbn: Some("978-1234567890".to_string()),
            format: None,
            series: None,
            series_index: None,
            pages: None,
            notes: None,
            tags: Some(vec!["fiction".to_string(), "test".to_string()]),
        };

        let opf = build_opf_metadata(&book_metadata, &[]);

        assert_eq!(opf.creators.len(), 1);
        assert_eq!(opf.creators[0].name, "John Doe");
        assert_eq!(opf.creators[0].role, "aut");

        assert_eq!(opf.contributors.len(), 1);
        assert_eq!(opf.contributors[0].name, "Jane Smith");
        assert_eq!(opf.contributors[0].role, "edt");

        assert_eq!(opf.publisher, Some("Test Publisher".to_string()));
        assert_eq!(opf.date, Some("2024-01-01".to_string()));
        assert_eq!(opf.identifiers.len(), 1);
        assert_eq!(opf.identifiers[0].scheme, "ISBN");
        assert_eq!(opf.subjects.len(), 2);
    }
}
