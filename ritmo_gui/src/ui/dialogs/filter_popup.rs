use egui::{Align2, Color32, Frame, Key, Margin, Vec2};
use crate::app::{App, DemoFilterCard, FilterPopupStep};
use crate::ui::palette::UiPalette;

// ---------------------------------------------------------------------------
// Demo autocomplete data
// ---------------------------------------------------------------------------

/// Book-scope filterable fields with their fake autocomplete items.
const BOOK_FIELDS: &[(&str, &[&str])] = &[
    ("Author",    &["Tolkien, J.R.R.", "Pratchett, Terry", "Le Guin, Ursula K.", "Martin, G.R.R.", "Asimov, Isaac"]),
    ("Publisher", &["HarperCollins", "Tor Books", "Orbit", "Penguin", "Del Rey"]),
    ("Series",    &["Lord of the Rings", "Discworld", "Earthsea", "Foundation", "A Song of Ice and Fire"]),
    ("Format",    &["EPUB", "PDF", "MOBI", "AZW3", "CBZ"]),
    ("Year",      &["2020", "2021", "2022", "2023", "2024"]),
];

/// Content-scope filterable fields.
const CONTENT_FIELDS: &[(&str, &[&str])] = &[
    ("Author", &["Tolkien, J.R.R.", "Pratchett, Terry", "Le Guin, Ursula K."]),
    ("Type",   &["Short Story", "Novel", "Novella", "Collection", "Anthology"]),
    ("Year",   &["2020", "2021", "2022", "2023", "2024"]),
];

fn suggestions_for(scope: &str, field: &str, query: &str) -> Vec<String> {
    let table = if scope == "Book" { BOOK_FIELDS } else { CONTENT_FIELDS };
    for (f, items) in table {
        if *f == field {
            let q = query.to_lowercase();
            return items
                .iter()
                .filter(|s| s.to_lowercase().contains(&q))
                .map(|s| s.to_string())
                .collect();
        }
    }
    vec![]
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Render the filter popup overlay (modal window).
pub fn render(app: &mut App, ctx: &egui::Context) {
    if !app.filter_ui.popup_open {
        return;
    }

    // Close on ESC
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        close_popup(app);
        return;
    }

    let palette = UiPalette::from_settings(&app.settings.theme_mode, &app.settings.custom_themes);

    // Full-screen dim overlay
    let screen_rect = ctx.screen_rect();
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("filter_popup_overlay"),
    ));
    painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(160));

    // Click-outside detection: we check *before* drawing the window so that
    // the window itself blocks the pointer.
    let pointer_down = ctx.input(|i| i.pointer.primary_clicked());

    // Modal window
    let mut open = true;
    egui::Window::new("##filter_popup")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size([480.0, 420.0])
        .frame(
            Frame::window(&ctx.style())
                .fill(palette.surface)
                .stroke(egui::Stroke::new(1.0, palette.border2))
                .inner_margin(Margin::same(0.0))
                .rounding(egui::Rounding::same(8.0)),
        )
        .open(&mut open)
        .show(ctx, |ui| {
            render_popup_contents(app, ui, &palette, pointer_down);
        });

    if !open {
        close_popup(app);
    }
}

// ---------------------------------------------------------------------------
// Popup contents
// ---------------------------------------------------------------------------

