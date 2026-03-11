use crate::utils::opt_year_to_timestamp;
use ritmo_db::{
    crud_get, get_or_create, Book, BookPersonRole, BookTag, Format, Person, Publisher, Role,
    Series, Tag,
};
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
    // 1. Verifica che il libro esista e caricalo  crud_get::<Book>(pool, id).await
    let mut book = crud_get::<Book>(pool, book_id)
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

    if let Some(year) = metadata.year {
        book.publication_date = opt_year_to_timestamp(Some(year));
    }

    // 3. Aggiorna relazioni foreign key
    if let Some(format_name) = metadata.format {
        book.format_id = Some(get_or_create::<Format>(pool, &format_name).await?);
    }

    if let Some(publisher_name) = metadata.publisher {
        book.publisher_id = Some(get_or_create::<Publisher>(pool, &publisher_name).await?);
    }

    if let Some(series_name) = metadata.series {
        book.series_id = Some(get_or_create::<Series>(pool, &series_name).await?);
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
        sqlx::query!(
            "DELETE FROM x_books_people_roles WHERE book_id = ?",
            book_id
        )
        .execute(pool)
        .await?;

        // Aggiungi le nuove persone con i loro ruoli
        for (person_name, role_name) in people {
            let person_id = get_or_create::<Person>(pool, &person_name).await?;
            let role_id = get_or_create::<Role>(pool, &role_name).await?;

            BookPersonRole::create(
                pool,
                &BookPersonRole {
                    book_id,
                    person_id,
                    role_id,
                },
            )
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
            let tag_id = get_or_create::<Tag>(pool, &tag_name).await?;
            BookTag::create(pool, &BookTag { book_id, tag_id }).await?;
        }
    }

    Ok(())
}
