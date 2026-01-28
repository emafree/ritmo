use crate::dto::ContentInput;
use crate::epub_opf_modifier::{build_opf_metadata, modify_epub_metadata};
use crate::service::book_import_service::BookImportMetadata;
use chrono::Datelike;
use ritmo_db::{
    clear_sync_mark, Book, Content, Format, Publisher, Series, Type,
};
use ritmo_db_core::LibraryConfig;
use ritmo_errors::{RitmoErr, RitmoResult};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// Result of syncing a single book
#[derive(Debug)]
pub struct SyncResult {
    pub book_id: i64,
    pub old_hash: String,
    pub new_hash: String,
    pub old_path: PathBuf,
    pub new_path: PathBuf,
}

/// Sync metadata for a single book: DB → EPUB
///
/// Steps:
/// 1. Read all book metadata from DB
/// 2. Build BookImportMetadata from DB data
/// 3. Read contents associated with this book
/// 4. Build OPFMetadata
/// 5. Modify EPUB with DB metadata
/// 6. Calculate new hash
/// 7. Move file to new hash-based path
/// 8. Update DB with new hash and path
/// 9. Clear sync mark
pub async fn sync_book_metadata(
    config: &LibraryConfig,
    pool: &sqlx::SqlitePool,
    book_id: i64,
) -> RitmoResult<SyncResult> {
    // Step 1: Read book metadata from DB
    let book = Book::get(pool, book_id)
        .await?
        .ok_or_else(|| RitmoErr::Generic(format!("Book ID {} not found", book_id)))?;

    let old_hash = book
        .file_hash
        .clone()
        .ok_or_else(|| RitmoErr::Generic(format!("Book ID {} has no file_hash", book_id)))?;

    let old_path = config.canonical_storage_path().join(
        book.file_link
            .as_ref()
            .ok_or_else(|| RitmoErr::Generic(format!("Book ID {} has no file_link", book_id)))?,
    );

    if !old_path.exists() {
        return Err(RitmoErr::Generic(format!(
            "EPUB file not found: {}",
            old_path.display()
        )));
    }

    // Step 2: Build BookImportMetadata from DB
    let metadata = build_book_metadata_from_db(pool, &book).await?;

    // Step 3: Read contents associated with this book
    let contents = get_book_contents(pool, book_id).await?;

    // Step 4: Build OPFMetadata
    let opf_metadata = build_opf_metadata(&metadata, &contents);

    // Step 5: Modify EPUB
    let temp_epub = old_path.with_extension("epub.sync.tmp");
    modify_epub_metadata(&old_path, &temp_epub, &opf_metadata)?;

    // Step 6: Calculate new hash
    let file_content = fs::read(&temp_epub)?;
    let new_hash = calculate_hash(&file_content);

    // Step 7: Determine new path
    let extension = old_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("epub");
    let new_relative_path = format!(
        "books/{}/{}/{}.{}",
        &new_hash[0..2],
        &new_hash[2..4],
        &new_hash[4..],
        extension
    );
    let new_path = config.canonical_storage_path().join(&new_relative_path);

    // Step 8: Move file to new location
    if let Some(parent) = new_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&temp_epub, &new_path)?;

    // Step 9: Delete old file if different location
    if old_path != new_path {
        let _ = fs::remove_file(&old_path);
    }

    // Step 10: Update DB
    let now = chrono::Utc::now().timestamp();
    sqlx::query!(
        "UPDATE books SET file_hash = ?, file_link = ?, last_modified_date = ? WHERE id = ?",
        new_hash,
        new_relative_path,
        now,
        book_id
    )
    .execute(pool)
    .await?;

    // Step 11: Clear sync mark
    clear_sync_mark(pool, book_id).await?;

    Ok(SyncResult {
        book_id,
        old_hash,
        new_hash,
        old_path,
        new_path,
    })
}

