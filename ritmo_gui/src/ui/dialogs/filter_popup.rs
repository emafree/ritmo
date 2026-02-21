use egui::{Align2, Color32, Frame, Key, Margin, Vec2};
use crate::app::{App, DemoFilterCard};
use crate::events::{FilterField, FilterMode, FilterValue, Message, TabState};
use crate::ui::palette::UiPalette;

// ---------------------------------------------------------------------------
// Filterable criteria with demo ML suggestion data
// ---------------------------------------------------------------------------

/// Book-scope filterable criteria with demo autocomplete items.
const BOOK_FIELDS: &[(&str, &str, &[&str])] = &[
    ("Book", "Title",     &["Il nome della rosa", "1984", "Dune", "Fondazione", "Il Signore degli Anelli"]),
    ("Book", "Author",    &["Tolkien, J.R.R.", "Pratchett, Terry", "Le Guin, Ursula K.", "Martin, G.R.R.", "Asimov, Isaac"]),
    ("Book", "Publisher", &["HarperCollins", "Tor Books", "Orbit", "Penguin", "Del Rey"]),
    ("Book", "Tag",       &["fantasy", "sci-fi", "classic", "award-winner", "italian"]),
    ("Book", "Series",    &["Lord of the Rings", "Discworld", "Earthsea", "Foundation", "A Song of Ice and Fire"]),
    ("Book", "Format",    &["EPUB", "PDF", "MOBI", "AZW3", "CBZ"]),
    ("Book", "Year",      &["2020", "2021", "2022", "2023", "2024"]),
];

/// Content-scope filterable criteria with demo autocomplete items.
const CONTENT_FIELDS: &[(&str, &str, &[&str])] = &[
    ("Content", "Title",      &["La veglia del re stregone", "Preludio alla Fondazione", "Nessuno è indispensabile"]),
    ("Content", "Author",     &["Tolkien, J.R.R.", "Pratchett, Terry", "Le Guin, Ursula K."]),
    ("Content", "Translator", &["Bianchi, Marco", "Rossi, Elena", "Ferrari, Giovanni"]),
    ("Content", "Tag",        &["fantasy", "short-story", "translated", "award-winner"]),
    ("Content", "Type",       &["Short Story", "Novel", "Novella", "Collection", "Anthology"]),
    ("Content", "Year",       &["2020", "2021", "2022", "2023", "2024"]),
];

/// All criteria in order: scope label + field label pairs.
fn all_criteria() -> Vec<(&'static str, &'static str)> {
    let mut out = Vec::new();
    for (scope, field, _) in BOOK_FIELDS {
        out.push((*scope, *field));
    }
    for (scope, field, _) in CONTENT_FIELDS {
        out.push((*scope, *field));
    }
    out
}

/// Return ML-style suggestions for the given scope/field/query.
fn suggestions_for(scope: &str, field: &str, query: &str) -> Vec<String> {
    let table: &[(&str, &str, &[&str])] = if scope == "Book" { BOOK_FIELDS } else { CONTENT_FIELDS };
    for (_, f, items) in table {
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

    // Modal window
    let mut open = true;
    egui::Window::new("##filter_popup")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size([520.0, 380.0])
        .frame(
            Frame::window(&ctx.style())
                .fill(palette.surface)
                .stroke(egui::Stroke::new(1.0, palette.border2))
                .inner_margin(Margin::same(0.0))
                .rounding(egui::Rounding::same(8.0)),
        )
        .open(&mut open)
        .show(ctx, |ui| {
            render_popup_contents(app, ui, &palette);
        });

    if !open {
        close_popup(app);
    }
}

// ---------------------------------------------------------------------------
// Popup contents — single 3-field row
// ---------------------------------------------------------------------------

fn render_popup_contents(app: &mut App, ui: &mut egui::Ui, palette: &UiPalette) {
    // Header bar
    let header_rect = {
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 44.0),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(
            rect,
            egui::Rounding { nw: 8.0, ne: 8.0, sw: 0.0, se: 0.0 },
            palette.surface2,
        );
        painter.hline(rect.x_range(), rect.bottom(), egui::Stroke::new(1.0, palette.border));
        rect
    };

    // Header label + close button
    {
        let mut header_ui = ui.child_ui(header_rect, egui::Layout::left_to_right(egui::Align::Center));
        header_ui.add_space(14.0);
        header_ui.colored_label(
            palette.text,
            egui::RichText::new("Nuovo filtro").size(13.0).strong(),
        );

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
            render_filter_row(app, ui, palette);
        });
}

// ---------------------------------------------------------------------------
// 3-field filter row: [criterion] [mode] [target + ML suggestions]
// ---------------------------------------------------------------------------

