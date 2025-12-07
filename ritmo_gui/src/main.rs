slint::include_modules!();

use ritmo_db_core::LibraryConfig;
use slint::{Model, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::Mutex;

// Helper function to convert Vec into ModelRc
fn to_model<T: Clone + 'static>(v: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(v)))
}

// Struttura per gestire lo stato dell'applicazione
struct AppState {
    config: LibraryConfig,
    runtime: tokio::runtime::Runtime,
}

impl AppState {
    fn new(library_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = LibraryConfig::new(library_path);
        let runtime = tokio::runtime::Runtime::new()?;

        Ok(Self { config, runtime })
    }

    fn initialize_library(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.initialize()?;
        self.runtime.block_on(async {
            self.config.initialize_database().await
        })?;
        Ok(())
    }

    // Vista LIBRI: libri con i loro contenuti
    fn get_books_with_contents(&self) -> Result<Vec<BookWithContents>, Box<dyn std::error::Error>> {
        // TODO: Query database reale
        // Per ora dati di esempio realistici
        Ok(vec![
            BookWithContents {
                id: 1,
                name: "I romanzi di Italo Calvino - Volume 1".into(),
                original_title: "".into(),
                publisher: "Mondadori".into(),
                format: "EPUB".into(),
                series: "Opere di Calvino".into(),
                publication_date: "2010".into(),
                isbn: "978-8804597659".into(),
                file_link: "/books/calvino_vol1.epub".into(),
                contents: to_model(vec![
                    ContentInfo {
                        id: 1,
                        name: "Il visconte dimezzato".into(),
                        original_title: "".into(),
                        type_name: "Romanzo".into(),
                        publication_date: "1952".into(),
                        people: to_model(vec![
                            PersonWithRole {
                                person_id: 1,
                                person_name: "Italo Calvino".into(),
                                role_name: "Autore".into(),
                            }
]),
                    },
                    ContentInfo {
                        id: 2,
                        name: "Il barone rampante".into(),
                        original_title: "".into(),
                        type_name: "Romanzo".into(),
                        publication_date: "1957".into(),
                        people: to_model(vec![
                            PersonWithRole {
                                person_id: 1,
                                person_name: "Italo Calvino".into(),
                                role_name: "Autore".into(),
                            }
]),
                    },
                    ContentInfo {
                        id: 3,
                        name: "Il cavaliere inesistente".into(),
                        original_title: "".into(),
                        type_name: "Romanzo".into(),
                        publication_date: "1959".into(),
                        people: to_model(vec![
                            PersonWithRole {
                                person_id: 1,
                                person_name: "Italo Calvino".into(),
                                role_name: "Autore".into(),
                            }
]),
                    },
]),
            },
            BookWithContents {
                id: 2,
                name: "Antologia della letteratura italiana moderna".into(),
                original_title: "".into(),
                publisher: "Einaudi".into(),
                format: "PDF".into(),
                series: "".into(),
                publication_date: "2015".into(),
                isbn: "978-8806223458".into(),
                file_link: "/books/antologia_moderna.pdf".into(),
                contents: to_model(vec![
                    ContentInfo {
                        id: 1,
                        name: "Il visconte dimezzato".into(),
                        original_title: "".into(),
                        type_name: "Romanzo".into(),
                        publication_date: "1952".into(),
                        people: to_model(vec![
                            PersonWithRole {
                                person_id: 1,
                                person_name: "Italo Calvino".into(),
                                role_name: "Autore".into(),
                            }
]),
                    },
                    ContentInfo {
                        id: 4,
                        name: "Il deserto dei Tartari".into(),
                        original_title: "".into(),
                        type_name: "Romanzo".into(),
                        publication_date: "1940".into(),
                        people: to_model(vec![
                            PersonWithRole {
                                person_id: 2,
                                person_name: "Dino Buzzati".into(),
                                role_name: "Autore".into(),
                            }
]),
                    },
                    ContentInfo {
                        id: 5,
                        name: "La coscienza di Zeno".into(),
                        original_title: "".into(),
                        type_name: "Romanzo".into(),
                        publication_date: "1923".into(),
                        people: to_model(vec![
                            PersonWithRole {
                                person_id: 3,
                                person_name: "Italo Svevo".into(),
                                role_name: "Autore".into(),
                            }
]),
                    },
]),
            },
            BookWithContents {
                id: 3,
                name: "Il deserto dei Tartari - Edizione integrale".into(),
                original_title: "".into(),
                publisher: "Bompiani".into(),
                format: "EPUB".into(),
                series: "".into(),
                publication_date: "2018".into(),
                isbn: "978-8845297229".into(),
                file_link: "/books/deserto_tartari.epub".into(),
                contents: to_model(vec![
                    ContentInfo {
                        id: 4,
                        name: "Il deserto dei Tartari".into(),
                        original_title: "".into(),
                        type_name: "Romanzo".into(),
                        publication_date: "1940".into(),
                        people: to_model(vec![
                            PersonWithRole {
                                person_id: 2,
                                person_name: "Dino Buzzati".into(),
                                role_name: "Autore".into(),
                            }
]),
                    },
]),
            },
        ])
    }