/// Build BookImportMetadata from database
async fn build_book_metadata_from_db(
    pool: &sqlx::SqlitePool,
    book: &Book,
) -> RitmoResult<BookImportMetadata> {
    // Get publisher name
    let publisher = if let Some(pub_id) = book.publisher_id {
        Publisher::get(pool, pub_id)
            .await?
            .map(|p| p.name)
    } else {
        None
    };

    // Get series name
    let series = if let Some(series_id) = book.series_id {
        Series::get(pool, series_id)
            .await?
            .map(|s| s.name)
    } else {
        None
    };

    // Get people with roles
    let people_records = sqlx::query!(
        r#"
        SELECT p.name, r.key as role_key
        FROM x_books_people_roles xbpr
        JOIN people p ON xbpr.person_id = p.id
        JOIN roles r ON xbpr.role_id = r.id
        WHERE xbpr.book_id = ?
        "#,
        book.id
    )
    .fetch_all(pool)
    .await?;

    let people = if people_records.is_empty() {
        None
    } else {
        Some(
            people_records
                .into_iter()
                .map(|r| (r.name, r.role_key))
                .collect(),
        )
    };

    // Get tags
    let tag_records = sqlx::query!(
        r#"
        SELECT t.name
        FROM x_books_tags xbt
        JOIN tags t ON xbt.tag_id = t.id
        WHERE xbt.book_id = ?
        "#,
        book.id
    )
    .fetch_all(pool)
    .await?;

    let tags = if tag_records.is_empty() {
        None
    } else {
        Some(tag_records.into_iter().map(|r| r.name).collect())
    };

    // Get year from publication_date
    let year = book.publication_date.map(|ts| {
        chrono::DateTime::from_timestamp(ts, 0)
            .map(|dt| dt.year() as i32)
            .unwrap_or(2000)
    });

    // Get format key
    let format = if let Some(fmt_id) = book.format_id {
        Format::get(pool, fmt_id)
            .await?
            .map(|f| f.key)
    } else {
        None
    };

    Ok(BookImportMetadata {
        title: book.name.clone(),
        original_title: book.original_title.clone(),
        people,
        publisher,
        year,
        isbn: book.isbn.clone(),
        format,
        series,
        series_index: book.series_index,
        pages: book.pages,
        notes: book.notes.clone(),
        tags,
    })
}

/// Get contents associated with a book (for OPF aggregation)
async fn get_book_contents(
    pool: &sqlx::SqlitePool,
    book_id: i64,
) -> RitmoResult<Vec<ContentInput>> {
    // Query contents linked to this book
    let content_ids: Vec<i64> = sqlx::query!(
        "SELECT content_id FROM x_books_contents WHERE book_id = ?",
        book_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| r.content_id)
    .collect();

    let mut contents = Vec::new();

    for content_id in content_ids {
        let content = Content::get(pool, content_id)
            .await?
            .ok_or_else(|| {
                RitmoErr::Generic(format!("Content ID {} not found", content_id))
            })?;

        // Get people for this content
        let people_records = sqlx::query!(
            r#"
            SELECT p.name, r.key as role_key
            FROM x_contents_people_roles xcpr
            JOIN people p ON xcpr.person_id = p.id
            JOIN roles r ON xcpr.role_id = r.id
            WHERE xcpr.content_id = ?
            "#,
            content_id
        )
        .fetch_all(pool)
        .await?;

        let people: Vec<_> = people_records
            .into_iter()
            .map(|r| crate::dto::PersonInput {
                name: r.name,
                role: r.role_key,
            })
            .collect();

        // Get languages for this content
        let lang_records = sqlx::query!(
            r#"
            SELECT rl.iso_code_2char as iso_code, rl.language_role as role
            FROM x_contents_languages xcl
            JOIN running_languages rl ON xcl.language_id = rl.id
            WHERE xcl.content_id = ?
            "#,
            content_id
        )
        .fetch_all(pool)
        .await?;

        let languages: Vec<_> = lang_records
            .into_iter()
            .map(|r| crate::dto::LanguageInput {
                code: r.iso_code,
                role: r.role,
            })
            .collect();

        // Get type key
        let content_type = if let Some(type_id) = content.type_id {
            Type::get(type_id, pool)
                .await?
                .map(|t| t.key)
        } else {
            None
        };

        // Get year from publication_date
        let year = content.publication_date.map(|ts| {
            chrono::DateTime::from_timestamp(ts, 0)
                .map(|dt| dt.year() as i32)
                .unwrap_or(2000)
        });

        contents.push(ContentInput {
            title: content.name,
            original_title: content.original_title,
            people,
            content_type,
            year,
            languages,
        });
    }

    Ok(contents)
}

fn calculate_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
