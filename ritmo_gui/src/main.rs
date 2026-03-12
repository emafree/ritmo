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

    // Applica la palette dal config
    {
        let settings_path = ritmo_config::settings_file().unwrap_or_default();
        if let Ok(app_settings) = ritmo_config::AppSettings::load_or_create(&settings_path) {
            let palette_idx: i32 = match app_settings.preferences.ui_theme.as_str() {
                "light" => 1,
                "custom" => 2,
                _ => 0, // "dark" è il default
            };
            win.global::<crate::Theme>().set_active_palette(palette_idx);

            // Se custom, popola anche i token colore
            if palette_idx == 2 {
                let cp = &app_settings.preferences.custom_palette;
                let palette = crate::ColorPalette {
                    bg: parse_hex_color(
                        cp.bg.as_deref().unwrap_or("#0f0f0f"),
                    ),
                    surface: parse_hex_color(
                        cp.surface.as_deref().unwrap_or("#171717"),
                    ),
                    surface2: parse_hex_color(
                        cp.surface2.as_deref().unwrap_or("#1f1f1f"),
                    ),
                    surface3: parse_hex_color(
                        cp.surface3.as_deref().unwrap_or("#282828"),
                    ),
                    border: parse_hex_color(
                        cp.border.as_deref().unwrap_or("#2e2e2e"),
                    ),
                    border2: parse_hex_color(
                        cp.border2.as_deref().unwrap_or("#3a3a3a"),
                    ),
                    text_primary: parse_hex_color(
                        cp.text_primary.as_deref().unwrap_or("#e8e8e8"),
                    ),
                    text_secondary: parse_hex_color(
                        cp.text_secondary.as_deref().unwrap_or("#999999"),
                    ),
                    text_muted: parse_hex_color(
                        cp.text_muted.as_deref().unwrap_or("#555555"),
                    ),
                    accent: parse_hex_color(
                        cp.accent.as_deref().unwrap_or("#c8a96e"),
                    ),
                    accent2: parse_hex_color(
                        cp.accent2.as_deref().unwrap_or("#a07040"),
                    ),
                    active_bg: parse_hex_color(
                        cp.active_bg.as_deref().unwrap_or("#2a2218"),
                    ),
                    tag_bg: parse_hex_color(
                        cp.tag_bg.as_deref().unwrap_or("#1e2a1e"),
                    ),
                    tag_text: parse_hex_color(
                        cp.tag_text.as_deref().unwrap_or("#7ab87a"),
                    ),
                    danger: parse_hex_color(
                        cp.danger.as_deref().unwrap_or("#c0392b"),
                    ),
                    success: parse_hex_color(
                        cp.success.as_deref().unwrap_or("#27ae60"),
                    ),
                };
                win.global::<crate::Theme>().set_custom_palette(palette);
            }
        }
    }

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

    // Open BookDialog when the "+" button is clicked
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

    // Open FilterDialog when the filter action is triggered
    win.on_request_filter(move || {
        let _ = dialogs::open_filter_dialog();
    });

    // Placeholder for options menu (⋮)
    win.on_request_options({
        let win_weak = win.as_weak();
        move || {
            if let Some(w) = win_weak.upgrade() {
                let _ = dialogs::open_options_dialog(&w);
            }
        }
    });

    win.run()?;
    Ok(())
}

/// Parsa un colore hex nel formato "#rrggbb" in un `slint::Color`.
fn parse_hex_color(hex: &str) -> slint::Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return slint::Color::from_rgb_u8(0, 0, 0);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    slint::Color::from_rgb_u8(r, g, b)
}

/// Loads book and content FieldDefinitions from the DB, translating names via i18n.
async fn load_field_defs_from_db(
    lang: &str,
) -> anyhow::Result<(Vec<crate::FieldDefinition>, Vec<crate::FieldDefinition>)> {
    let settings_path = ritmo_config::settings_file()?;
    let app_settings = ritmo_config::AppSettings::load_or_create(&settings_path)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let library_path = app_settings
        .last_library_path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No library configured"))?;

    let config = ritmo_db_core::LibraryConfig::load_or_create(
        library_path.join("config").join("ritmo.toml"),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut reporter = ritmo_errors::reporter::SilentReporter;
    let db = config.create_database(&mut reporter).await?;
    let pool = db.pool();

    let book_rows = ritmo_db::PageFieldRow::list_for_page(pool, "book_page").await?;
    let content_rows = ritmo_db::PageFieldRow::list_for_page(pool, "content_page").await?;

    let book_fields = i18n::rows_to_slint_fields(&book_rows, lang);
    let content_fields = i18n::rows_to_slint_fields(&content_rows, lang);

    Ok((book_fields, content_fields))
}
