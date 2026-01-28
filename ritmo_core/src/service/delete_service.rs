use ritmo_db::{Book, Content};
use ritmo_db_core::LibraryConfig;
use ritmo_errors::reporter::RitmoReporter;
use ritmo_errors::{RitmoErr, RitmoResult};
use std::fs;

/// Opzioni per la cancellazione di un libro
///
/// Controlla il comportamento della funzione `delete_book()`.
#[derive(Debug, Clone, Default)]
pub struct DeleteOptions {
    /// Se true, elimina anche il file fisico dalla directory `storage/`.
    /// Se false, elimina solo il record dal database.
    pub delete_file: bool,

    /// Se true, continua la cancellazione anche se il file non esiste o non può essere eliminato.
    /// Se false, la funzione restituisce errore in caso di problemi con il filesystem.
    /// Utile quando il file è già stato eliminato manualmente o non è più accessibile.
    pub force: bool,
}

/// Elimina un libro dal database e opzionalmente il file fisico
///
/// Questa funzione:
/// 1. Verifica che il libro esista
/// 2. Opzionalmente elimina il file fisico dallo storage
/// 3. Elimina il record dal database
///
/// # Comportamento CASCADE automatico (ON DELETE CASCADE nel database schema)
///
/// Quando un libro viene eliminato, le seguenti relazioni vengono rimosse automaticamente:
/// - **x_books_contents**: Associazioni libro-contenuti
/// - **x_books_people_roles**: Associazioni libro-autori/contributori
/// - **x_books_tags**: Associazioni libro-tag
///
/// Le entità referenziate (people, publishers, series, formats, tags, contents) **NON** vengono
/// eliminate e possono diventare orfane. Utilizzare `cleanup_orphaned_entities()` per rimuoverle.
///
/// # Arguments
/// * `config` - Configurazione della libreria (per trovare i file)
/// * `pool` - Pool di connessioni al database
/// * `book_id` - ID del libro da eliminare
/// * `options` - Opzioni di cancellazione (delete_file, force)
/// * `reporter` - Reporter per messaggi di stato ed errori
///
/// # Errors
/// Restituisce errore se:
/// - Il libro non esiste
/// - Il file non può essere eliminato (senza flag --force)
/// - Errori del database durante la cancellazione
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
/// Un'entità è considerata "orfana" quando non è più referenziata da nessun libro o contenuto.
/// Questo accade tipicamente dopo la cancellazione di libri con `delete_book()`.
///
/// Questa funzione rimuove dal database:
/// - **People** (autori, traduttori, etc.): non presenti in `x_books_people_roles` né `x_contents_people_roles`
/// - **Publishers**: non referenziati da nessun libro (`books.publisher_id`)
/// - **Series**: non referenziate da nessun libro (`books.series_id`)
/// - **Formats**: non usati da nessun libro (`books.format_id`)
/// - **Types**: non usati da nessun contenuto (`contents.type_id`)
/// - **Tags**: non presenti in `x_books_tags` né `x_contents_tags`
///
/// # Workflow raccomandato
/// ```text
/// 1. Eliminare uno o più libri con delete_book()
/// 2. Chiamare cleanup_orphaned_entities() per rimuovere entità orfane
/// ```
///
/// # Returns
/// `CleanupStats` con il conteggio di entità eliminate per categoria
///
/// # Errors
/// Restituisce errore in caso di problemi di database
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
///
/// Contiene il numero di entità rimosse per categoria dalla funzione `cleanup_orphaned_entities()`.
/// Ogni campo rappresenta il conteggio di record eliminati dalla rispettiva tabella.
#[derive(Debug, Default, Clone)]
pub struct CleanupStats {
    /// Numero di persone (autori, traduttori, etc.) rimosse
    pub people_removed: u64,
    /// Numero di editori rimossi
    pub publishers_removed: u64,
    /// Numero di serie rimosse
    pub series_removed: u64,
    /// Numero di formati (epub, pdf, etc.) rimossi
    pub formats_removed: u64,
    /// Numero di tipi di contenuto rimossi
    pub types_removed: u64,
    /// Numero di tag rimossi
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
