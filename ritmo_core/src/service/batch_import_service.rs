use crate::dto::{BatchImportInput, ContentInput, ImportObject};
use crate::service::book_import_service::{import_book_with_contents, BookImportMetadata};
use ritmo_db::{Content, Person, Role, RunningLanguages, Type};
use ritmo_db_core::LibraryConfig;
use ritmo_errors::{RitmoErr, RitmoResult};
use std::path::PathBuf;

/// Result of a single import operation
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub file_path: String,
    pub success: bool,
    pub book_id: Option<i64>,
    pub error_message: Option<String>,
}

/// Summary of batch import operation
#[derive(Debug, Clone)]
pub struct BatchImportSummary {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped_duplicates: usize,
    pub results: Vec<ImportResult>,
}

impl BatchImportSummary {
    pub fn new() -> Self {
        Self {
            total: 0,
            successful: 0,
            failed: 0,
            skipped_duplicates: 0,
            results: Vec::new(),
        }
    }

    pub fn add_success(&mut self, file_path: String, book_id: i64) {
        self.successful += 1;
        self.results.push(ImportResult {
            file_path,
            success: true,
            book_id: Some(book_id),
            error_message: None,
        });
    }

    pub fn add_failure(&mut self, file_path: String, error: String) {
        self.failed += 1;
        self.results.push(ImportResult {
            file_path,
            success: false,
            book_id: None,
            error_message: Some(error),
        });
    }

    pub fn add_duplicate(&mut self, file_path: String) {
        self.skipped_duplicates += 1;
        self.results.push(ImportResult {
            file_path,
            success: false,
            book_id: None,
            error_message: Some("Duplicate (already imported)".to_string()),
        });
    }
}

/// Import books from JSON batch input
///
/// # Arguments
/// * `config` - Library configuration
/// * `pool` - Database connection pool
/// * `batch_input` - Deserialized JSON array of ImportObject
/// * `stop_on_error` - If true, abort on first error; if false, continue on errors
///
/// # Returns
/// * `BatchImportSummary` with results for each import operation
pub async fn batch_import(
    config: &LibraryConfig,
    pool: &sqlx::SqlitePool,
    batch_input: BatchImportInput,
    stop_on_error: bool,
) -> RitmoResult<BatchImportSummary> {
    let mut summary = BatchImportSummary::new();
    summary.total = batch_input.len();

    for import_obj in batch_input {
        let result = import_single(config, pool, import_obj.clone()).await;

        match result {
            Ok(book_id) => {
                summary.add_success(import_obj.file_path.clone(), book_id);
            }
            Err(e) => {
                let error_msg = format!("{:?}", e);

                // Check if it's a duplicate error
                if error_msg.contains("già importato") || error_msg.contains("already imported") {
                    summary.add_duplicate(import_obj.file_path.clone());
                } else {
                    summary.add_failure(import_obj.file_path.clone(), error_msg.clone());

                    if stop_on_error {
                        return Err(RitmoErr::Generic(format!(
                            "Import aborted at '{}': {}",
                            import_obj.file_path, error_msg
                        )));
                    }
                }
            }
        }
    }

    Ok(summary)
}

/// Import a single book with its contents from ImportObject
async fn import_single(
    config: &LibraryConfig,
    pool: &sqlx::SqlitePool,
    import_obj: ImportObject,
) -> RitmoResult<i64> {
    // 1. Validate import object
    validate_import_object(&import_obj)?;

    // 2. Resolve file path (support both absolute and relative)
    let file_path = PathBuf::from(&import_obj.file_path);
    let file_path = if file_path.is_absolute() {
        file_path
    } else {
        std::env::current_dir()?.join(file_path)
    };

    // 3. Build BookImportMetadata from BookInput
    let book_metadata = BookImportMetadata {
        title: import_obj.book.title.clone(),
        original_title: import_obj.book.original_title.clone(),
        people: if import_obj.book.people.is_empty() {
            None
        } else {
            Some(
                import_obj
                    .book
                    .people
                    .iter()
                    .map(|p| (p.name.clone(), p.role.clone()))
                    .collect(),
            )
        },
        publisher: import_obj.book.publisher.clone(),
        year: import_obj.book.year,
        isbn: import_obj.book.isbn.clone(),
        format: import_obj.book.format.clone(),
        series: import_obj.book.series.clone(),
        series_index: import_obj.book.series_index,
        pages: import_obj.book.pages,
        notes: import_obj.book.notes.clone(),
        tags: if import_obj.book.tags.is_empty() {
            None
        } else {
            Some(import_obj.book.tags.clone())
        },
    };

    // 4. Import book using existing service WITH contents for OPF modification
    let book_id = import_book_with_contents(
        config,
        pool,
        &file_path,
        book_metadata,
        &import_obj.contents,
    )
    .await?;

    // 5. Create and associate contents
    for content_input in import_obj.contents {
        let content_id = create_content_from_input(pool, &content_input).await?;

        // Link content to book
        sqlx::query!(
            "INSERT INTO x_books_contents (book_id, content_id) VALUES (?, ?)",
            book_id,
            content_id
        )
        .execute(pool)
        .await?;

        // Associate content people with roles
        for person_input in &content_input.people {
            let person_id = Person::get_or_create_by_name(pool, &person_input.name).await?;
            let role_id = Role::get_or_create_by_key(pool, &person_input.role).await?;

            sqlx::query!(
                "INSERT INTO x_contents_people_roles (content_id, person_id, role_id) VALUES (?, ?, ?)",
                content_id,
                person_id,
                role_id
            )
            .execute(pool)
            .await?;
        }

        // Associate content languages
        for lang_input in &content_input.languages {
            // Use official name as the language code for now (can be enhanced later)
            // ISO3 is empty string if not provided
            let language_id = RunningLanguages::get_or_create_by_iso_and_role(
                pool,
                &lang_input.code.to_uppercase(), // Use code as official name
                &lang_input.code,                 // ISO 639-1 code (2 char)
                "",                                // ISO 639-2 code (3 char) - empty for now
                &lang_input.role,
            )
            .await?;

            // Link language to content
            sqlx::query!(
                "INSERT INTO x_contents_languages (content_id, language_id) VALUES (?, ?)
                 ON CONFLICT DO NOTHING",
                content_id,
                language_id
            )
            .execute(pool)
            .await?;
        }
    }

    Ok(book_id)
}

