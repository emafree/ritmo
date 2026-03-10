slint::include_modules!();
mod dialogs;
mod i18n;

fn main() -> anyhow::Result<()> {
    let win = MainWindow::new()?;

    let lang = std::env::var("LANG")
        .unwrap_or_default()
        .split('_')
        .next()
        .unwrap_or("en")
        .to_string();

    i18n::apply_translations(&win, &lang);

    // ── Load field definitions from DB ────────────────────────────────
    let (book_fields, content_fields) = {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            match load_field_defs_from_db(&lang).await {
                Ok(pair) => pair,
                Err(e) => {
                    eprintln!("Warning: could not load field definitions from DB: {e}");
                    (vec![], vec![])
                }
            }
        })
    };

    let book_fields = std::rc::Rc::new(book_fields);
    let content_fields = std::rc::Rc::new(content_fields);

    win.on_request_add_book({
        let win_weak = win.as_weak();
        let bf = book_fields.clone();
        let cf = content_fields.clone();
        move || {
            if let Some(w) = win_weak.upgrade() {
                let _ = dialogs::open_book_dialog(&w, (*bf).clone(), (*cf).clone());
            }
        }
    });

    win.on_request_filter(move || {
        let _ = dialogs::open_filter_dialog();
    });

    win.run()?;
    Ok(())
}

/// Loads book and content FieldDefinitions from the DB, translating names via i18n.
async fn load_field_defs_from_db(
    lang: &str,
) -> anyhow::Result<(Vec<crate::FieldDefinition>, Vec<crate::FieldDefinition>)> {
    let settings_path = ritmo_config::settings_file()?;
    let app_settings = ritmo_config::AppSettings::load_or_create(&settings_path)
        .unwrap_or_default();
    let library_path = app_settings
        .get_library_to_use()
        .ok_or_else(|| anyhow::anyhow!("No library configured"))?;

    let config = ritmo_db_core::LibraryConfig::load_or_create(
        library_path.join("config").join("ritmo.toml"),
    )
    .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut reporter = ritmo_errors::reporter::SilentReporter;
    let db = config.create_database(&mut reporter).await?;
    let pool = db.pool();

    let book_rows = ritmo_db::FieldDefinitionRow::list_for_entity(pool, "book").await?;
    let content_rows = ritmo_db::FieldDefinitionRow::list_for_entity(pool, "content").await?;

    let book_fields = i18n::rows_to_slint_fields(&book_rows, lang);
    let content_fields = i18n::rows_to_slint_fields(&content_rows, lang);

    Ok((book_fields, content_fields))
}
