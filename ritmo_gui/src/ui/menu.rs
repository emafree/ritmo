use crate::app::App;
use crate::config::ThemePreset;
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
    
    // Themes submenu
    ui.menu_button("Themes", |ui| {
        ui.label("Preset Themes:");
        ui.separator();
        
        for preset in ThemePreset::all() {
            if ui.button(preset.name()).clicked() {
                app.handle_message(Message::ThemePresetSelected(*preset));
                ui.close_menu();
            }
        }
        
        ui.separator();
        ui.label("Custom Themes:");
        ui.separator();
        
        if ui.button("Create New Theme").clicked() {
            app.handle_message(Message::CreateCustomTheme);
            ui.close_menu();
        }
        
        if ui.button("Manage Custom Themes").clicked() {
            app.handle_message(Message::ThemeManagerOpened);
            ui.close_menu();
        }
        
        // Show custom themes if any exist - clone to avoid borrow issues
        let custom_themes: Vec<_> = app.settings.custom_themes.iter()
            .map(|t| t.name.clone())
            .collect();
        
        if !custom_themes.is_empty() {
            ui.separator();
            for theme_name in custom_themes {
                if ui.button(&theme_name).clicked() {
                    app.handle_message(Message::CustomThemeSelected(theme_name));
                    ui.close_menu();
                }
            }
        }
    });
    
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
