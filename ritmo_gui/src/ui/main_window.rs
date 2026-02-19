use crate::app::App;
use crate::config::ViewMode;
use crate::events::{Message, TabState};
use crate::ui::{menu, tabs};

/// Render the main window
pub fn render(app: &mut App, ctx: &egui::Context) {
    // Global keyboard shortcuts for view mode switching
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::L)) {
        app.handle_message(Message::ViewModeChanged(ViewMode::List));
    }
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::G)) {
        app.handle_message(Message::ViewModeChanged(ViewMode::Grid));
    }

    // Top panel
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Menu button
            ui.menu_button("☰ Menu", |ui| {
                menu::render_menu_contents(app, ui);
            });
            
            ui.separator();
            
            // Tab selector
            ui.selectable_value(&mut app.active_tab, TabState::Books, "📚 BOOKS");
            ui.selectable_value(&mut app.active_tab, TabState::Contents, "📄 CONTENTS");
            
            // Save tab change
            if ui.input(|i| i.pointer.any_click()) {
                app.settings.last_tab = app.active_tab;
                let _ = app.settings.save();
            }

            ui.separator();

            // View mode toggle (Ctrl+L / Ctrl+G)
            let is_list = app.settings.view_mode == ViewMode::List;
            if ui.selectable_label(is_list, "☰ List").clicked() && !is_list {
                app.handle_message(Message::ViewModeChanged(ViewMode::List));
            }
            let is_grid = app.settings.view_mode == ViewMode::Grid;
            if ui.selectable_label(is_grid, "⊞ Grid").clicked() && !is_grid {
                app.handle_message(Message::ViewModeChanged(ViewMode::Grid));
            }
        });
    });
    
    // Status bar
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if let Some(ref msg) = app.status_message {
                let color = if app.status_is_error {
                    egui::Color32::RED
                } else {
                    ui.visuals().text_color()
                };
                ui.colored_label(color, msg);
            }
        });
    });
    
    // Main content area (expanded to full width)
    egui::CentralPanel::default().show(ctx, |ui| {
        match app.active_tab {
            TabState::Books => tabs::render_books_tab(app, ui),
            TabState::Contents => tabs::render_contents_tab(app, ui),
        }
    });
}
