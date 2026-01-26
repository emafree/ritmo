use ritmo_db::{Content, Person, Role, RunningLanguages, Tag, Type};
use ritmo_errors::{RitmoErr, RitmoResult};

/// Metadati opzionali per l'aggiornamento di un contenuto
/// I campi None vengono ignorati (non modificati)
#[derive(Debug, Clone, Default)]
pub struct ContentUpdateMetadata {
    pub title: Option<String>,
    pub original_title: Option<String>,
    pub people: Option<Vec<(String, String)>>, // (name, role)
    pub content_type: Option<String>,
    pub year: Option<i32>,
    pub notes: Option<String>,
    pub pages: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub languages: Option<Vec<(String, String, String, String)>>, // (name, iso2, iso3, role)
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
        content.type_id = Some(Type::get_or_create_by_key(pool, &type_name).await?);
    }

    // 4. Salva modifiche nel database
    let rows_affected = content.update(pool).await?;

    if rows_affected == 0 {
        return Err(RitmoErr::Generic(format!(
            "Nessuna modifica applicata al contenuto con ID {}",
            content_id
        )));
    }

    // 5. Gestisci aggiornamento persone e ruoli se specificato
    if let Some(people) = metadata.people {
        // Rimuovi tutte le relazioni persone-ruoli esistenti
        sqlx::query!(
            "DELETE FROM x_contents_people_roles WHERE content_id = ?",
            content_id
        )
        .execute(pool)
        .await?;

        // Aggiungi le nuove persone con i loro ruoli
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

    // 6. Gestisci aggiornamento tags se specificato
    if let Some(tags) = metadata.tags {
        // Rimuovi tutti i tags esistenti
        sqlx::query!("DELETE FROM x_contents_tags WHERE content_id = ?", content_id)
            .execute(pool)
            .await?;

        // Aggiungi i nuovi tags
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

    // 7. Gestisci aggiornamento languages se specificato
    if let Some(languages) = metadata.languages {
        // Rimuovi tutte le lingue esistenti
        sqlx::query!(
            "DELETE FROM x_contents_languages WHERE content_id = ?",
            content_id
        )
        .execute(pool)
        .await?;

        // Aggiungi le nuove lingue
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

    Ok(())
}