fn render_popup_contents(
    app: &mut App,
    ui: &mut egui::Ui,
    palette: &UiPalette,
    pointer_down_outside: bool,
) {
    // Header bar
    let header_rect = {
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 44.0),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, egui::Rounding { nw: 8.0, ne: 8.0, sw: 0.0, se: 0.0 }, palette.surface2);
        painter.hline(rect.x_range(), rect.bottom(), egui::Stroke::new(1.0, palette.border));
        rect
    };

    // Header label + close button inside the header rect
    {
        let mut header_ui = ui.child_ui(header_rect, egui::Layout::left_to_right(egui::Align::Center));
        header_ui.add_space(14.0);
        let title = match app.filter_ui.popup_step {
            FilterPopupStep::Step1ChooseField => "Nuovo filtro — scegli campo",
            FilterPopupStep::Step2AddValues   => "Aggiungi valori al filtro",
        };
        header_ui.colored_label(palette.text, egui::RichText::new(title).size(13.0).strong());

        // Close button aligned right
        let close_width = 36.0;
        let close_rect = egui::Rect::from_min_size(
            egui::pos2(header_rect.right() - close_width, header_rect.top()),
            Vec2::new(close_width, header_rect.height()),
        );
        let close_resp = header_ui.allocate_rect(close_rect, egui::Sense::click());
        header_ui.painter().text(
            close_rect.center(),
            Align2::CENTER_CENTER,
            "✕",
            egui::FontId::proportional(13.0),
            if close_resp.hovered() { palette.text } else { palette.text2 },
        );
        if close_resp.clicked() {
            app.filter_ui.popup_open = false;
        }
    }

    // Body
    egui::Frame::none()
        .inner_margin(Margin::symmetric(16.0, 12.0))
        .show(ui, |ui| {
            match app.filter_ui.popup_step {
                FilterPopupStep::Step1ChooseField => render_step1(app, ui, palette),
                FilterPopupStep::Step2AddValues   => render_step2(app, ui, palette),
            }
        });

    // If the user clicked outside the popup window rect, close
    // (We can't easily detect "outside" within this function; handled by `open` flag above.)
    let _ = pointer_down_outside;
}

// ---------------------------------------------------------------------------
// Step 1: choose field
// ---------------------------------------------------------------------------

fn render_step1(app: &mut App, ui: &mut egui::Ui, palette: &UiPalette) {
    ui.colored_label(palette.text2, "Seleziona il campo su cui filtrare:");
    ui.add_space(10.0);

    let mut chosen: Option<(&str, &str)> = None; // (scope, field)

    for (scope, fields) in &[("Book", BOOK_FIELDS), ("Content", CONTENT_FIELDS)] {
        // Section heading
        ui.colored_label(palette.text3, egui::RichText::new(*scope).size(10.0).strong());
        ui.add_space(4.0);

        egui::Grid::new(format!("step1_grid_{}", scope))
            .num_columns(3)
            .spacing([6.0, 6.0])
            .show(ui, |ui| {
                for (i, (field, _)) in fields.iter().enumerate() {
                    let btn = egui::Button::new(*field)
                        .fill(palette.surface2)
                        .stroke(egui::Stroke::new(1.0, palette.border))
                        .rounding(egui::Rounding::same(5.0));
                    if ui.add(btn).clicked() {
                        chosen = Some((scope, field));
                    }
                    if (i + 1) % 3 == 0 {
                        ui.end_row();
                    }
                }
                ui.end_row();
            });
        ui.add_space(8.0);
    }

    if let Some((scope, field)) = chosen {
        app.filter_ui.popup_field = Some(field.to_string());
        app.filter_ui.popup_scope = Some(scope.to_string());
        app.filter_ui.popup_step = FilterPopupStep::Step2AddValues;
        app.filter_ui.popup_search.clear();
        app.filter_ui.popup_staged.clear();
    }
}

// ---------------------------------------------------------------------------
// Step 2: add values (autocomplete + chip staging)
// ---------------------------------------------------------------------------

