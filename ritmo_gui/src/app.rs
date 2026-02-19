use crate::config::{GuiSettings, ThemeConfig, ThemeMode, ThemePreset, CustomTheme};
use crate::events::{Message, TabState};
use crate::state::{LibraryState, BooksFilterState, ContentsFilterState};
use std::collections::HashMap;
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
    
    /// Theme editor dialog state
    pub theme_editor_open: bool,
    pub theme_editor_theme: Option<CustomTheme>,
    
    /// Theme manager dialog state
    pub theme_manager_open: bool,
    
    /// Thumbnail texture cache: keyed by book id.
    /// The thumbnail path is resolved as `library_root/covers/thumbnails/{id}.jpg`.
    /// (SHA-256 is not exposed in BookSummary; the book id is used as a surrogate key.)
    pub thumbnail_cache: HashMap<i64, egui::TextureHandle>,
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
            theme_editor_open: false,
            theme_editor_theme: None,
            theme_manager_open: false,
            thumbnail_cache: HashMap::new(),
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
            
            Message::ViewModeChanged(mode) => {
                if self.settings.view_mode != mode {
                    self.settings.view_mode = mode;
                    let _ = self.settings.save();
                }
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
            
            Message::ThemePresetSelected(preset) => {
                self.settings.theme_mode = ThemeMode::Preset(preset);
                let _ = self.settings.save();
                self.status_message = Some(format!("Theme changed to: {}", preset.name()));
                self.status_is_error = false;
            }
            
            Message::CustomThemeSelected(name) => {
                self.settings.theme_mode = ThemeMode::Custom(name.clone());
                let _ = self.settings.save();
                self.status_message = Some(format!("Theme changed to: {}", name));
                self.status_is_error = false;
            }
            
            Message::ThemeEditorOpened => {
                self.theme_editor_open = true;
            }
            
            Message::ThemeEditorClosed => {
                self.theme_editor_open = false;
                self.theme_editor_theme = None;
            }
            
            Message::ThemeManagerOpened => {
                self.theme_manager_open = true;
            }
            
            Message::ThemeManagerClosed => {
                self.theme_manager_open = false;
            }
            
            Message::CreateCustomTheme => {
                // Create a new theme from current preset
                let base_preset = match &self.settings.theme_mode {
                    ThemeMode::Preset(p) => *p,
                    ThemeMode::Custom(_) => ThemePreset::Dark,
                };
                let theme = CustomTheme::from_preset(base_preset, "New Theme".to_string());
                self.theme_editor_theme = Some(theme);
                self.theme_editor_open = true;
            }
            
            Message::SaveCustomTheme(theme) => {
                self.settings.save_custom_theme(theme.clone());
                let _ = self.settings.save();
                self.settings.theme_mode = ThemeMode::Custom(theme.name.clone());
                self.theme_editor_open = false;
                self.theme_editor_theme = None;
                self.status_message = Some(format!("Theme '{}' saved", theme.name));
                self.status_is_error = false;
            }
            
            Message::DeleteCustomTheme(name) => {
                if self.settings.delete_custom_theme(&name) {
                    // If we're currently using this theme, switch to Dark
                    if let ThemeMode::Custom(current_name) = &self.settings.theme_mode {
                        if current_name == &name {
                            self.settings.theme_mode = ThemeMode::Preset(ThemePreset::Dark);
                        }
                    }
                    let _ = self.settings.save();
                    self.status_message = Some(format!("Theme '{}' deleted", name));
                    self.status_is_error = false;
                } else {
                    self.status_message = Some(format!("Theme '{}' not found", name));
                    self.status_is_error = true;
                }
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
    
    /// Get or load a thumbnail texture for a book.
    ///
    /// Thumbnail path: `library_root/covers/thumbnails/{book_id}.jpg`
    /// (SHA-256 is not present in BookSummary; the book id is used as a surrogate
    /// identifier for deterministic thumbnail lookup.)
    ///
    /// Returns `None` if the file does not exist or cannot be decoded; callers
    /// should show a placeholder in that case.
    pub fn get_thumbnail(
        &mut self,
        ctx: &egui::Context,
        book_id: i64,
    ) -> Option<&egui::TextureHandle> {
        // Return cached texture if already loaded
        if self.thumbnail_cache.contains_key(&book_id) {
            return self.thumbnail_cache.get(&book_id);
        }

        // Resolve path: library_root/covers/thumbnails/{book_id}.jpg
        let path = self.library_state.library_root()
            .join("covers")
            .join("thumbnails")
            .join(format!("{}.jpg", book_id));

        // Load and decode the JPEG; fail silently → placeholder
        let texture = (|| -> Option<egui::TextureHandle> {
            let data = std::fs::read(&path).ok()?;
            let img = image::load_from_memory_with_format(&data, image::ImageFormat::Jpeg).ok()?;
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [w as usize, h as usize],
                &rgba,
            );
            Some(ctx.load_texture(
                format!("thumb_{}", book_id),
                color_image,
                egui::TextureOptions::LINEAR,
            ))
        })();

        if let Some(tex) = texture {
            self.thumbnail_cache.insert(book_id, tex);
            self.thumbnail_cache.get(&book_id)
        } else {
            None
        }
    }

    /// Get current theme visuals
    pub fn get_visuals(&self) -> egui::Visuals {
        ThemeConfig::get_visuals(&self.settings.theme_mode, &self.settings.custom_themes)
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
