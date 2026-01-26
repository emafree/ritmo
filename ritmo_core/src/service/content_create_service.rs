use ritmo_db::{Content, Person, Role, RunningLanguages, Tag, Type};
use ritmo_errors::{RitmoErr, RitmoResult};

/// Metadati per la creazione di un nuovo contenuto
#[derive(Debug, Clone)]
pub struct ContentCreateMetadata {
    pub title: String,
    pub original_title: Option<String>,
    pub people: Option<Vec<(String, String)>>, // (name, role)
    pub content_type: Option<String>,
    pub year: Option<i32>,
    pub pages: Option<i64>,
    pub notes: Option<String>,
    pub book_id: Option<i64>, // Opzionale: associa il content a un book
    pub tags: Option<Vec<String>>,
    pub languages: Option<Vec<(String, String, String, String)>>, // (name, iso2, iso3, role)
}

/// Crea un nuovo contenuto nel database
///
/// Questa funzione:
/// 1. Valida i metadati forniti
/// 2. Crea/ottiene il tipo di contenuto (se specificato)
/// 3. Salva il contenuto nel database
/// 4. Associa l'autore (se specificato)
/// 5. Associa il contenuto a un book (se specificato)
pub async fn create_content(
    pool: &sqlx::SqlitePool,
    metadata: ContentCreateMetadata,
) -> RitmoResult<i64> {
    // 1. Valida titolo (campo obbligatorio)
    if metadata.title.trim().is_empty() {
        return Err(RitmoErr::Generic(
            "Il titolo del contenuto è obbligatorio".into(),
        ));
    }

    // 2. Ottieni/crea tipo contenuto se specificato
    let type_id = if let Some(type_name) = &metadata.content_type {
        Some(Type::get_or_create_by_key(pool, type_name).await?)
    } else {
        None
    };

    // 3. Converti anno in timestamp se presente
    let publication_date = metadata.year.map(|y| {
        chrono::NaiveDate::from_ymd_opt(y, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp()
    });

    // 4. Crea Content
    let now = chrono::Utc::now().timestamp();
    let content = Content {
        id: None,
        name: metadata.title.clone(),
        original_title: metadata.original_title,
        type_id,
        publication_date,
        pages: metadata.pages,
        notes: metadata.notes,
        created_at: now,
        updated_at: now,
    };

    // 5. Salva nel database
    let content_id = content.save(pool).await?;

    // 6. Associa persone con i loro ruoli se specificato
    if let Some(people) = metadata.people {
        for (person_name, role_name) in people {
            let person_id = Person::get_or_create_by_name(pool, &person_name).await?;
            let role_id = Role::get_or_create_by_key(pool, &role_name).await?;

            sqlx::query!(
                "INSERT INTO x_contents_people_roles (content_id, person_id, role_id) VALUES (?, ?, ?)",
                content_id,
                person_id,
                role_id
            )
            .execute(pool)
            .await?;
        }
    }

    // 7. Associa a un book se specificato
    if let Some(book_id) = metadata.book_id {
        // Verifica che il book esista
        let book_exists = sqlx::query!("SELECT id FROM books WHERE id = ?", book_id)
            .fetch_optional(pool)
            .await?;

        if book_exists.is_none() {
            return Err(RitmoErr::Generic(format!(
                "Libro con ID {} non trovato",
                book_id
            )));
        }

        // Crea l'associazione
        sqlx::query!(
            "INSERT INTO x_books_contents (book_id, content_id) VALUES (?, ?)",
            book_id,
            content_id
        )
        .execute(pool)
        .await?;
    }

    // 8. Crea e collega tags
    if let Some(tags) = metadata.tags {
        for tag_name in tags {
            let tag_id = Tag::get_or_create_by_name(pool, &tag_name).await?;
            sqlx::query!(
                "INSERT INTO x_contents_tags (content_id, tag_id) VALUES (?, ?)",
                content_id,
                tag_id
            )
            .execute(pool)
            .await?;
        }
    }

    // 9. Crea e collega languages
    if let Some(languages) = metadata.languages {
        for (official_name, iso2, iso3, role) in languages {
            let lang_id = RunningLanguages::get_or_create_by_iso_and_role(
                pool,
                &official_name,
                &iso2,
                &iso3,
                &role,
            )
            .await?;
            sqlx::query!(
                "INSERT INTO x_contents_languages (content_id, language_id) VALUES (?, ?)",
                content_id,
                lang_id
            )
            .execute(pool)
            .await?;
        }
    }

    Ok(content_id)
}

/// Associa un contenuto esistente a un libro
pub async fn link_content_to_book(
    pool: &sqlx::SqlitePool,
    content_id: i64,
    book_id: i64,
) -> RitmoResult<()> {
    // Verifica che il contenuto esista
    let content_exists = Content::get(pool, content_id).await?;
    if content_exists.is_none() {
        return Err(RitmoErr::Generic(format!(
            "Contenuto con ID {} non trovato",
            content_id
        )));
    }

    // Verifica che il book esista
    let book_exists = sqlx::query!("SELECT id FROM books WHERE id = ?", book_id)
        .fetch_optional(pool)
        .await?;

    if book_exists.is_none() {
        return Err(RitmoErr::Generic(format!(
            "Libro con ID {} non trovato",
            book_id
        )));
    }

    // Verifica se l'associazione esiste già
    let link_exists = sqlx::query!(
        "SELECT * FROM x_books_contents WHERE book_id = ? AND content_id = ?",
        book_id,
        content_id
    )
    .fetch_optional(pool)
    .await?;

    if link_exists.is_some() {
        return Err(RitmoErr::Generic(format!(
            "Il contenuto {} è già associato al libro {}",
            content_id, book_id
        )));
    }

    // Crea l'associazione
    sqlx::query!(
        "INSERT INTO x_books_contents (book_id, content_id) VALUES (?, ?)",
        book_id,
        content_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Rimuovi l'associazione tra un contenuto e un libro
pub async fn unlink_content_from_book(
    pool: &sqlx::SqlitePool,
    content_id: i64,
    book_id: i64,
) -> RitmoResult<()> {
    let result = sqlx::query!(
        "DELETE FROM x_books_contents WHERE book_id = ? AND content_id = ?",
        book_id,
        content_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(RitmoErr::Generic(format!(
            "Nessuna associazione trovata tra contenuto {} e libro {}",
            content_id, book_id
        )));
    }

    Ok(())
}
