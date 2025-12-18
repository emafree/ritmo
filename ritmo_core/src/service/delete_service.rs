use ritmo_db::{Book, Content};
use ritmo_db_core::LibraryConfig;
use ritmo_errors::reporter::RitmoReporter;
use ritmo_errors::{RitmoErr, RitmoResult};
use std::fs;

/// Opzioni per la cancellazione di un libro
#[derive(Debug, Clone, Default)]
pub struct DeleteOptions {
    /// Se true, elimina anche il file fisico dallo storage
    pub delete_file: bool,
    /// Se true, forza la cancellazione anche se ci sono errori nel filesystem
    pub force: bool,
}

/// Elimina un libro dal database e opzionalmente il file fisico
///
/// Questa funzione:
/// 1. Verifica che il libro esista
/// 2. Opzionalmente elimina il file fisico dallo storage
/// 3. Elimina il record dal database (le relazioni vengono eliminate automaticamente per CASCADE)
///
/// # Arguments
/// * `config` - Configurazione della libreria (per trovare i file)
/// * `pool` - Pool di connessioni al database
/// * `book_id` - ID del libro da eliminare
/// * `options` - Opzioni di cancellazione
/// * `reporter` - Reporter per messaggi di stato ed errori
pub async fn delete_book(
    config: &LibraryConfig,
    pool: &sqlx::SqlitePool,
    book_id: i64,
    options: &DeleteOptions,
    reporter: &mut impl RitmoReporter,
) -> RitmoResult<()> {
    // 1. Verifica che il libro esista e ottieni i dettagli
    let book = Book::get(pool, book_id)
        .await?
        .ok_or_else(|| RitmoErr::Generic(format!("Libro con ID {} non trovato", book_id)))?;

    // 2. Se richiesto, elimina il file fisico
    if options.delete_file {
        if let Some(file_link) = &book.file_link {
            let file_path = config.canonical_storage_path().join(file_link);

            if file_path.exists() {
                match fs::remove_file(&file_path) {
                    Ok(_) => {
                        reporter.status(&format!("File eliminato: {}", file_path.display()));
                    }
                    Err(e) => {
                        if !options.force {
                            return Err(RitmoErr::Generic(format!(
                                "Impossibile eliminare file {}: {}",
                                file_path.display(),
                                e
                            )));
                        } else {
                            reporter.error(&format!(
                                "Warning: impossibile eliminare file {} (continuando per --force): {}",
                                file_path.display(),
                                e
                            ));
                        }
                    }
                }
            } else if !options.force {
                return Err(RitmoErr::Generic(format!(
                    "File non trovato: {} (usa --force per ignorare)",
                    file_path.display()
                )));
            }
        }
    }

    // 3. Elimina record dal database
    // Le relazioni in x_books_people_roles, x_books_tags, x_books_contents
    // vengono eliminate automaticamente grazie a ON DELETE CASCADE
    let rows_affected = Book::delete(pool, book_id).await?;

    if rows_affected == 0 {
        return Err(RitmoErr::Generic(format!(
            "Nessun libro eliminato con ID {}",
            book_id
        )));
    }

    Ok(())
}

/// Elimina un contenuto dal database
///
/// Questa funzione:
/// 1. Verifica che il contenuto esista
/// 2. Elimina il record dal database (le relazioni vengono eliminate automaticamente per CASCADE)
///
/// # Arguments
/// * `pool` - Pool di connessioni al database
/// * `content_id` - ID del contenuto da eliminare
/// * `reporter` - Reporter per messaggi di stato
pub async fn delete_content(
    pool: &sqlx::SqlitePool,
    content_id: i64,
    reporter: &mut impl RitmoReporter,
) -> RitmoResult<()> {
    // 1. Verifica che il contenuto esista
    let content = Content::get(pool, content_id)
        .await?
        .ok_or_else(|| RitmoErr::Generic(format!("Contenuto con ID {} non trovato", content_id)))?;

    // 2. Elimina record dal database
    // Le relazioni in x_contents_people_roles, x_contents_tags, x_contents_languages,
    // x_books_contents vengono eliminate automaticamente grazie a ON DELETE CASCADE
    let rows_affected = Content::delete(pool, content_id).await?;

    if rows_affected == 0 {
        return Err(RitmoErr::Generic(format!(
            "Nessun contenuto eliminato con ID {}",
            content_id
        )));
    }

    reporter.status(&format!(
        "Contenuto '{}' eliminato con successo",
        content.name
    ));

    Ok(())
}

