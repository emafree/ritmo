pub mod main_window;
pub mod menu;
pub mod filters_panel;
pub mod tabs;
pub mod widgets;

use crate::app::App;

/// Main UI rendering function
pub fn render(app: &mut App, ctx: &egui::Context) {
    main_window::render(app, ctx);
}
