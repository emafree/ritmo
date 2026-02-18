use crate::app::App;
use crate::events::Message;
use egui::{Color32, Window};

/// Render the theme manager dialog
pub fn render(app: &mut App, ctx: &egui::Context) {
    if !app.theme_manager_open {
        return;
    }
    
    let mut open = true;
    Window::new("Custom Themes Manager")
        .open(&mut open)
        .default_width(600.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Manage Custom Themes");
            ui.separator();
            
            if app.settings.custom_themes.is_empty() {
                ui.label("No custom themes saved.");
                ui.add_space(10.0);
            } else {
                ui.label(format!("You have {} custom theme(s):", app.settings.custom_themes.len()));
                ui.add_space(10.0);
                
                // Clone to avoid borrow issues
                let themes = app.settings.custom_themes.clone();
                
                for theme in themes.iter() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.strong(&theme.name);
                                if let Some(desc) = &theme.description {
                                    ui.label(desc);
                                }
                                ui.label(format!("Created: {}", format_timestamp(theme.created_at)));
                                if theme.updated_at != theme.created_at {
                                    ui.label(format!("Updated: {}", format_timestamp(theme.updated_at)));
                                }
                            });
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("🗑 Delete").clicked() {
                                    app.handle_message(Message::DeleteCustomTheme(theme.name.clone()));
                                }
                                
                                if ui.button("✓ Use").clicked() {
                                    app.handle_message(Message::CustomThemeSelected(theme.name.clone()));
                                }
                            });
                        });
                        
                        // Color preview
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("Colors:");
                            render_color_preview(ui, "BG", &theme.background);
                            render_color_preview(ui, "FG", &theme.foreground);
                            render_color_preview(ui, "Accent", &theme.accent);
                            render_color_preview(ui, "Success", &theme.success);
                            render_color_preview(ui, "Warning", &theme.warning);
                            render_color_preview(ui, "Error", &theme.error);
                        });
                    });
                    ui.add_space(5.0);
                }
            }
            
            ui.add_space(10.0);
            ui.separator();
            
            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("Create New Theme").clicked() {
                    app.handle_message(Message::CreateCustomTheme);
                }
                
                if ui.button("Close").clicked() {
                    app.handle_message(Message::ThemeManagerClosed);
                }
            });
        });
    
    if !open {
        app.handle_message(Message::ThemeManagerClosed);
    }
}

/// Render a small color preview box
fn render_color_preview(ui: &mut egui::Ui, label: &str, color: &[u8; 3]) {
    let color32 = Color32::from_rgb(color[0], color[1], color[2]);
    ui.label(label);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(20.0, 20.0),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(rect, 2.0, color32);
}

/// Format timestamp for display
fn format_timestamp(timestamp: i64) -> String {
    use chrono::{DateTime, Utc};
    
    if let Some(dt) = DateTime::<Utc>::from_timestamp(timestamp, 0) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        "Unknown".to_string()
    }
}