/// Pulisce entità orfane (autori, editori, serie non referenziati)
///
/// Questa funzione rimuove dal database:
/// - Autori (people) non associati a nessun libro o contenuto
/// - Editori (publishers) non associati a nessun libro
/// - Serie (series) non associate a nessun libro
/// - Formati (formats) non usati
/// - Tipi (types) non usati
/// - Tag non associati a libri o contenuti
///
/// Restituisce il numero di entità eliminate per categoria
pub async fn cleanup_orphaned_entities(pool: &sqlx::SqlitePool) -> RitmoResult<CleanupStats> {
    let mut stats = CleanupStats::default();

    // 1. Rimuovi persone orfane (non in x_books_people_roles e x_contents_people_roles)
    let people_deleted = sqlx::query!(
        "DELETE FROM people
         WHERE id NOT IN (
             SELECT DISTINCT person_id FROM x_books_people_roles
             UNION
             SELECT DISTINCT person_id FROM x_contents_people_roles
         )"
    )
    .execute(pool)
    .await?;
    stats.people_removed = people_deleted.rows_affected();

    // 2. Rimuovi editori orfani
    let publishers_deleted = sqlx::query!(
        "DELETE FROM publishers
         WHERE id NOT IN (SELECT DISTINCT publisher_id FROM books WHERE publisher_id IS NOT NULL)"
    )
    .execute(pool)
    .await?;
    stats.publishers_removed = publishers_deleted.rows_affected();

    // 3. Rimuovi serie orfane
    let series_deleted = sqlx::query!(
        "DELETE FROM series
         WHERE id NOT IN (SELECT DISTINCT series_id FROM books WHERE series_id IS NOT NULL)"
    )
    .execute(pool)
    .await?;
    stats.series_removed = series_deleted.rows_affected();

    // 4. Rimuovi formati orfani
    let formats_deleted = sqlx::query!(
        "DELETE FROM formats
         WHERE id NOT IN (SELECT DISTINCT format_id FROM books WHERE format_id IS NOT NULL)"
    )
    .execute(pool)
    .await?;
    stats.formats_removed = formats_deleted.rows_affected();

    // 5. Rimuovi tipi orfani
    let types_deleted = sqlx::query!(
        "DELETE FROM types
         WHERE id NOT IN (SELECT DISTINCT type_id FROM contents WHERE type_id IS NOT NULL)"
    )
    .execute(pool)
    .await?;
    stats.types_removed = types_deleted.rows_affected();

    // 6. Rimuovi tag orfani
    let tags_deleted = sqlx::query!(
        "DELETE FROM tags
         WHERE id NOT IN (
             SELECT DISTINCT tag_id FROM x_books_tags
             UNION
             SELECT DISTINCT tag_id FROM x_contents_tags
         )"
    )
    .execute(pool)
    .await?;
    stats.tags_removed = tags_deleted.rows_affected();

    Ok(stats)
}

/// Statistiche di pulizia entità orfane
#[derive(Debug, Default, Clone)]
pub struct CleanupStats {
    pub people_removed: u64,
    pub publishers_removed: u64,
    pub series_removed: u64,
    pub formats_removed: u64,
    pub types_removed: u64,
    pub tags_removed: u64,
}

impl CleanupStats {
    pub fn total(&self) -> u64 {
        self.people_removed
            + self.publishers_removed
            + self.series_removed
            + self.formats_removed
            + self.types_removed
            + self.tags_removed
    }

    pub fn has_changes(&self) -> bool {
        self.total() > 0
    }
}