fn render_step2(app: &mut App, ui: &mut egui::Ui, palette: &UiPalette) {
    let field = app.filter_ui.popup_field.clone().unwrap_or_default();
    let scope = app.filter_ui.popup_scope.clone().unwrap_or_default();

    // Back link
    if ui.link("← Cambia campo").clicked() {
        app.filter_ui.popup_step = FilterPopupStep::Step1ChooseField;
        return;
    }
    ui.add_space(6.0);

    ui.colored_label(
        palette.accent,
        egui::RichText::new(format!("{} › {}", scope, field)).size(13.0).strong(),
    );
    ui.add_space(8.0);

    // Search / autocomplete box
    let search_resp = ui.add(
        egui::TextEdit::singleline(&mut app.filter_ui.popup_search)
            .hint_text("Cerca o digita un valore…")
            .desired_width(f32::INFINITY),
    );
    ui.add_space(4.0);

    // Suggestions list
    let suggestions = suggestions_for(&scope, &field, &app.filter_ui.popup_search);
    let mut to_add: Option<String> = None;

    egui::ScrollArea::vertical()
        .max_height(120.0)
        .show(ui, |ui| {
            for s in &suggestions {
                let already = app.filter_ui.popup_staged.contains(s);
                let label = if already {
                    egui::RichText::new(format!("✓ {}", s)).color(palette.accent)
                } else {
                    egui::RichText::new(s)
                };
                let resp = ui.selectable_label(already, label);
                if resp.clicked() && !already {
                    to_add = Some(s.clone());
                }
            }

            // "Add new" option when query has text and doesn't match any suggestion
            if !app.filter_ui.popup_search.is_empty()
                && !suggestions.iter().any(|s| s.eq_ignore_ascii_case(&app.filter_ui.popup_search))
            {
                ui.separator();
                let new_val = app.filter_ui.popup_search.clone();
                if ui.button(format!("+ Aggiungi \"{}\"", new_val)).clicked() {
                    to_add = Some(new_val);
                }
            }
        });

    if let Some(v) = to_add {
        if !app.filter_ui.popup_staged.contains(&v) {
            app.filter_ui.popup_staged.push(v);
        }
        app.filter_ui.popup_search.clear();
        search_resp.request_focus();
    }

    // Staged chips
    if !app.filter_ui.popup_staged.is_empty() {
        ui.add_space(8.0);
        ui.colored_label(palette.text2, "Valori selezionati:");
        ui.add_space(4.0);

        let mut to_remove: Option<usize> = None;
        ui.horizontal_wrapped(|ui| {
            for (i, v) in app.filter_ui.popup_staged.iter().enumerate() {
                let chip = egui::Button::new(format!("{} ✕", v))
                    .fill(palette.active)
                    .stroke(egui::Stroke::new(1.0, palette.accent2))
                    .rounding(egui::Rounding::same(12.0));
                if ui.add(chip).clicked() {
                    to_remove = Some(i);
                }
            }
        });
        if let Some(i) = to_remove {
            app.filter_ui.popup_staged.remove(i);
        }
    }

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    // CTA
    let can_confirm = !app.filter_ui.popup_staged.is_empty();
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.add_enabled_ui(can_confirm, |ui| {
            let cta = egui::Button::new("Aggiungi al filtro")
                .fill(palette.accent)
                .stroke(egui::Stroke::NONE)
                .rounding(egui::Rounding::same(5.0));
            if ui.add(cta).clicked() {
                confirm_filter(app);
            }
        });
        ui.add_space(8.0);
        if ui.button("Annulla").clicked() {
            close_popup(app);
        }
    });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn confirm_filter(app: &mut App) {
    let field  = app.filter_ui.popup_field.clone().unwrap_or_default();
    let scope  = app.filter_ui.popup_scope.clone().unwrap_or_default();
    let values = app.filter_ui.popup_staged.clone();

    if let Some(idx) = app.filter_ui.popup_target_idx {
        // Add values to existing card
        if let Some(card) = app.filter_ui.active_filters.get_mut(idx) {
            for v in values {
                if !card.values.contains(&v) {
                    card.values.push(v);
                }
            }
        }
    } else {
        // Create new card
        app.filter_ui.active_filters.push(DemoFilterCard {
            field,
            scope,
            values,
            collapsed: false,
        });
    }

    close_popup(app);
}

fn close_popup(app: &mut App) {
    app.filter_ui.popup_open = false;
    app.filter_ui.popup_step = FilterPopupStep::Step1ChooseField;
    app.filter_ui.popup_target_idx = None;
    app.filter_ui.popup_field = None;
    app.filter_ui.popup_scope = None;
    app.filter_ui.popup_search.clear();
    app.filter_ui.popup_staged.clear();
}
