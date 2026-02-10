slint::include_modules!();

use ritmo_config::{config_dir, settings_file, AppSettings};
use ritmo_db::{get_books_with_nested_data, get_contents_with_nested_data};
use ritmo_db_core::LibraryConfig;
use ritmo_errors::reporter::SilentReporter;
use slint::{Model, ModelRc, SharedString, VecModel};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::Mutex;

// Helper function to convert Vec into ModelRc
fn to_model<T: Clone + 'static>(v: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(v)))
}

// Application state with multi-library support
struct AppState {
    app_settings: AppSettings,
    settings_path: PathBuf,
    current_library: Option<LibraryConfig>,
    runtime: tokio::runtime::Runtime,
}

impl AppState {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize config directory
        let config_dir = config_dir()?;
        std::fs::create_dir_all(&config_dir)?;

        let settings_path = settings_file()?;
        let app_settings = AppSettings::load_or_create(&settings_path)
            .map_err(|e| format!("Failed to load settings: {}", e))?;

        let runtime = tokio::runtime::Runtime::new()?;

        Ok(Self {
            app_settings,
            settings_path,
            current_library: None,
            runtime,
        })
    }

    fn load_library(&mut self, library_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from(library_path);
        let config = LibraryConfig::new(&path);

        // Check if library exists
        if !config.exists() {
            return Err(format!("Library does not exist: {}", library_path).into());
        }

        self.current_library = Some(config);
        self.app_settings.last_library_path = Some(path);
        self.app_settings
            .save(&self.settings_path)
            .map_err(|e| format!("Failed to save settings: {}", e))?;

        Ok(())
    }

    fn initialize_current_library(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref config) = self.current_library {
            config.initialize()?;
            self.runtime
                .block_on(async { config.initialize_database().await })
                .map_err(|e| format!("Failed to initialize database: {}", e))?;
            Ok(())
        } else {
            Err("No library loaded".into())
        }
    }

    fn get_books_with_contents(
        &self,
        search: Option<&str>,
    ) -> Result<Vec<BookWithContents>, Box<dyn std::error::Error>> {
        let config = self
            .current_library
            .as_ref()
            .ok_or("No library loaded")?;

        let mut reporter = SilentReporter;
        let books = self
            .runtime
            .block_on(async {
                let pool = config
                    .create_pool(&mut reporter)
                    .await
                    .map_err(|e| format!("Failed to create pool: {}", e))?;
                get_books_with_nested_data(&pool, search)
                    .await
                    .map_err(|e| format!("Query failed: {}", e))
            })
            .map_err(|e: String| -> Box<dyn std::error::Error> { e.into() })?;

        // Convert ritmo_db::BookWithDetails to Slint BookWithContents
        let slint_books: Vec<BookWithContents> = books
            .into_iter()
            .map(|book| {
                // Convert contents with people
                let contents: Vec<ContentInfo> = book
                    .contents
                    .into_iter()
                    .map(|content| {
                        // Convert people with roles
                        let people: Vec<PersonWithRole> = content
                            .people
                            .into_iter()
                            .map(|person| PersonWithRole {
                                person_id: person.person_id as i32,
                                person_name: person.person_name.into(),
                                role_name: person.role_key.into(), // Phase 2: Could translate here
                            })
                            .collect();

                        ContentInfo {
                            id: content.content_id as i32,
                            name: content.content_name.into(),
                            original_title: content.content_original_title.unwrap_or_default().into(),
                            type_name: content.content_type_key.unwrap_or_default().into(),
                            publication_date: content
                                .content_publication_date
                                .map(|ts| {
                                    use chrono::{DateTime, Utc};
                                    DateTime::<Utc>::from_timestamp(ts, 0)
                                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                                        .unwrap_or_default()
                                })
                                .unwrap_or_default()
                                .into(),
                            people: to_model(people),
                        }
                    })
                    .collect();

                BookWithContents {
                    id: book.book_id as i32,
                    name: book.book_name.into(),
                    original_title: book.book_original_title.unwrap_or_default().into(),
                    publisher: book.book_publisher.unwrap_or_default().into(),
                    format: book.book_format_key.unwrap_or_default().into(),
                    series: book.book_series.unwrap_or_default().into(),
                    publication_date: book
                        .book_publication_date
                        .map(|ts| {
                            use chrono::{DateTime, Utc};
                            DateTime::<Utc>::from_timestamp(ts, 0)
                                .map(|dt| dt.format("%Y-%m-%d").to_string())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default()
                        .into(),
                    isbn: book.book_isbn.unwrap_or_default().into(),
                    file_link: book.book_file_link.unwrap_or_default().into(),
                    contents: to_model(contents),
                }
            })
            .collect();

        Ok(slint_books)
    }

    fn get_contents_with_books(
        &self,
        search: Option<&str>,
    ) -> Result<Vec<ContentWithBooks>, Box<dyn std::error::Error>> {
        let config = self
            .current_library
            .as_ref()
            .ok_or("No library loaded")?;

        let mut reporter = SilentReporter;
        let contents = self
            .runtime
            .block_on(async {
                let pool = config
                    .create_pool(&mut reporter)
                    .await
                    .map_err(|e| format!("Failed to create pool: {}", e))?;
                get_contents_with_nested_data(&pool, search)
                    .await
                    .map_err(|e| format!("Query failed: {}", e))
            })
            .map_err(|e: String| -> Box<dyn std::error::Error> { e.into() })?;

        // Convert ritmo_db::ContentWithDetails to Slint ContentWithBooks
        let slint_contents: Vec<ContentWithBooks> = contents
            .into_iter()
            .map(|content| {
                // Convert people with roles
                let people: Vec<PersonWithRole> = content
                    .people
                    .into_iter()
                    .map(|person| PersonWithRole {
                        person_id: person.person_id as i32,
                        person_name: person.person_name.into(),
                        role_name: person.role_key.into(), // Phase 2: Could translate here
                    })
                    .collect();

                // Convert books
                let books: Vec<BookInfo> = content
                    .books
                    .into_iter()
                    .map(|book| BookInfo {
                        id: book.book_id as i32,
                        name: book.book_name.into(),
                        publisher: book.book_publisher.unwrap_or_default().into(),
                        format: book.book_format_key.unwrap_or_default().into(),
                        publication_date: book
                            .book_publication_date
                            .map(|ts| {
                                use chrono::{DateTime, Utc};
                                DateTime::<Utc>::from_timestamp(ts, 0)
                                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                                    .unwrap_or_default()
                            })
                            .unwrap_or_default()
                            .into(),
                    })
                    .collect();

                ContentWithBooks {
                    id: content.content_id as i32,
                    name: content.content_name.into(),
                    original_title: content.content_original_title.unwrap_or_default().into(),
                    type_name: content.content_type_key.unwrap_or_default().into(),
                    publication_date: content
                        .content_publication_date
                        .map(|ts| {
                            use chrono::{DateTime, Utc};
                            DateTime::<Utc>::from_timestamp(ts, 0)
                                .map(|dt| dt.format("%Y-%m-%d").to_string())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default()
                        .into(),
                    people: to_model(people),
                    books: to_model(books),
                }
            })
            .collect();

        Ok(slint_contents)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ui = MainWindow::new()?;

    // Initialize application state with ritmo_config
    let app_state = match AppState::new() {
        Ok(state) => Arc::new(Mutex::new(state)),
        Err(e) => {
            eprintln!("Errore nell'inizializzazione: {}", e);
            return Err(e);
        }
    };

    // Determine library path
    let library_path = {
        let state = app_state.blocking_lock();
        state
            .app_settings
            .last_library_path
            .clone()
            .or_else(|| dirs::home_dir().map(|p| p.join("RitmoLibrary")))
            .unwrap_or_else(|| PathBuf::from("./ritmo_library"))
    };

    // Load library
    {
        let mut state = app_state.blocking_lock();
        match state.load_library(&library_path.to_string_lossy()) {
            Ok(_) => {
                // Try to initialize if it doesn't exist
                if let Err(_e) = state.initialize_current_library() {
                    ui.set_status_message(StatusMessage {
                        text: format!("Libreria inizializzata: {}", library_path.display()).into(),
                        is_error: false,
                    });
                } else {
                    ui.set_status_message(StatusMessage {
                        text: format!("Libreria caricata: {}", library_path.display()).into(),
                        is_error: false,
                    });
                }
            }
            Err(e) => {
                ui.set_status_message(StatusMessage {
                    text: format!("Errore caricamento libreria: {}", e).into(),
                    is_error: true,
                });
            }
        }
    }

    // Callback: Refresh books
    {
        let ui_weak = ui.as_weak();
        let app_state = app_state.clone();

        ui.on_refresh_books(move || {
            let ui = ui_weak.unwrap();
            let state = app_state.blocking_lock();

            match state.get_books_with_contents(None) {
                Ok(books) => {
                    let books_model = Rc::new(VecModel::from(books));
                    ui.set_books(ModelRc::from(books_model));
                    ui.set_status_message(StatusMessage {
                        text: format!("Caricati {} libri", ui.get_books().row_count()).into(),
                        is_error: false,
                    });
                }
                Err(e) => {
                    ui.set_status_message(StatusMessage {
                        text: format!("Errore caricamento libri: {}", e).into(),
                        is_error: true,
                    });
                }
            }
        });
    }

    // Callback: Refresh contents
    {
        let ui_weak = ui.as_weak();
        let app_state = app_state.clone();

        ui.on_refresh_contents(move || {
            let ui = ui_weak.unwrap();
            let state = app_state.blocking_lock();

            match state.get_contents_with_books(None) {
                Ok(contents) => {
                    let contents_model = Rc::new(VecModel::from(contents));
                    ui.set_contents(ModelRc::from(contents_model));
                    ui.set_status_message(StatusMessage {
                        text: format!("Caricati {} contenuti", ui.get_contents().row_count()).into(),
                        is_error: false,
                    });
                }
                Err(e) => {
                    ui.set_status_message(StatusMessage {
                        text: format!("Errore caricamento contenuti: {}", e).into(),
                        is_error: true,
                    });
                }
            }
        });
    }

    // Callback: Search with real database query
    {
        let ui_weak = ui.as_weak();
        let app_state = app_state.clone();

        ui.on_search(move |search_text: SharedString| {
            let ui = ui_weak.unwrap();
            let state = app_state.blocking_lock();

            if search_text.is_empty() {
                // Empty search = show all
                if ui.get_view_mode() == 0 {
                    ui.invoke_refresh_books();
                } else {
                    ui.invoke_refresh_contents();
                }
                return;
            }

            let search_query = search_text.to_string();

            if ui.get_view_mode() == 0 {
                // Books view - search in database
                match state.get_books_with_contents(Some(&search_query)) {
                    Ok(books) => {
                        let count = books.len();
                        let books_model = Rc::new(VecModel::from(books));
                        ui.set_books(ModelRc::from(books_model));
                        ui.set_status_message(StatusMessage {
                            text: format!("Trovati {} libri", count).into(),
                            is_error: false,
                        });
                    }
                    Err(e) => {
                        ui.set_status_message(StatusMessage {
                            text: format!("Errore ricerca: {}", e).into(),
                            is_error: true,
                        });
                    }
                }
            } else {
                // Contents view - search in database
                match state.get_contents_with_books(Some(&search_query)) {
                    Ok(contents) => {
                        let count = contents.len();
                        let contents_model = Rc::new(VecModel::from(contents));
                        ui.set_contents(ModelRc::from(contents_model));
                        ui.set_status_message(StatusMessage {
                            text: format!("Trovati {} contenuti", count).into(),
                            is_error: false,
                        });
                    }
                    Err(e) => {
                        ui.set_status_message(StatusMessage {
                            text: format!("Errore ricerca: {}", e).into(),
                            is_error: true,
                        });
                    }
                }
            }
        });
    }

    // Callback: Add new book (placeholder for Phase 2)
    {
        let ui_weak = ui.as_weak();

        ui.on_add_new_book(move || {
            let ui = ui_weak.unwrap();
            ui.set_status_message(StatusMessage {
                text: "Funzione 'Aggiungi Libro' in sviluppo (Fase 2)...".into(),
                is_error: false,
            });
        });
    }

    // Callback: Show book detail (placeholder for Phase 2)
    {
        let ui_weak = ui.as_weak();

        ui.on_show_book_detail(move |book_id: i32| {
            let ui = ui_weak.unwrap();
            ui.set_status_message(StatusMessage {
                text: format!("Dettaglio libro ID: {} (Fase 2)", book_id).into(),
                is_error: false,
            });
        });
    }

    // Callback: Show content detail (placeholder for Phase 2)
    {
        let ui_weak = ui.as_weak();

        ui.on_show_content_detail(move |content_id: i32| {
            let ui = ui_weak.unwrap();
            ui.set_status_message(StatusMessage {
                text: format!("Dettaglio contenuto ID: {} (Fase 2)", content_id).into(),
                is_error: false,
            });
        });
    }

    // Load initial data
    ui.invoke_refresh_books();
    ui.invoke_refresh_contents();

    // Run UI
    ui.run()?;

    Ok(())
}