fn render_filter_row(app: &mut App, ui: &mut egui::Ui, palette: &UiPalette) {
    ui.colored_label(palette.text2, "Definisci il criterio di filtro:");
    ui.add_space(10.0);

    // --- Row: three fields side by side ---
    let criteria = all_criteria();

    // Determine current criterion label for the ComboBox
    let criterion_label = match (&app.filter_ui.popup_scope, &app.filter_ui.popup_field) {
        (Some(s), Some(f)) => format!("{}-{}", s, f),
        _ => "Seleziona…".to_string(),
    };

    ui.horizontal(|ui| {
        // Field 1 – criterion ComboBox
        egui::ComboBox::from_id_source("filter_criterion")
            .selected_text(
                egui::RichText::new(&criterion_label).size(12.0),
            )
            .width(170.0)
            .show_ui(ui, |ui| {
                let mut last_scope = "";
                for (scope, field) in &criteria {
                    if *scope != last_scope {
                        ui.colored_label(
                            palette.text3,
                            egui::RichText::new(*scope).size(10.0).strong(),
                        );
                        last_scope = scope;
                    }
                    let label = format!("{}-{}", scope, field);
                    let selected = app.filter_ui.popup_scope.as_deref() == Some(scope)
                        && app.filter_ui.popup_field.as_deref() == Some(field);
                    if ui.selectable_label(selected, label).clicked() {
                        app.filter_ui.popup_scope = Some(scope.to_string());
                        app.filter_ui.popup_field = Some(field.to_string());
                        app.filter_ui.popup_search.clear();
                    }
                }
            });

        ui.add_space(6.0);

        // Field 2 – mode ComboBox
        egui::ComboBox::from_id_source("filter_mode")
            .selected_text(
                egui::RichText::new(app.filter_ui.popup_mode.display_name()).size(12.0),
            )
            .width(100.0)
            .show_ui(ui, |ui| {
                for mode in FilterMode::all() {
                    let selected = app.filter_ui.popup_mode == *mode;
                    if ui.selectable_label(selected, mode.display_name()).clicked() {
                        app.filter_ui.popup_mode = *mode;
                    }
                }
            });

        ui.add_space(6.0);

        // Field 3 – target text input
        let search_resp = ui.add(
            egui::TextEdit::singleline(&mut app.filter_ui.popup_search)
                .hint_text("Valore…")
                .desired_width(ui.available_width()),
        );
        // Request focus on the target field when a criterion is selected
        if app.filter_ui.popup_field.is_some() && !search_resp.has_focus() {
            search_resp.request_focus();
        }
    });

    ui.add_space(8.0);

    // --- ML suggestions list ---
    let scope = app.filter_ui.popup_scope.clone().unwrap_or_default();
    let field = app.filter_ui.popup_field.clone().unwrap_or_default();
    let suggestions = if !field.is_empty() {
        suggestions_for(&scope, &field, &app.filter_ui.popup_search)
    } else {
        vec![]
    };

    let mut chosen_suggestion: Option<String> = None;

    if !suggestions.is_empty() || (!app.filter_ui.popup_search.is_empty() && !field.is_empty()) {
        egui::Frame::none()
            .fill(palette.surface2)
            .stroke(egui::Stroke::new(1.0, palette.border))
            .rounding(egui::Rounding::same(4.0))
            .inner_margin(Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                ui.colored_label(
                    palette.text3,
                    egui::RichText::new("Suggerimenti ML").size(10.0),
                );
                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .show(ui, |ui| {
                        for s in &suggestions {
                            if ui
                                .selectable_label(false, egui::RichText::new(s).size(12.0))
                                .clicked()
                            {
                                chosen_suggestion = Some(s.clone());
                            }
                        }
                        // Free-text add option
                        if !app.filter_ui.popup_search.is_empty()
                            && !suggestions
                                .iter()
                                .any(|s| s.eq_ignore_ascii_case(&app.filter_ui.popup_search))
                        {
                            ui.separator();
                            let new_val = app.filter_ui.popup_search.clone();
                            if ui
                                .button(
                                    egui::RichText::new(format!("+ \"{}\"", new_val)).size(12.0),
                                )
                                .clicked()
                            {
                                chosen_suggestion = Some(new_val);
                            }
                        }
                    });
            });
        ui.add_space(8.0);
    }

    if let Some(v) = chosen_suggestion {
        app.filter_ui.popup_search = v;
    }

    // --- CTA buttons ---
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(8.0);

    let row_valid = app.filter_ui.popup_field.is_some()
        && !app.filter_ui.popup_search.is_empty();
    let can_accept = row_valid || !app.filter_ui.active_filters.is_empty();

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.add_enabled_ui(can_accept, |ui| {
            let cta = egui::Button::new(
                egui::RichText::new("Accetta").size(12.0),
            )
            .fill(palette.accent)
            .stroke(egui::Stroke::NONE)
            .rounding(egui::Rounding::same(5.0));
            if ui.add(cta).clicked() {
                accept_filter(app);
            }
        });
        ui.add_space(8.0);
        ui.add_enabled_ui(row_valid, |ui| {
            if ui.button("Continua").clicked() {
                confirm_filter(app);
            }
        });
        ui.add_space(8.0);
        if ui.button("Chiudi").clicked() {
            close_popup(app);
        }
    });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Confirm the current row: add a filter card and reset the row for the next entry.
