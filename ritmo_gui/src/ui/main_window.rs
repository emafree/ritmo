use crate::app::App;
use crate::config::ViewMode;
use crate::events::{Message, TabState};
use crate::ui::{menu, palette::UiPalette, tabs};

/// Render the main window
pub fn render(app: &mut App, ctx: &egui::Context) {
    // Build palette from current theme
    let p = UiPalette::from_settings(&app.settings.theme_mode, &app.settings.custom_themes);

    // Global keyboard shortcuts for view mode switching
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::L)) {
        app.handle_message(Message::ViewModeChanged(ViewMode::List));
    }
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::G)) {
        app.handle_message(Message::ViewModeChanged(ViewMode::Grid));
    }

    // ── Topbar ─────────────────────────────────────────────────────────────
    egui::TopBottomPanel::top("top_panel")
        .frame(
            egui::Frame::none()
                .fill(p.surface)
                .stroke(egui::Stroke::new(1.0, p.border))
                .inner_margin(egui::Margin::symmetric(8.0, 6.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // App name / menu
                ui.menu_button(
                    egui::RichText::new("RITMO").color(p.accent).strong(),
                    |ui| {
                        menu::render_menu_contents(app, ui);
                    },
                );

                ui.add_space(4.0);

                // ── Tab selector (segmented pill) ─────────────────────────
                render_segmented_toggle(
                    ui,
                    &p,
                    "tab_toggle",
                    &[
                        ("📚 Books",    app.active_tab == TabState::Books),
                        ("📄 Contents", app.active_tab == TabState::Contents),
                    ],
                    |idx| {
                        let new_tab = if idx == 0 { TabState::Books } else { TabState::Contents };
                        app.active_tab = new_tab;
                        app.settings.last_tab = new_tab;
                        let _ = app.settings.save();
                    },
                );

                ui.add_space(8.0);

                // ── Search box ────────────────────────────────────────────
                egui::Frame::none()
                    .fill(p.surface2)
                    .stroke(egui::Stroke::new(1.0, p.border))
                    .rounding(egui::Rounding::same(5.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                    .show(ui, |ui| {
                        ui.colored_label(p.text3, "🔍");
                        ui.add(
                            egui::TextEdit::singleline(&mut app.filter_ui.search_query)
                                .hint_text("Cerca…")
                                .desired_width(220.0)
                                .frame(false),
                        );
                    });

                ui.add_space(8.0);

                // ── View mode toggle (segmented pill) ─────────────────────
                render_segmented_toggle(
                    ui,
                    &p,
                    "view_toggle",
                    &[
                        ("☰ List", app.settings.view_mode == ViewMode::List),
                        ("⊞ Grid", app.settings.view_mode == ViewMode::Grid),
                    ],
                    |idx| {
                        let mode = if idx == 0 { ViewMode::List } else { ViewMode::Grid };
                        app.handle_message(Message::ViewModeChanged(mode));
                    },
                );
            });
        });

    // ── Status bar ─────────────────────────────────────────────────────────
    egui::TopBottomPanel::bottom("status_bar")
        .frame(
            egui::Frame::none()
                .fill(p.surface)
                .stroke(egui::Stroke::new(1.0, p.border))
                .inner_margin(egui::Margin::symmetric(10.0, 4.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(ref msg) = app.status_message {
                    let color = if app.status_is_error { egui::Color32::from_rgb(192, 57, 43) } else { p.text3 };
                    ui.colored_label(color, egui::RichText::new(msg).size(11.0));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let books_count = app.library_state.get_books().len();
                    let contents_count = app.library_state.get_contents().len();
                    ui.colored_label(p.text3, egui::RichText::new(
                        format!("{} books · {} contents", books_count, contents_count)
                    ).size(11.0));
                });
            });
        });

    // ── Left sidebar ───────────────────────────────────────────────────────
    egui::SidePanel::left("filter_sidebar")
        .resizable(false)
        .exact_width(240.0)
        .frame(
            egui::Frame::none()
                .fill(p.surface)
                .stroke(egui::Stroke::new(1.0, p.border)),
        )
        .show(ctx, |ui| {
            render_sidebar(app, ui, &p);
        });

    // ── Main content area ──────────────────────────────────────────────────
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(p.bg).inner_margin(egui::Margin::same(0.0)))
        .show(ctx, |ui| {
            match app.active_tab {
                TabState::Books    => tabs::render_books_tab(app, ui),
                TabState::Contents => tabs::render_contents_tab(app, ui),
            }
        });
}

// ─────────────────────────────────────────────────────────────────────────────
// Sidebar
// ─────────────────────────────────────────────────────────────────────────────

