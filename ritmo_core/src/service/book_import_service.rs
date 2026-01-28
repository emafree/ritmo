use crate::dto::ContentInput;
use crate::epub_opf_modifier;
use crate::epub_utils::extract_opf;
use ritmo_db::{Book, Format, Person, Publisher, Role, Series, Tag};
use ritmo_db_core::LibraryConfig;
use ritmo_errors::{RitmoErr, RitmoResult};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Metadati per l'import di un libro
#[derive(Debug, Clone)]
pub struct BookImportMetadata {
    pub title: String,
    pub original_title: Option<String>,
    pub people: Option<Vec<(String, String)>>, // (name, role)
    pub publisher: Option<String>,
    pub year: Option<i32>,
    pub isbn: Option<String>,
    pub format: Option<String>,
    pub series: Option<String>,
    pub series_index: Option<i64>,
    pub pages: Option<i64>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Importa un libro da file con metadati forniti
///
/// Questa funzione:
/// 1. Verifica che il file esista
/// 2. Calcola l'hash per rilevare duplicati
/// 3. Crea/ottiene le entità correlate (formato, publisher, series, autore)
/// 4. Salva il libro nel database
/// 5. Modifica metadati OPF nell'EPUB (se applicabile)
/// 6. Copia il file nello storage
///
/// # Arguments
/// * `config` - Library configuration
/// * `pool` - Database connection pool
/// * `file_path` - Path to the file to import
/// * `metadata` - Book metadata provided by user
/// * `contents` - Optional content metadata (from batch import Level 2)
///
/// # Returns
/// Book ID on success
pub async fn import_book_with_contents(
    config: &LibraryConfig,
    pool: &sqlx::SqlitePool,
    file_path: &Path,
    metadata: BookImportMetadata,
    contents: &[ContentInput],
) -> RitmoResult<i64> {
    // 1. Verifica che il file esista
    if !file_path.exists() {
        return Err(RitmoErr::Generic(format!(
            "File non trovato: {}",
            file_path.display()
        )));
    }

    // 2. Leggi il file e calcola hash
    let file_content = fs::read(file_path)?;
    let file_hash = calculate_hash(&file_content);

    // 3. Verifica duplicati (controlla se l'hash esiste già)
    let existing = sqlx::query!(
        "SELECT id, name FROM books WHERE file_hash = ? LIMIT 1",
        file_hash
    )
    .fetch_optional(pool)
    .await?;

    if let Some(dup) = existing {
        return Err(RitmoErr::Generic(format!(
            "File già importato: {} (ID: {})",
            dup.name, dup.id
        )));
    }

    // 4. Determina formato dal metadato o dall'estensione
    let format_name = metadata.format.clone().or_else(|| {
        file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
    });

    // 5. Ottieni/crea IDs per entità correlate usando i metodi dei modelli
    let format_id = if let Some(fmt) = format_name {
        Some(Format::get_or_create_by_key(pool, &fmt).await?)
    } else {
        None
    };

    let publisher_id = if let Some(pub_name) = &metadata.publisher {
        Some(Publisher::get_or_create_by_name(pool, pub_name).await?)
    } else {
        None
    };

    let series_id = if let Some(series_name) = &metadata.series {
        Some(Series::get_or_create_by_name(pool, series_name).await?)
    } else {
        None
    };

    // 6. Crea Book
    let now = chrono::Utc::now().timestamp();
    let publication_date = metadata.year.map(|y| {
        chrono::NaiveDate::from_ymd_opt(y, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp()
    });

    // Determina estensione file
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("epub");

    // Costruisci OPF metadata PRIMA di consumare metadata nel Book struct
    // (necessario perché alcuni campi vengono spostati nel Book)
    let opf_metadata = epub_opf_modifier::build_opf_metadata(&metadata, contents);

    // Genera path hash-based gerarchico
    // Formato: books/{hash[0:2]}/{hash[2:4]}/{hash[4:]}.{ext}
    let relative_path = format!(
        "books/{}/{}/{}.{}",
        &file_hash[0..2],   // Primo livello directory
        &file_hash[2..4],   // Secondo livello directory
        &file_hash[4..],    // Nome file (resto dell'hash)
        extension
    );

    let book = Book {
        id: None,
        name: metadata.title.clone(),
        original_title: metadata.original_title,
        publisher_id,
        format_id,
        series_id,
        series_index: metadata.series_index,
        publication_date,
        last_modified_date: now,
        isbn: metadata.isbn,
        pages: metadata.pages,
        notes: metadata.notes,
        has_cover: 0,
        has_paper: 0,
        file_link: Some(relative_path.clone()),
        file_size: Some(file_content.len() as i64),
        file_hash: Some(file_hash.clone()),
        created_at: now,
    };

    // 7. Salva nel database
    let book_id = book.save(pool).await?;

    // 8. Prepara directory storage
    let storage_path = config.canonical_storage_path().join(&relative_path);
    if let Some(parent) = storage_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 9. Estrai e salva OPF originale (solo per EPUB) - BACKUP
    if extension == "epub" {
        match extract_opf(file_path) {
            Ok(opf_content) => {
                // Path OPF: storage/originals_opf/{hash[0:2]}/{hash[2:4]}/{hash[4:]}.opf.xml
                let opf_relative_path = format!(
                    "originals_opf/{}/{}/{}.opf.xml",
                    &file_hash[0..2],
                    &file_hash[2..4],
                    &file_hash[4..]
                );

                let opf_storage_path = config.canonical_storage_path().join(&opf_relative_path);

                // Crea directory se non esistono
                if let Some(parent) = opf_storage_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                // Salva OPF originale
                let mut opf_file = fs::File::create(&opf_storage_path)?;
                opf_file.write_all(opf_content.as_bytes())?;
            }
            Err(_) => {
                // Se l'estrazione fallisce, continuiamo comunque l'import
                // (alcuni EPUB potrebbero avere strutture non standard)
            }
        }
    }

    // 10. Modifica EPUB con metadati utente (solo per EPUB)
    if extension == "epub" {
        // opf_metadata già costruito all'inizio della funzione

        // Crea temp file per EPUB modificato
        let temp_epub = storage_path.with_extension("epub.tmp");

        match epub_opf_modifier::modify_epub_metadata(file_path, &temp_epub, &opf_metadata) {
            Ok(_) => {
                // Successo: sposta temp → finale
                fs::rename(&temp_epub, &storage_path)?;
            }
            Err(e) => {
                // Fallimento: log warning, copia EPUB originale as-is
                eprintln!("Warning: Could not modify EPUB metadata: {:?}", e);
                eprintln!("Copying original EPUB without modification");

                // Rimuovi temp file se esiste
                let _ = fs::remove_file(&temp_epub);

                // Copia originale
                fs::copy(file_path, &storage_path)?;
            }
        }
    } else {
        // Non-EPUB: copia as-is
        fs::copy(file_path, &storage_path)?;
    }

    // 10. Crea persone e collegamento con i loro ruoli
    if let Some(people) = metadata.people {
        for (person_name, role_name) in people {
            let person_id = Person::get_or_create_by_name(pool, &person_name).await?;
            let role_id = Role::get_or_create_by_key(pool, &role_name).await?;

            sqlx::query!(
                "INSERT INTO x_books_people_roles (book_id, person_id, role_id) VALUES (?, ?, ?)",
                book_id,
                person_id,
                role_id
            )
            .execute(pool)
            .await?;
        }
    }

    // 11. Crea e collega tags
    if let Some(tags) = metadata.tags {
        for tag_name in tags {
            let tag_id = Tag::get_or_create_by_name(pool, &tag_name).await?;
            sqlx::query!(
                "INSERT INTO x_books_tags (book_id, tag_id) VALUES (?, ?)",
                book_id,
                tag_id
            )
            .execute(pool)
            .await?;
        }
    }

    Ok(book_id)
}

/// Importa un libro senza contents (Level 1 - manual import)
///
/// Wrapper for backward compatibility. Calls `import_book_with_contents` with empty contents array.
///
/// # Arguments
/// * `config` - Library configuration
/// * `pool` - Database connection pool
/// * `file_path` - Path to the file to import
/// * `metadata` - Book metadata provided by user
///
/// # Returns
/// Book ID on success
pub async fn import_book(
    config: &LibraryConfig,
    pool: &sqlx::SqlitePool,
    file_path: &Path,
    metadata: BookImportMetadata,
) -> RitmoResult<i64> {
    import_book_with_contents(config, pool, file_path, metadata, &[]).await
}

fn calculate_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
