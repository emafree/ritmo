use crate::app::App;
use crate::events::Message;

/// Render menu contents
pub fn render_menu_contents(app: &mut App, ui: &mut egui::Ui) {
    if ui.button("Add Book").clicked() {
        app.handle_message(Message::MenuAddBook);
        ui.close_menu();
    }
    
    if ui.button("Remove Book").clicked() {
        app.handle_message(Message::MenuRemoveBook);
        ui.close_menu();
    }
    
    if ui.button("Duplicate Book").clicked() {
        app.handle_message(Message::MenuDuplicateBook);
        ui.close_menu();
    }
    
    ui.separator();
    
    if ui.button("Settings").clicked() {
        app.handle_message(Message::MenuOpenSettings);
        ui.close_menu();
    }
    
    ui.separator();
    
    if ui.button("Exit").clicked() {
        app.handle_message(Message::MenuExit);
    }
}