/// Create a content from ContentInput
async fn create_content_from_input(
    pool: &sqlx::SqlitePool,
    content_input: &ContentInput,
) -> RitmoResult<i64> {
    // Get or create type if specified
    let type_id = if let Some(type_key) = &content_input.content_type {
        Some(Type::get_or_create_by_key(pool, type_key).await?)
    } else {
        None
    };

    // Convert year to timestamp if present
    let publication_date = content_input.year.map(|y| {
        chrono::NaiveDate::from_ymd_opt(y, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp()
    });

    // Create Content record
    let now = chrono::Utc::now().timestamp();
    let content = Content {
        id: None,
        name: content_input.title.clone(),
        original_title: content_input.original_title.clone(),
        type_id,
        publication_date,
        pages: None, // Contents don't have separate page counts
        notes: None, // Contents inherit notes from book
        created_at: now,
        updated_at: now,
    };

    // Save and return ID
    let id = content.save(pool).await?;
    Ok(id)
}

/// Validate ImportObject structure
pub fn validate_import_object(obj: &ImportObject) -> RitmoResult<()> {
    // Validate file_path
    if obj.file_path.trim().is_empty() {
        return Err(RitmoErr::Generic(
            "file_path cannot be empty".to_string(),
        ));
    }

    // Validate book.title
    if obj.book.title.trim().is_empty() {
        return Err(RitmoErr::Generic("book.title cannot be empty".to_string()));
    }

    // Validate book.year if present
    if let Some(year) = obj.book.year {
        if !(1000..=2100).contains(&year) {
            return Err(RitmoErr::Generic(format!(
                "book.year must be between 1000 and 2100, got {}",
                year
            )));
        }
    }

    // Validate book.series_index if present
    if let Some(idx) = obj.book.series_index {
        if idx < 1 {
            return Err(RitmoErr::Generic(format!(
                "book.series_index must be positive, got {}",
                idx
            )));
        }
    }

    // Validate book.pages if present
    if let Some(pages) = obj.book.pages {
        if pages < 1 {
            return Err(RitmoErr::Generic(format!(
                "book.pages must be positive, got {}",
                pages
            )));
        }
    }

    // Validate book people
    for person in &obj.book.people {
        if person.name.trim().is_empty() {
            return Err(RitmoErr::Generic(
                "book.people[].name cannot be empty".to_string(),
            ));
        }
        if !person.role.starts_with("role.") {
            return Err(RitmoErr::Generic(format!(
                "book.people[].role must be i18n key starting with 'role.', got '{}'",
                person.role
            )));
        }
    }

    // Validate contents
    for (idx, content) in obj.contents.iter().enumerate() {
        // Validate content.title
        if content.title.trim().is_empty() {
            return Err(RitmoErr::Generic(format!(
                "contents[{}].title cannot be empty",
                idx
            )));
        }

        // Validate content.year if present
        if let Some(year) = content.year {
            if !(1000..=2100).contains(&year) {
                return Err(RitmoErr::Generic(format!(
                    "contents[{}].year must be between 1000 and 2100, got {}",
                    idx, year
                )));
            }
        }

        // Validate content.type if present
        if let Some(ref ctype) = content.content_type {
            if !ctype.starts_with("type.") {
                return Err(RitmoErr::Generic(format!(
                    "contents[{}].type must be i18n key starting with 'type.', got '{}'",
                    idx, ctype
                )));
            }
        }

        // Validate content people
        for person in &content.people {
            if person.name.trim().is_empty() {
                return Err(RitmoErr::Generic(format!(
                    "contents[{}].people[].name cannot be empty",
                    idx
                )));
            }
            if !person.role.starts_with("role.") {
                return Err(RitmoErr::Generic(format!(
                    "contents[{}].people[].role must be i18n key starting with 'role.', got '{}'",
                    idx, person.role
                )));
            }
        }

        // Validate content languages
        for lang in &content.languages {
            if lang.code.len() != 2 {
                return Err(RitmoErr::Generic(format!(
                    "contents[{}].languages[].code must be 2-letter ISO 639-1 code, got '{}'",
                    idx, lang.code
                )));
            }
            if !lang.role.starts_with("language_role.") {
                return Err(RitmoErr::Generic(format!(
                    "contents[{}].languages[].role must be i18n key starting with 'language_role.', got '{}'",
                    idx, lang.role
                )));
            }
        }
    }

    Ok(())
}
