use ritmo_db::{Book, Format, Person, Publisher, Role, Series, Tag};
use ritmo_errors::{RitmoErr, RitmoResult};

/// Metadati opzionali per l'aggiornamento di un libro
/// I campi None vengono ignorati (non modificati)
#[derive(Debug, Clone, Default)]
pub struct BookUpdateMetadata {
    pub title: Option<String>,
    pub original_title: Option<String>,
    pub people: Option<Vec<(String, String)>>, // (name, role)
    pub publisher: Option<String>,
    pub year: Option<i32>,
    pub isbn: Option<String>,
    pub format: Option<String>,
    pub series: Option<String>,
    pub series_index: Option<i64>,
    pub notes: Option<String>,
    pub pages: Option<i64>,
    pub tags: Option<Vec<String>>,
}

/// Aggiorna un libro esistente nel database
///
/// Questa funzione:
/// 1. Verifica che il libro esista
/// 2. Applica solo le modifiche specificate (campi Some)
/// 3. Aggiorna le relazioni (formato, publisher, series, autore) se modificate
/// 4. Salva le modifiche nel database
pub async fn update_book(
    pool: &sqlx::SqlitePool,
    book_id: i64,
    metadata: BookUpdateMetadata,
) -> RitmoResult<()> {
    // 1. Verifica che il libro esista e caricalo
    let mut book = Book::get(pool, book_id)
        .await?
        .ok_or_else(|| RitmoErr::Generic(format!("Libro con ID {} non trovato", book_id)))?;

    // 2. Applica modifiche ai metadati diretti
    if let Some(title) = metadata.title {
        book.name = title;
    }

    if let Some(original_title) = metadata.original_title {
        book.original_title = Some(original_title);
    }

    if let Some(isbn) = metadata.isbn {
        book.isbn = Some(isbn);
    }

    if let Some(notes) = metadata.notes {
        book.notes = Some(notes);
    }

    if let Some(pages) = metadata.pages {
        book.pages = Some(pages);
    }

    if let Some(year) = metadata.year {
        book.publication_date = Some(
            chrono::NaiveDate::from_ymd_opt(year, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp(),
        );
    }

    // 3. Aggiorna relazioni foreign key
    if let Some(format_name) = metadata.format {
        book.format_id = Some(Format::get_or_create_by_name(pool, &format_name).await?);
    }

    if let Some(publisher_name) = metadata.publisher {
        book.publisher_id = Some(Publisher::get_or_create_by_name(pool, &publisher_name).await?);
    }

    if let Some(series_name) = metadata.series {
        book.series_id = Some(Series::get_or_create_by_name(pool, &series_name).await?);
    }

    if let Some(series_index) = metadata.series_index {
        book.series_index = Some(series_index);
    }

    // 4. Salva modifiche nel database
    let rows_affected = book.update(pool).await?;

    if rows_affected == 0 {
        return Err(RitmoErr::Generic(format!(
            "Nessuna modifica applicata al libro con ID {}",
            book_id
        )));
    }

    // 5. Gestisci aggiornamento persone e ruoli se specificato
    if let Some(people) = metadata.people {
        // Rimuovi tutte le relazioni persone-ruoli esistenti
        sqlx::query!("DELETE FROM x_books_people_roles WHERE book_id = ?", book_id)
            .execute(pool)
            .await?;

        // Aggiungi le nuove persone con i loro ruoli
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

    // 6. Gestisci aggiornamento tags se specificato
    if let Some(tags) = metadata.tags {
        // Rimuovi tutti i tags esistenti
        sqlx::query!("DELETE FROM x_books_tags WHERE book_id = ?", book_id)
            .execute(pool)
            .await?;

        // Aggiungi i nuovi tags
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

    Ok(())
}