fn render_sidebar(app: &mut App, ui: &mut egui::Ui, p: &UiPalette) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.set_min_width(240.0);

            // ── Section: active filters ─────────────────────────────────
            sidebar_section_title(ui, p, "Filtri attivi");

            // Render each active filter card
            let n_filters = app.filter_ui.active_filters.len();
            let mut toggle_idx: Option<usize> = None;
            let mut remove_value: Option<(usize, usize)> = None;
            let mut open_step2: Option<usize> = None;

            for idx in 0..n_filters {
                let card = &app.filter_ui.active_filters[idx];
                let field   = card.field.clone();
                let scope   = card.scope.clone();
                let mode    = card.mode;
                let values  = card.values.clone();
                let collapsed = card.collapsed;

                egui::Frame::none()
                    .fill(if !collapsed { p.active } else { p.surface2 })
                    .stroke(egui::Stroke::new(1.0, if !collapsed { p.accent2 } else { p.border }))
                    .rounding(egui::Rounding::same(5.0))
                    .outer_margin(egui::Margin { left: 8.0, right: 8.0, top: 2.0, bottom: 2.0 })
                    .show(ui, |ui| {
                        // Header row
                        let header_resp = ui.horizontal(|ui| {
                            // Dot indicator
                            let (dot_rect, _) = ui.allocate_exact_size(
                                egui::vec2(10.0, 10.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().circle_filled(dot_rect.center(), 4.0, p.accent);

                            ui.colored_label(
                                p.text,
                                egui::RichText::new(format!("{} › {} [{}]", scope, field, mode.display_name())).size(12.0).strong(),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // "+" button opens Step2 for this card
                                if ui.small_button("+").clicked() {
                                    open_step2 = Some(idx);
                                }
                                ui.colored_label(p.text3, if collapsed { "▶" } else { "▼" });
                            });
                        });

                        if header_resp.response.interact(egui::Sense::click()).clicked() {
                            toggle_idx = Some(idx);
                        }

                        // Criteria chips (shown when not collapsed)
                        if !collapsed {
                            ui.add_space(4.0);
                            for (vi, v) in values.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.add_space(12.0);
                                    // chip
                                    egui::Frame::none()
                                        .fill(p.tag)
                                        .stroke(egui::Stroke::new(1.0, p.border2))
                                        .rounding(egui::Rounding::same(10.0))
                                        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                        .show(ui, |ui| {
                                            ui.colored_label(p.tag_text, egui::RichText::new(v).size(11.0));
                                        });
                                    // remove chip button
                                    if ui.small_button("✕").clicked() {
                                        remove_value = Some((idx, vi));
                                    }
                                });
                            }
                            ui.add_space(4.0);
                        }
                    });
            }

            // Apply mutations after iteration
            if let Some(i) = toggle_idx {
                app.filter_ui.active_filters[i].collapsed ^= true;
            }
            if let Some((card_i, val_i)) = remove_value {
                app.filter_ui.active_filters[card_i].values.remove(val_i);
                if app.filter_ui.active_filters[card_i].values.is_empty() {
                    app.filter_ui.active_filters.remove(card_i);
                }
            }
            if let Some(idx) = open_step2 {
                let card = &app.filter_ui.active_filters[idx];
                app.filter_ui.popup_target_idx = Some(idx);
                app.filter_ui.popup_field = Some(card.field.clone());
                app.filter_ui.popup_scope = Some(card.scope.clone());
                app.filter_ui.popup_open = true;
                app.filter_ui.popup_search.clear();
            }

            ui.add_space(8.0);

            // "+ Nuovo filtro" button
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let btn = egui::Button::new("+ Nuovo filtro")
                    .fill(p.surface2)
                    .stroke(egui::Stroke::new(1.0, p.border2))
                    .rounding(egui::Rounding::same(5.0));
                if ui.add(btn).clicked() {
                    app.filter_ui.popup_target_idx = None;
                    app.filter_ui.popup_open = true;
                }
            });

            ui.add_space(4.0);
            ui.add(egui::Separator::default().horizontal());

            // ── Section: saved filters ──────────────────────────────────
            sidebar_section_title(ui, p, "Filtri salvati");

            for saved in &app.filter_ui.saved_filters.clone() {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    if ui.selectable_label(false,
                        egui::RichText::new(saved).size(12.0).color(p.text2)
                    ).clicked() {
                        // Demo: apply saved filter (just show a status message)
                        app.status_message = Some(format!("Filtro salvato applicato: {}", saved));
                        app.status_is_error = false;
                    }
                });
            }

            ui.add_space(8.0);
        });
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Draw a small uppercase section title (like the wireframe's sidebar-section-title).
fn sidebar_section_title(ui: &mut egui::Ui, p: &UiPalette, title: &str) {
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.colored_label(
            p.text3,
            egui::RichText::new(title.to_uppercase()).size(10.0).strong(),
        );
    });
    ui.add_space(4.0);
}

/// Render a pill-style segmented toggle.
///
/// `labels` is a slice of `(label, is_active)` pairs.
/// `on_click` is called with the clicked index.
fn render_segmented_toggle<F>(
    ui: &mut egui::Ui,
    p: &UiPalette,
    _id: &str,
    labels: &[(&str, bool)],
    mut on_click: F,
) where
    F: FnMut(usize),
{
    egui::Frame::none()
        .fill(p.surface2)
        .stroke(egui::Stroke::new(1.0, p.border))
        .rounding(egui::Rounding::same(6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for (i, (label, active)) in labels.iter().enumerate() {
                    let btn = egui::Button::new(egui::RichText::new(*label).size(12.0).color(if *active { egui::Color32::from_rgb(15, 15, 15) } else { p.text2 }))
                        .fill(if *active { p.accent } else { egui::Color32::TRANSPARENT })
                        .stroke(egui::Stroke::NONE)
                        .rounding(egui::Rounding::same(4.0))
                        .min_size(egui::vec2(0.0, 24.0));
                    let resp = ui.add(btn);
                    if resp.clicked() && !*active {
                        on_click(i);
                    }
                }
            });
        });
}
