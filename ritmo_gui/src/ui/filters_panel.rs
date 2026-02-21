use crate::app::App;
use crate::events::{FilterField, FilterValue, Message, TabState};

/// Render filters panel
pub fn render(app: &mut App, ui: &mut egui::Ui) {
    ui.heading("Filters");
    ui.separator();
    
    // Show active filters
    let filters = match app.active_tab {
        TabState::Books => app.books_filter_state.get_filters(),
        TabState::Contents => app.contents_filter_state.get_filters(),
    };
    
    let mut to_remove = None;
    
    for (i, filter) in filters.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("{}: {}", 
                filter.field.display_name(),
                filter.value.display_name()
            ));
            if ui.button("✕").clicked() {
                to_remove = Some(i);
            }
        });
    }
    
    if let Some(index) = to_remove {
        app.handle_message(Message::FilterRemoved(index));
    }
    
    ui.separator();
    
    // Add filter section
    ui.heading("Add Filter");
    
    // Field selection
    let available_fields = match app.active_tab {
        TabState::Books => vec![
            FilterField::BookAuthor,
            FilterField::BookPublisher,
            FilterField::BookSeries,
            FilterField::BookFormat,
            FilterField::BookYear,
        ],
        TabState::Contents => vec![
            FilterField::ContentAuthor,
            FilterField::ContentType,
            FilterField::ContentYear,
        ],
    };
    
    for field in available_fields {
        if ui.button(field.display_name()).clicked() {
            // Show value selection popup
            show_value_selector(app, ui, field);
        }
    }
    
    ui.separator();
    
    if ui.button("Clear All Filters").clicked() {
        app.handle_message(Message::FilterCleared);
    }
}

fn show_value_selector(app: &mut App, ui: &mut egui::Ui, field: FilterField) {
    // Get available values based on field
    let values: Vec<String> = match &field {
        FilterField::BookAuthor => app.library_state.get_book_authors(),
        FilterField::BookPublisher => app.library_state.get_publishers(),
        FilterField::BookFormat => app.library_state.get_formats(),
        FilterField::ContentAuthor => app.library_state.get_content_authors(),
        FilterField::ContentType => app.library_state.get_content_types(),
        _ => vec![],
    };
    
    ui.menu_button("Select Value", |ui| {
        // Special values
        if ui.button("NESSUNO").clicked() {
            add_filter(app, field, FilterValue::Nessuno);
            ui.close_menu();
        }
        
        if ui.button("ALMENO UNO").clicked() {
            add_filter(app, field, FilterValue::AlmenoUno);
            ui.close_menu();
        }
        
        ui.separator();
        
        // Specific values
        for value in values {
            if ui.button(&value).clicked() {
                add_filter(app, field, FilterValue::Specific(vec![value]));
                ui.close_menu();
            }
        }
    });
}

fn add_filter(app: &mut App, field: FilterField, value: FilterValue) {
    match app.active_tab {
        TabState::Books => {
            app.books_filter_state.add_filter(field, value);
        }
        TabState::Contents => {
            app.contents_filter_state.add_filter(field, value);
        }
    }
    app.handle_message(Message::FilterAdded);
}
