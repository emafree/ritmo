use crate::config::{GuiSettings, ThemeConfig};
use crate::events::{Message, TabState};
use crate::state::{LibraryState, BooksFilterState, ContentsFilterState};
use std::path::PathBuf;

/// Main application state
pub struct App {
    /// Library data management
    pub library_state: LibraryState,
    
    /// Filter state for books tab
    pub books_filter_state: BooksFilterState,
    
    /// Filter state for contents tab
    pub contents_filter_state: ContentsFilterState,
    
    /// Currently active tab
    pub active_tab: TabState,
    
    /// GUI settings
    pub settings: GuiSettings,
    
    /// Selected book ID
    pub selected_book_id: Option<i64>,
    
    /// Selected content ID
    pub selected_content_id: Option<i64>,
    
    /// Status message
    pub status_message: Option<String>,
    
    /// Error flag for status message
    pub status_is_error: bool,
}

impl App {
    /// Create a new App instance
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Load settings
        let settings = GuiSettings::load();
        
        // Determine library path
        let library_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("RitmoLibrary");
        
        // Create library state
        let mut library_state = LibraryState::new(library_path.clone())
            .expect("Failed to create library state");
        
        // Initialize library
        if let Err(e) = library_state.initialize() {
            eprintln!("Warning: Failed to initialize library: {}", e);
        }
        
        // Load filter states from settings
        let books_filter_state = settings
            .last_books_filter
            .as_ref()
            .map(|json| BooksFilterState::from_json(json))
            .unwrap_or_default();
        
        let contents_filter_state = settings
            .last_contents_filter
            .as_ref()
            .map(|json| ContentsFilterState::from_json(json))
            .unwrap_or_default();
        
        // Initial data load
        let _ = library_state.refresh_books(books_filter_state.to_book_filters());
        let _ = library_state.refresh_contents(contents_filter_state.to_content_filters());
        
        Self {
            library_state,
            books_filter_state,
            contents_filter_state,
            active_tab: settings.last_tab,
            settings,
            selected_book_id: None,
            selected_content_id: None,
            status_message: Some(format!("Library loaded: {}", library_path.display())),
            status_is_error: false,
        }
    }
    
    /// Handle messages
    pub fn handle_message(&mut self, message: Message) {
        match message {
            Message::TabSelected(tab) => {
                self.active_tab = tab;
                self.settings.last_tab = tab;
                let _ = self.settings.save();
            }
            
            Message::BookSelected(id) => {
                self.selected_book_id = Some(id);
            }
            
            Message::BookDoubleClicked(id) => {
                self.status_message = Some(format!("Book detail (ID: {}) - Coming in Phase 2", id));
                self.status_is_error = false;
            }
            
            Message::ContentSelected(id) => {
                self.selected_content_id = Some(id);
            }
            
            Message::ContentDoubleClicked(id) => {
                self.status_message = Some(format!("Content detail (ID: {}) - Coming in Phase 2", id));
                self.status_is_error = false;
            }
            
            Message::FilterAdded => {
                // Filter addition is handled in the UI
                self.refresh_current_tab();
            }
            
            Message::FilterRemoved(index) => {
                match self.active_tab {
                    TabState::Books => self.books_filter_state.remove_filter(index),
                    TabState::Contents => self.contents_filter_state.remove_filter(index),
                }
                self.refresh_current_tab();
            }
            
            Message::FilterCleared => {
                match self.active_tab {
                    TabState::Books => self.books_filter_state.clear(),
                    TabState::Contents => self.contents_filter_state.clear(),
                }
                self.refresh_current_tab();
            }
            
            Message::MenuAddBook => {
                self.status_message = Some("Add Book - Coming in Phase 2".to_string());
                self.status_is_error = false;
            }
            
            Message::MenuRemoveBook => {
                self.status_message = Some("Remove Book - Coming in Phase 2".to_string());
                self.status_is_error = false;
            }
            
            Message::MenuDuplicateBook => {
                self.status_message = Some("Duplicate Book - Coming in Phase 2".to_string());
                self.status_is_error = false;
            }
            
            Message::MenuOpenSettings => {
                self.status_message = Some("Settings - Coming in Phase 2".to_string());
                self.status_is_error = false;
            }
            
            Message::MenuExit => {
                std::process::exit(0);
            }
            
            Message::ThemeChanged(theme) => {
                self.settings.theme = theme;
                let _ = self.settings.save();
            }
            
            _ => {}
        }
    }
    
    /// Refresh data for current tab
    fn refresh_current_tab(&mut self) {
        match self.active_tab {
            TabState::Books => {
                let filters = self.books_filter_state.to_book_filters();
                if let Err(e) = self.library_state.refresh_books(filters) {
                    self.status_message = Some(format!("Error loading books: {}", e));
                    self.status_is_error = true;
                } else {
                    self.status_message = Some(format!("Loaded {} books", self.library_state.get_books().len()));
                    self.status_is_error = false;
                    
                    // Save filter state
                    self.settings.last_books_filter = self.books_filter_state.to_json();
                    let _ = self.settings.save();
                }
            }
            TabState::Contents => {
                let filters = self.contents_filter_state.to_content_filters();
                if let Err(e) = self.library_state.refresh_contents(filters) {
                    self.status_message = Some(format!("Error loading contents: {}", e));
                    self.status_is_error = true;
                } else {
                    self.status_message = Some(format!("Loaded {} contents", self.library_state.get_contents().len()));
                    self.status_is_error = false;
                    
                    // Save filter state
                    self.settings.last_contents_filter = self.contents_filter_state.to_json();
                    let _ = self.settings.save();
                }
            }
        }
    }
    
    /// Get current theme visuals
    pub fn get_visuals(&self) -> egui::Visuals {
        ThemeConfig::get_visuals(self.settings.theme)
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set theme
        ctx.set_visuals(self.get_visuals());
        
        // Render UI
        crate::ui::render(self, ctx);
    }
}
