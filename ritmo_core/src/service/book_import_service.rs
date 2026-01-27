use ritmo_db::{Book, Format, Person, Publisher, Role, Series, Tag};
use ritmo_db_core::LibraryConfig;
use ritmo_errors::{RitmoErr, RitmoResult};
use sha2::{Digest, Sha256};
use std::fs;
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
/// 5. Copia il file nello storage
pub async fn import_book(
    config: &LibraryConfig,
    pool: &sqlx::SqlitePool,
    file_path: &Path,
    metadata: BookImportMetadata,
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
        file_hash: Some(file_hash),
        created_at: now,
    };

    // 7. Salva nel database
    let book_id = book.save(pool).await?;

    // 8. Copia file nello storage
    let storage_path = config.canonical_storage_path().join(&relative_path);
    if let Some(parent) = storage_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(file_path, &storage_path)?;

    // 9. Crea persone e collegamento con i loro ruoli
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

    // 10. Crea e collega tags
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

fn calculate_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
