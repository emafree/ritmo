pub mod main_window;
pub mod menu;
pub mod filters_panel;
pub mod tabs;
pub mod widgets;
pub mod dialogs;
pub mod palette;

use crate::app::App;

/// Main UI rendering function
pub fn render(app: &mut App, ctx: &egui::Context) {
    main_window::render(app, ctx);
    
    // Render dialogs
    dialogs::theme_editor::render(app, ctx);
    dialogs::theme_manager::render(app, ctx);
    dialogs::filter_popup::render(app, ctx);
}
