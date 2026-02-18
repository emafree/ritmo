use crate::app::App;
use crate::events::TabState;
use crate::ui::{menu, tabs};

/// Render the main window
pub fn render(app: &mut App, ctx: &egui::Context) {
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
