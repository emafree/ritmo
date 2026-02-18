use crate::app::App;
use crate::config::{CustomTheme, ThemePreset};
use crate::events::Message;
use egui::{Color32, Window};

/// Render the theme editor dialog
pub fn render(app: &mut App, ctx: &egui::Context) {
    if !app.theme_editor_open {
        return;
    }
    
    // Initialize theme if not set
    if app.theme_editor_theme.is_none() {
        let theme = CustomTheme::from_preset(ThemePreset::Dark, "New Theme".to_string());
        app.theme_editor_theme = Some(theme);
    }
    
    let mut open = true;
    let mut message_to_send: Option<Message> = None;
    
    Window::new("Theme Editor")
        .open(&mut open)
        .default_width(500.0)
        .resizable(true)
        .show(ctx, |ui| {
            if let Some(theme) = &mut app.theme_editor_theme {
                ui.heading("Custom Theme Editor");
                ui.separator();
                
                // Theme name
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut theme.name);
                });
                
                ui.add_space(10.0);
                
                // Description
                ui.horizontal(|ui| {
                    ui.label("Description:");
                    let mut desc = theme.description.clone().unwrap_or_default();
                    if ui.text_edit_singleline(&mut desc).changed() {
                        theme.description = if desc.is_empty() { None } else { Some(desc) };
                    }
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.heading("Colors");
                
                // Color editors
                ui.add_space(5.0);
                render_color_picker(ui, "Background", &mut theme.background);
                ui.add_space(5.0);
                render_color_picker(ui, "Foreground", &mut theme.foreground);
                ui.add_space(5.0);
                render_color_picker(ui, "Accent", &mut theme.accent);
                ui.add_space(5.0);
                render_color_picker(ui, "Success", &mut theme.success);
                ui.add_space(5.0);
                render_color_picker(ui, "Warning", &mut theme.warning);
                ui.add_space(5.0);
                render_color_picker(ui, "Error", &mut theme.error);
                ui.add_space(5.0);
                render_color_picker(ui, "Text Color", &mut theme.text_color);
                ui.add_space(5.0);
                render_color_picker(ui, "Weak Text", &mut theme.weak_text_color);
                
                ui.add_space(10.0);
                ui.separator();
                
                // Template selector - clone name and description to avoid borrow issues
                let current_name = theme.name.clone();
                let current_desc = theme.description.clone();
                
                ui.horizontal(|ui| {
                    ui.label("Load from template:");
                    for preset in ThemePreset::all() {
                        if ui.button(preset.name()).clicked() {
                            let mut new_theme = CustomTheme::from_preset(*preset, current_name.clone());
                            new_theme.description = current_desc.clone();
                            *theme = new_theme;
                        }
                    }
                });
                
                ui.add_space(10.0);
                ui.separator();
                
                // Action buttons - clone theme to avoid borrow issues
                let theme_clone = theme.clone();
                ui.horizontal(|ui| {
                    if ui.button("Save Theme").clicked() {
                        let mut theme_to_save = theme_clone.clone();
                        theme_to_save.update_timestamp();
                        message_to_send = Some(Message::SaveCustomTheme(theme_to_save));
                    }
                    
                    if ui.button("Cancel").clicked() {
                        message_to_send = Some(Message::ThemeEditorClosed);
                    }
                });
            }
        });
    
    if !open {
        message_to_send = Some(Message::ThemeEditorClosed);
    }
    
    // Send message after window is closed
    if let Some(msg) = message_to_send {
        app.handle_message(msg);
    }
}

/// Render a color picker for a theme color
fn render_color_picker(ui: &mut egui::Ui, label: &str, color: &mut [u8; 3]) {
    ui.horizontal(|ui| {
        ui.label(format!("{:16}", label));
        
        if ui.color_edit_button_srgb(color).changed() {
            // Color has been updated by the button
        }
        
        // Show RGB values
        ui.label(format!("RGB({}, {}, {})", color[0], color[1], color[2]));
    });
}
