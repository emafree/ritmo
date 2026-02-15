use crate::app::App;
use crate::events::Message;

/// Render books tab
pub fn render_books_tab(app: &mut App, ui: &mut egui::Ui) {
    ui.heading("Books");
    ui.separator();
    
    let books = app.library_state.get_books().to_vec();
    let selected_book_id = app.selected_book_id;
    
    if books.is_empty() {
        ui.label("No books found. Try adjusting your filters or add books to your library.");
        return;
    }
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        for book in &books {
            let is_selected = selected_book_id == Some(book.id);
            
            let response = ui.selectable_label(is_selected, format!(
                "📚 {} {}",
                book.title,
                book.authors.join(", ")
            ));
            
            if response.clicked() {
                app.handle_message(Message::BookSelected(book.id));
            }
            
            if response.double_clicked() {
                app.handle_message(Message::BookDoubleClicked(book.id));
            }
            
            // Show additional info when selected
            if is_selected {
                ui.indent("book_details", |ui| {
                    if let Some(ref publisher) = book.publisher {
                        ui.label(format!("Publisher: {}", publisher));
                    }
                    if let Some(year) = book.year {
                        ui.label(format!("Year: {}", year));
                    }
                    if let Some(ref format) = book.format {
                        ui.label(format!("Format: {}", format));
                    }
                    if let Some(size) = book.file_size {
                        ui.label(format!("Size: {} bytes", size));
                    }
                });
            }
            
            ui.separator();
        }
    });
}
