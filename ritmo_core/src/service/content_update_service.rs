use ritmo_db::{Content, Person, Role, Type};
use ritmo_errors::{RitmoErr, RitmoResult};

/// Metadati opzionali per l'aggiornamento di un contenuto
/// I campi None vengono ignorati (non modificati)
#[derive(Debug, Clone, Default)]
pub struct ContentUpdateMetadata {
    pub title: Option<String>,
    pub original_title: Option<String>,
    pub author: Option<String>,
    pub content_type: Option<String>,
    pub year: Option<i32>,
    pub notes: Option<String>,
    pub pages: Option<i64>,
}

/// Aggiorna un contenuto esistente nel database
///
/// Questa funzione:
/// 1. Verifica che il contenuto esista
/// 2. Applica solo le modifiche specificate (campi Some)
/// 3. Aggiorna le relazioni (tipo, autore) se modificate
/// 4. Salva le modifiche nel database
pub async fn update_content(
    pool: &sqlx::SqlitePool,
    content_id: i64,
    metadata: ContentUpdateMetadata,
) -> RitmoResult<()> {
    // 1. Verifica che il contenuto esista e caricalo
    let mut content = Content::get(pool, content_id)
        .await?
        .ok_or_else(|| RitmoErr::Generic(format!("Contenuto con ID {} non trovato", content_id)))?;

    // 2. Applica modifiche ai metadati diretti
    if let Some(title) = metadata.title {
        content.name = title;
    }

    if let Some(original_title) = metadata.original_title {
        content.original_title = Some(original_title);
    }

    if let Some(notes) = metadata.notes {
        content.notes = Some(notes);
    }

    if let Some(pages) = metadata.pages {
        content.pages = Some(pages);
    }

    if let Some(year) = metadata.year {
        content.publication_date = Some(
            chrono::NaiveDate::from_ymd_opt(year, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp(),
        );
    }

    // 3. Aggiorna tipo contenuto
    if let Some(type_name) = metadata.content_type {
        content.type_id = Some(Type::get_or_create_by_name(pool, &type_name).await?);
    }

    // 4. Salva modifiche nel database
    let rows_affected = content.update(pool).await?;

    if rows_affected == 0 {
        return Err(RitmoErr::Generic(format!(
            "Nessuna modifica applicata al contenuto con ID {}",
            content_id
        )));
    }

    // 5. Gestisci aggiornamento autore se specificato
    if let Some(author_name) = metadata.author {
        // Rimuovi tutte le relazioni autore esistenti
        sqlx::query!(
            "DELETE FROM x_contents_people_roles
             WHERE content_id = ?
             AND role_id = (SELECT id FROM roles WHERE name = 'Autore')",
            content_id
        )
        .execute(pool)
        .await?;

        // Aggiungi il nuovo autore
        let person_id = Person::get_or_create_by_name(pool, &author_name).await?;
        let author_role_id = Role::get_or_create_by_name(pool, "Autore").await?;

        sqlx::query!(
            "INSERT INTO x_contents_people_roles (content_id, person_id, role_id) VALUES (?, ?, ?)",
            content_id,
            person_id,
            author_role_id
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
