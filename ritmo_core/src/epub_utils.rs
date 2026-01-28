use ritmo_errors::{RitmoErr, RitmoResult};
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;
use zip::ZipArchive;

/// Estrae il file OPF (Open Packaging Format) da un EPUB
///
/// Gli EPUB sono file ZIP che contengono un file OPF con i metadati.
/// Il path al file OPF è specificato in META-INF/container.xml.
///
/// # Arguments
/// * `epub_path` - Path al file EPUB
///
/// # Returns
/// Contenuto del file OPF come String
pub fn extract_opf(epub_path: &Path) -> RitmoResult<String> {
    let file = File::open(epub_path)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader).map_err(|e| {
        RitmoErr::Generic(format!("Impossibile aprire EPUB come ZIP: {}", e))
    })?;

    // 1. Cerca il path OPF in META-INF/container.xml
    let opf_path = find_opf_path_in_container(&mut archive)?;

    // 2. Estrai il file OPF
    let mut opf_file = archive
        .by_name(&opf_path)
        .map_err(|e| RitmoErr::Generic(format!("File OPF '{}' non trovato: {}", opf_path, e)))?;

    let mut opf_content = String::new();
    opf_file
        .read_to_string(&mut opf_content)
        .map_err(|e| RitmoErr::Generic(format!("Errore lettura OPF: {}", e)))?;

    Ok(opf_content)
}

/// Cerca il path del file OPF nel META-INF/container.xml
fn find_opf_path_in_container(archive: &mut ZipArchive<BufReader<File>>) -> RitmoResult<String> {
    // Leggi META-INF/container.xml
    let container_content = {
        let mut container_file = archive
            .by_name("META-INF/container.xml")
            .map_err(|_| RitmoErr::Generic("META-INF/container.xml non trovato".to_string()))?;

        let mut content = String::new();
        container_file
            .read_to_string(&mut content)
            .map_err(|e| RitmoErr::Generic(format!("Errore lettura container.xml: {}", e)))?;
        content
    }; // container_file viene droppato qui

    // Parse XML per trovare il full-path del file OPF
    // Cerchiamo la riga: <rootfile full-path="..." media-type="application/oebps-package+xml"/>
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

    // Fallback: cerca content.opf nelle posizioni comuni
    let common_paths = vec![
        "OEBPS/content.opf",
        "EPUB/content.opf",
        "content.opf",
        "OPS/content.opf",
    ];

    for path in common_paths {
        if archive.by_name(path).is_ok() {
            return Ok(path.to_string());
        }
    }

    Err(RitmoErr::Generic(
        "Impossibile trovare file OPF nell'EPUB".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: I test richiedono file EPUB reali, quindi sono commentati.
    // In produzione si possono creare EPUB test con struttura minima.

    #[test]
    #[ignore]
    fn test_extract_opf() {
        // let opf = extract_opf(Path::new("test.epub")).unwrap();
        // assert!(opf.contains("<?xml"));
        // assert!(opf.contains("<package"));
    }
}
