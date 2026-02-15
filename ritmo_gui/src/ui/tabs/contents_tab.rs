use crate::app::App;
use crate::events::Message;

/// Render contents tab
pub fn render_contents_tab(app: &mut App, ui: &mut egui::Ui) {
    ui.heading("Contents");
    ui.separator();
    
    let contents = app.library_state.get_contents().to_vec();
    let selected_content_id = app.selected_content_id;
    
    if contents.is_empty() {
        ui.label("No contents found. Try adjusting your filters or add content to your library.");
        return;
    }
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        for content in &contents {
            let is_selected = selected_content_id == Some(content.id);
            
            let response = ui.selectable_label(is_selected, format!(
                "📄 {} {}",
                content.title,
                content.authors.join(", ")
            ));
            
            if response.clicked() {
                app.handle_message(Message::ContentSelected(content.id));
            }
            
            if response.double_clicked() {
                app.handle_message(Message::ContentDoubleClicked(content.id));
            }
            
            // Show additional info when selected
            if is_selected {
                ui.indent("content_details", |ui| {
                    if let Some(ref content_type) = content.content_type {
                        ui.label(format!("Type: {}", content_type));
                    }
                    if let Some(year) = content.year {
                        ui.label(format!("Year: {}", year));
                    }
                    if let Some(pages) = content.pages {
                        ui.label(format!("Pages: {}", pages));
                    }
                });
            }
            
            ui.separator();
        }
    });
}