    // Vista CONTENUTI: contenuti con i libri che li contengono
    fn get_contents_with_books(&self) -> Result<Vec<ContentWithBooks>, Box<dyn std::error::Error>> {
        // TODO: Query database reale
        // Per ora dati di esempio realistici
        Ok(vec![
            ContentWithBooks {
                id: 1,
                name: "Il visconte dimezzato".into(),
                original_title: "".into(),
                type_name: "Romanzo".into(),
                publication_date: "1952".into(),
                people: to_model(vec![
                    PersonWithRole {
                        person_id: 1,
                        person_name: "Italo Calvino".into(),
                        role_name: "Autore".into(),
                    }
]),
                books: to_model(vec![
                    BookInfo {
                        id: 1,
                        name: "I romanzi di Italo Calvino - Volume 1".into(),
                        publisher: "Mondadori".into(),
                        format: "EPUB".into(),
                        publication_date: "2010".into(),
                    },
                    BookInfo {
                        id: 2,
                        name: "Antologia della letteratura italiana moderna".into(),
                        publisher: "Einaudi".into(),
                        format: "PDF".into(),
                        publication_date: "2015".into(),
                    },
]),
            },
            ContentWithBooks {
                id: 2,
                name: "Il barone rampante".into(),
                original_title: "".into(),
                type_name: "Romanzo".into(),
                publication_date: "1957".into(),
                people: to_model(vec![
                    PersonWithRole {
                        person_id: 1,
                        person_name: "Italo Calvino".into(),
                        role_name: "Autore".into(),
                    }
]),
                books: to_model(vec![
                    BookInfo {
                        id: 1,
                        name: "I romanzi di Italo Calvino - Volume 1".into(),
                        publisher: "Mondadori".into(),
                        format: "EPUB".into(),
                        publication_date: "2010".into(),
                    },
]),
            },
            ContentWithBooks {
                id: 3,
                name: "Il cavaliere inesistente".into(),
                original_title: "".into(),
                type_name: "Romanzo".into(),
                publication_date: "1959".into(),
                people: to_model(vec![
                    PersonWithRole {
                        person_id: 1,
                        person_name: "Italo Calvino".into(),
                        role_name: "Autore".into(),
                    }
]),
                books: to_model(vec![
                    BookInfo {
                        id: 1,
                        name: "I romanzi di Italo Calvino - Volume 1".into(),
                        publisher: "Mondadori".into(),
                        format: "EPUB".into(),
                        publication_date: "2010".into(),
                    },
]),
            },
            ContentWithBooks {
                id: 4,
                name: "Il deserto dei Tartari".into(),
                original_title: "".into(),
                type_name: "Romanzo".into(),
                publication_date: "1940".into(),
                people: to_model(vec![
                    PersonWithRole {
                        person_id: 2,
                        person_name: "Dino Buzzati".into(),
                        role_name: "Autore".into(),
                    }
]),
                books: to_model(vec![
                    BookInfo {
                        id: 2,
                        name: "Antologia della letteratura italiana moderna".into(),
                        publisher: "Einaudi".into(),
                        format: "PDF".into(),
                        publication_date: "2015".into(),
                    },
                    BookInfo {
                        id: 3,
                        name: "Il deserto dei Tartari - Edizione integrale".into(),
                        publisher: "Bompiani".into(),
                        format: "EPUB".into(),
                        publication_date: "2018".into(),
                    },
]),
            },
            ContentWithBooks {
                id: 5,
                name: "La coscienza di Zeno".into(),
                original_title: "".into(),
                type_name: "Romanzo".into(),
                publication_date: "1923".into(),
                people: to_model(vec![
                    PersonWithRole {
                        person_id: 3,
                        person_name: "Italo Svevo".into(),
                        role_name: "Autore".into(),
                    }
]),
                books: to_model(vec![
                    BookInfo {
                        id: 2,
                        name: "Antologia della letteratura italiana moderna".into(),
                        publisher: "Einaudi".into(),
                        format: "PDF".into(),
                        publication_date: "2015".into(),
                    },
]),
            },
        ])
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ui = MainWindow::new()?;

    // Percorso libreria
    let library_path = dirs::home_dir()
        .map(|p| p.join("RitmoLibrary"))
        .unwrap_or_else(|| std::path::PathBuf::from("./ritmo_library"));
    let library_path_str = library_path.to_string_lossy().to_string();

    // Inizializza stato applicazione
    let app_state = match AppState::new(&library_path_str) {
        Ok(state) => Arc::new(Mutex::new(state)),
        Err(e) => {
            eprintln!("Errore nell'inizializzazione: {}", e);
            return Err(e);
        }
    };

    // Inizializza libreria
    {
        let state = app_state.blocking_lock();
        match state.initialize_library() {
            Ok(_) => {
                ui.set_status_message(StatusMessage {
                    text: format!("Libreria inizializzata: {}", library_path_str).into(),
                    is_error: false,
                });
            }
            Err(e) => {
                ui.set_status_message(StatusMessage {
                    text: format!("Errore: {}", e).into(),
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

            match state.get_books_with_contents() {
                Ok(books) => {
                    let books_model = Rc::new(VecModel::from(books));
                    ui.set_books(ModelRc::from(books_model));
                    ui.set_status_message(StatusMessage {
                        text: "Libri caricati".into(),
                        is_error: false,
                    });
                }
                Err(e) => {
                    ui.set_status_message(StatusMessage {
                        text: format!("Errore: {}", e).into(),
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

            match state.get_contents_with_books() {
                Ok(contents) => {
                    let contents_model = Rc::new(VecModel::from(contents));
                    ui.set_contents(ModelRc::from(contents_model));
                    ui.set_status_message(StatusMessage {
                        text: "Contenuti caricati".into(),
                        is_error: false,
                    });
                }
                Err(e) => {
                    ui.set_status_message(StatusMessage {
                        text: format!("Errore: {}", e).into(),
                        is_error: true,
                    });
                }
            }
        });
    }

    // Callback: Search
    {
        let ui_weak = ui.as_weak();
        let app_state = app_state.clone();

        ui.on_search(move |search_text: SharedString| {
            let ui = ui_weak.unwrap();
            let state = app_state.blocking_lock();

            // TODO: Implementare ricerca vera nel database
            // Per ora filtriamo i dati di esempio
            if ui.get_view_mode() == 0 {
                // Vista libri
                match state.get_books_with_contents() {
                    Ok(mut books) => {
                        if !search_text.is_empty() {
                            let search_lower = search_text.to_lowercase();
                            books.retain(|book| {
                                book.name.to_lowercase().contains(&search_lower)
                                    || book.publisher.to_lowercase().contains(&search_lower)
                                    || book.contents.iter().any(|c| {
                                        c.name.to_lowercase().contains(&search_lower)
                                            || c.people.iter().any(|p| {
                                                p.person_name.to_lowercase().contains(&search_lower)
                                            })
                                    })
                            });
                        }
                        let books_model = Rc::new(VecModel::from(books));
                        ui.set_books(ModelRc::from(books_model));
                    }
                    Err(_) => {}
                }
            } else {
                // Vista contenuti
                match state.get_contents_with_books() {
                    Ok(mut contents) => {
                        if !search_text.is_empty() {
                            let search_lower = search_text.to_lowercase();
                            contents.retain(|content| {
                                content.name.to_lowercase().contains(&search_lower)
                                    || content.people.iter().any(|p| {
                                        p.person_name.to_lowercase().contains(&search_lower)
                                    })
                                    || content.books.iter().any(|b| {
                                        b.name.to_lowercase().contains(&search_lower)
                                    })
                            });
                        }
                        let contents_model = Rc::new(VecModel::from(contents));
                        ui.set_contents(ModelRc::from(contents_model));
                    }
                    Err(_) => {}
                }
            }
        });
    }

    // Callback: Add new book
    {
        let ui_weak = ui.as_weak();

        ui.on_add_new_book(move || {
            let ui = ui_weak.unwrap();
            ui.set_status_message(StatusMessage {
                text: "Funzione 'Aggiungi' in sviluppo...".into(),
                is_error: false,
            });
        });
    }

    // Callback: Show book detail
    {
        let ui_weak = ui.as_weak();

        ui.on_show_book_detail(move |book_id: i32| {
            let ui = ui_weak.unwrap();
            ui.set_status_message(StatusMessage {
                text: format!("Dettaglio libro ID: {}", book_id).into(),
                is_error: false,
            });
        });
    }

    // Callback: Show content detail
    {
        let ui_weak = ui.as_weak();

        ui.on_show_content_detail(move |content_id: i32| {
            let ui = ui_weak.unwrap();
            ui.set_status_message(StatusMessage {
                text: format!("Dettaglio contenuto ID: {}", content_id).into(),
                is_error: false,
            });
        });
    }

    // Carica dati iniziali
    ui.invoke_refresh_books();
    ui.invoke_refresh_contents();

    // Avvia UI
    ui.run()?;

    Ok(())
}