fn confirm_filter(app: &mut App) {
    let field = app.filter_ui.popup_field.clone().unwrap_or_default();
    let scope = app.filter_ui.popup_scope.clone().unwrap_or_default();
    let mode  = app.filter_ui.popup_mode;
    let value = app.filter_ui.popup_search.clone();

    if field.is_empty() || value.is_empty() {
        return;
    }

    app.filter_ui.active_filters.push(DemoFilterCard {
        field,
        scope,
        mode,
        values: vec![value],
        collapsed: false,
    });

    // Reset the row for a new entry (popup stays open)
    app.filter_ui.popup_field = None;
    app.filter_ui.popup_scope = None;
    app.filter_ui.popup_mode = FilterMode::default();
    app.filter_ui.popup_search.clear();
    app.filter_ui.popup_target_idx = None;
}

fn close_popup(app: &mut App) {
    app.filter_ui.popup_open = false;
    app.filter_ui.popup_target_idx = None;
    app.filter_ui.popup_field = None;
    app.filter_ui.popup_scope = None;
    app.filter_ui.popup_mode = FilterMode::default();
    app.filter_ui.popup_search.clear();
    app.filter_ui.active_filters.clear();
}

/// Map a DemoFilterCard scope/field pair to a FilterField enum value.
fn demo_card_to_filter_field(scope: &str, field: &str) -> Option<FilterField> {
    match (scope, field) {
        ("Book", "Author")    => Some(FilterField::BookAuthor),
        ("Book", "Publisher") => Some(FilterField::BookPublisher),
        ("Book", "Series")    => Some(FilterField::BookSeries),
        ("Book", "Format")    => Some(FilterField::BookFormat),
        ("Book", "Year")      => Some(FilterField::BookYear),
        ("Content", "Author") => Some(FilterField::ContentAuthor),
        ("Content", "Type")   => Some(FilterField::ContentType),
        ("Content", "Year")   => Some(FilterField::ContentYear),
        _ => None,
    }
}

/// Accept all accumulated rows: optionally save current row, apply rows to the
/// effective filter state for the active tab, then close the popup.
fn accept_filter(app: &mut App) {
    // confirm_filter internally checks validity and is a no-op if the row is empty
    confirm_filter(app);

    // Drain and apply all accumulated rows (each card holds exactly one value,
    // as set by confirm_filter via `values: vec![value]`)
    let cards: Vec<DemoFilterCard> = app.filter_ui.active_filters.drain(..).collect();
    for card in cards {
        let value = match card.values.into_iter().next() {
            Some(v) if !v.is_empty() => v,
            _ => continue,
        };
        if let Some(field) = demo_card_to_filter_field(&card.scope, &card.field) {
            let filter_value = FilterValue::Specific(value);
            match app.active_tab {
                TabState::Books => app.books_filter_state.add_filter(field, filter_value),
                TabState::Contents => app.contents_filter_state.add_filter(field, filter_value),
            }
        }
    }
    app.handle_message(Message::FilterAdded);
    close_popup(app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_field_mapping() {
        assert_eq!(demo_card_to_filter_field("Book", "Author"),    Some(FilterField::BookAuthor));
        assert_eq!(demo_card_to_filter_field("Book", "Publisher"), Some(FilterField::BookPublisher));
        assert_eq!(demo_card_to_filter_field("Book", "Series"),    Some(FilterField::BookSeries));
        assert_eq!(demo_card_to_filter_field("Book", "Format"),    Some(FilterField::BookFormat));
        assert_eq!(demo_card_to_filter_field("Book", "Year"),      Some(FilterField::BookYear));
    }

    #[test]
    fn test_content_field_mapping() {
        assert_eq!(demo_card_to_filter_field("Content", "Author"), Some(FilterField::ContentAuthor));
        assert_eq!(demo_card_to_filter_field("Content", "Type"),   Some(FilterField::ContentType));
        assert_eq!(demo_card_to_filter_field("Content", "Year"),   Some(FilterField::ContentYear));
    }

    #[test]
    fn test_unknown_field_mapping() {
        assert_eq!(demo_card_to_filter_field("Book", "Title"),       None);
        assert_eq!(demo_card_to_filter_field("Book", "Tag"),         None);
        assert_eq!(demo_card_to_filter_field("Content", "Translator"), None);
        assert_eq!(demo_card_to_filter_field("Unknown", "Author"),   None);
    }
}
