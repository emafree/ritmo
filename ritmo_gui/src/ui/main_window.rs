use crate::app::App;
use crate::events::TabState;
use crate::ui::{menu, filters_panel, tabs};

/// Render the main window
pub fn render(app: &mut App, ctx: &egui::Context) {
    // Top panel
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Menu button
            if ui.button("☰").clicked() {
                ui.menu_button("Menu", |ui| {
                    menu::render_menu_contents(app, ui);
                });
            }
            
            ui.separator();
            
            // Filters button
            if ui.button("⚙ Filters").clicked() {
                // Toggle will be handled by panel state
            }
            
            ui.separator();
            
            // Tab selector
            ui.selectable_value(&mut app.active_tab, TabState::Books, "📚 BOOKS");
            ui.selectable_value(&mut app.active_tab, TabState::Contents, "📄 CONTENTS");
            
            // Save tab change
            if ui.input(|i| i.pointer.any_click()) {
                app.settings.last_tab = app.active_tab;
                let _ = app.settings.save();
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
    
    // Filters side panel
    egui::SidePanel::left("filters_panel")
        .default_width(250.0)
        .show(ctx, |ui| {
            filters_panel::render(app, ui);
        });
    
    // Main content area
    egui::CentralPanel::default().show(ctx, |ui| {
        match app.active_tab {
            TabState::Books => tabs::render_books_tab(app, ui),
            TabState::Contents => tabs::render_contents_tab(app, ui),
        }
    });
}
