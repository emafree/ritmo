mod formatter;

use clap::{Parser, Subcommand};
use formatter::{format_books, format_contents, OutputFormat};
use ritmo_config::{
    detect_portable_library, settings_file, AppSettings, BookFilterPreset, ContentFilterPreset,
    NamedPreset, PresetType,
};
use ritmo_db_core::{
    execute_books_query, execute_contents_query, BookFilters, BookSortField, ContentFilters,
    ContentSortField, LibraryConfig,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ritmo")]
#[command(about = "Ritmo - Library Management System", long_about = None)]
struct Cli {
    /// Usa una libreria specifica invece della default
    #[arg(short, long, global = true)]
    library: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inizializza una nuova libreria o usa quella esistente
    Init {
        /// Percorso della libreria (default: ~/RitmoLibrary)
        path: Option<PathBuf>,
    },

    /// Mostra informazioni sulla libreria corrente
    Info,

    /// Lista tutte le librerie recenti
    ListLibraries,

    /// Imposta la libreria corrente
    SetLibrary {
        /// Percorso della libreria da impostare come corrente
        path: PathBuf,
    },

    /// Salva un preset di filtri
    SavePreset {
        /// Tipo di preset: books o contents
        preset_type: String,

        /// Nome del preset
        #[arg(long)]
        name: String,

        /// Descrizione opzionale
        #[arg(long)]
        description: Option<String>,

        // Filtri per books
        #[arg(long)]
        author: Option<String>,

        #[arg(long)]
        publisher: Option<String>,

        #[arg(long)]
        series: Option<String>,

        #[arg(long)]
        format: Option<String>,

        #[arg(long)]
        year: Option<i32>,

        #[arg(long)]
        isbn: Option<String>,

        #[arg(long)]
        search: Option<String>,

        // Filtro per contents
        #[arg(long)]
        content_type: Option<String>,

        #[arg(long, default_value = "title")]
        sort: String,

        #[arg(long)]
        limit: Option<i64>,

        #[arg(long, default_value = "0")]
        offset: i64,
    },

    /// Lista tutti i preset salvati
    ListPresets {
        /// Tipo opzionale: books o contents (mostra entrambi se omesso)
        preset_type: Option<String>,
    },

    /// Elimina un preset
    DeletePreset {
        /// Tipo di preset: books o contents
        preset_type: String,

        /// Nome del preset da eliminare
        name: String,
    },

    /// Lista libri con filtri
    ListBooks {
        /// Usa un preset salvato
        #[arg(long, short = 'p')]
        preset: Option<String>,

        /// Filtra per autore
        #[arg(long)]
        author: Option<String>,

        /// Filtra per editore
        #[arg(long)]
        publisher: Option<String>,

        /// Filtra per serie
        #[arg(long)]
        series: Option<String>,

        /// Filtra per formato (epub, pdf, mobi, etc.)
        #[arg(long)]
        format: Option<String>,

        /// Filtra per anno di pubblicazione
        #[arg(long)]
        year: Option<i32>,

        /// Filtra per ISBN
        #[arg(long)]
        isbn: Option<String>,

        /// Ricerca full-text (titolo, autori, note)
        #[arg(long, short)]
        search: Option<String>,

        /// Ordina per campo (title, author, year, date_added)
        #[arg(long, default_value = "title")]
        sort: String,

        /// Limita numero risultati
        #[arg(long)]
        limit: Option<i64>,

        /// Offset risultati (per paginazione)
        #[arg(long, default_value = "0")]
        offset: i64,

        /// Formato output (table, json, simple)
        #[arg(long, short = 'o', default_value = "table")]
        output: String,
    },

    /// Lista contenuti con filtri
    ListContents {
        /// Usa un preset salvato
        #[arg(long, short = 'p')]
        preset: Option<String>,

        /// Filtra per autore del contenuto
        #[arg(long)]
        author: Option<String>,

        /// Filtra per tipo (Romanzo, Racconto, Saggio, etc.)
        #[arg(long)]
        content_type: Option<String>,

        /// Filtra per anno di pubblicazione
        #[arg(long)]
        year: Option<i32>,

        /// Ricerca full-text (titolo, autori, note)
        #[arg(long, short)]
        search: Option<String>,

        /// Ordina per campo (title, author, year, type)
        #[arg(long, default_value = "title")]
        sort: String,

        /// Limita numero risultati
        #[arg(long)]
        limit: Option<i64>,

        /// Offset risultati (per paginazione)
        #[arg(long, default_value = "0")]
        offset: i64,

        /// Formato output (table, json, simple)
        #[arg(long, short = 'o', default_value = "table")]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Carica o crea AppSettings
    let settings_path = settings_file()?;
    let mut app_settings = AppSettings::load_or_create(&settings_path)?;

    match cli.command {
        Commands::Init { path } => {
            cmd_init(path, &mut app_settings, &settings_path).await?;
        }
        Commands::Info => {
            cmd_info(&cli.library, &app_settings).await?;
        }
        Commands::ListLibraries => {
            cmd_list_libraries(&app_settings)?;
        }
        Commands::SetLibrary { path } => {
            cmd_set_library(path, &mut app_settings, &settings_path)?;
        }
        Commands::SavePreset {
            preset_type,
            name,
            description,
            author,
            publisher,
            series,
            format,
            year,
            isbn,
            search,
            content_type,
            sort,
            limit,
            offset,
        } => {
            cmd_save_preset(
                &mut app_settings,
                &settings_path,
                preset_type,
                name,
                description,
                author,
                publisher,
                series,
                format,
                year,
                isbn,
                search,
                content_type,
                sort,
                limit,
                offset,
            )?;
        }
        Commands::ListPresets { preset_type } => {
            cmd_list_presets(&app_settings, preset_type)?;
        }
        Commands::DeletePreset { preset_type, name } => {
            cmd_delete_preset(&mut app_settings, &settings_path, preset_type, name)?;
        }
        Commands::ListBooks {
            preset,
            author,
            publisher,
            series,
            format,
            year,
            isbn,
            search,
            sort,
            limit,
            offset,
            output,
        } => {
            cmd_list_books(
                &cli.library,
                &app_settings,
                preset,
                author,
                publisher,
                series,
                format,
                year,
                isbn,
                search,
                sort,
                limit,
                offset,
                output,
            )
            .await?;
        }
        Commands::ListContents {
            preset,
            author,
            content_type,
            year,
            search,
            sort,
            limit,
            offset,
            output,
        } => {
            cmd_list_contents(
                &cli.library,
                &app_settings,
                preset,
                author,
                content_type,
                year,
                search,
                sort,
                limit,
                offset,
                output,
            )
            .await?;
        }
    }

    Ok(())
}

/// Comando: init - Inizializza una nuova libreria
async fn cmd_init(
    path: Option<PathBuf>,
    app_settings: &mut AppSettings,
    settings_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determina il path della libreria
    let library_path = path.unwrap_or_else(|| {
        dirs::home_dir()
            .map(|p| p.join("RitmoLibrary"))
            .unwrap_or_else(|| PathBuf::from("./RitmoLibrary"))
    });

    println!("Inizializzazione libreria: {}", library_path.display());

    // Crea LibraryConfig
    let config = LibraryConfig::new(&library_path);

    // Inizializza directory
    config.initialize()?;
    println!("✓ Directory create");

    // Inizializza database
    config.initialize_database().await?;
    println!("✓ Database inizializzato");

    // Valida
    if config.validate()? {
        println!("✓ Validazione completata");
    } else {
        println!("⚠ Problemi nella validazione");
    }

    // Health check
    let issues = config.health_check();
    if !issues.is_empty() {
        println!("⚠ Problemi rilevati:");
        for issue in issues {
            println!("  - {}", issue);
        }
    }

    // Salva config della libreria
    config.save(config.main_config_file())?;
    println!("✓ Configurazione salvata");

    // Aggiorna AppSettings
    app_settings.update_last_library(&library_path);
    app_settings.save(settings_path)?;
    println!("✓ Libreria impostata come corrente");

    println!("\n✓ Libreria inizializzata con successo!");
    println!("  Path: {}", library_path.display());

    Ok(())
}

/// Comando: info - Mostra informazioni sulla libreria corrente
async fn cmd_info(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determina quale libreria usare
    let library_path = if let Some(path) = cli_library {
        path.clone()
    } else if let Some(portable) = detect_portable_library() {
        println!("ℹ Modalità portabile rilevata");
        portable
    } else if let Some(path) = &app_settings.last_library_path {
        path.clone()
    } else {
        println!("✗ Nessuna libreria configurata");
        println!("  Usa 'ritmo init' per inizializzare una libreria");
        return Ok(());
    };

    println!("Libreria corrente: {}", library_path.display());
    println!();

    // Carica config
    let config = LibraryConfig::new(&library_path);

    if !config.exists() {
        println!("✗ La libreria non esiste");
        println!("  Usa 'ritmo init {}' per crearla", library_path.display());
        return Ok(());
    }

    // Mostra info
    println!("Struttura:");
    println!("  Database:  {}", config.database_path.display());
    println!("  Storage:   {}", config.storage_path.display());
    println!("  Config:    {}", config.config_path.display());
    println!("  Bootstrap: {}", config.bootstrap_path.display());
    println!();

    // Validazione
    if config.validate()? {
        println!("✓ Struttura valida");
    } else {
        println!("✗ Struttura non valida");
    }

    // Health check
    let issues = config.health_check();
    if issues.is_empty() {
        println!("✓ Nessun problema rilevato");
    } else {
        println!("⚠ Problemi rilevati:");
        for issue in issues {
            println!("  - {}", issue);
        }
    }

    Ok(())
}

/// Comando: list-libraries - Lista tutte le librerie recenti
fn cmd_list_libraries(app_settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
    if app_settings.recent_libraries.is_empty() {
        println!("Nessuna libreria recente trovata");
        println!("Usa 'ritmo init' per inizializzare una libreria");
        return Ok(());
    }

    println!("Librerie recenti:");
    for (i, path) in app_settings.recent_libraries.iter().enumerate() {
        let marker = if Some(path) == app_settings.last_library_path.as_ref() {
            "* "
        } else {
            "  "
        };
        println!("{}{}) {}", marker, i + 1, path.display());
    }

    if let Some(portable) = detect_portable_library() {
        println!("\nℹ Modalità portabile: {}", portable.display());
    }

    Ok(())
}

/// Comando: set-library - Imposta la libreria corrente
fn cmd_set_library(
    path: PathBuf,
    app_settings: &mut AppSettings,
    settings_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Verifica che la libreria esista
    let config = LibraryConfig::new(&path);
    if !config.exists() {
        println!("✗ La libreria non esiste: {}", path.display());
        println!("  Usa 'ritmo init {}' per crearla", path.display());
        return Ok(());
    }

    // Aggiorna settings
    app_settings.update_last_library(&path);
    app_settings.save(settings_path)?;

    println!("✓ Libreria impostata come corrente: {}", path.display());

    Ok(())
}

/// Comando: list-books - Lista libri con filtri
async fn cmd_list_books(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    preset: Option<String>,
    author: Option<String>,
    publisher: Option<String>,
    series: Option<String>,
    format: Option<String>,
    year: Option<i32>,
    isbn: Option<String>,
    search: Option<String>,
    sort: String,
    limit: Option<i64>,
    offset: i64,
    output: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determina quale libreria usare
    let library_path = get_library_path(cli_library, app_settings)?;

    // Crea LibraryConfig e pool di connessioni
    let config = LibraryConfig::new(&library_path);

    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let pool = config.create_pool().await?;

    // Costruisci filtri (con supporto preset)
    let filters = if let Some(preset_name) = preset {
        // Carica filtri dal preset
        let preset = app_settings
            .presets
            .get_book_preset(&preset_name)
            .ok_or_else(|| format!("Preset '{}' non trovato", preset_name))?;

        // Merge preset con parametri CLI (i parametri CLI hanno priorità)
        BookFilters {
            author: author.or(preset.filters.author.clone()),
            publisher: publisher.or(preset.filters.publisher.clone()),
            series: series.or(preset.filters.series.clone()),
            format: format.or(preset.filters.format.clone()),
            year: year.or(preset.filters.year),
            isbn: isbn.or(preset.filters.isbn.clone()),
            search: search.or(preset.filters.search.clone()),
            sort: BookSortField::from_str(&sort),
            limit: limit.or(preset.filters.limit),
            offset,
        }
    } else {
        // Usa solo parametri CLI
        BookFilters {
            author,
            publisher,
            series,
            format,
            year,
            isbn,
            search,
            sort: BookSortField::from_str(&sort),
            limit,
            offset,
        }
    };

    // Esegui query
    let books = execute_books_query(&pool, &filters).await?;

    // Formatta output
    let output_format = OutputFormat::from_str(&output);
    let formatted = format_books(&books, &output_format);

    println!("{}", formatted);

    Ok(())
}

/// Comando: list-contents - Lista contenuti con filtri
async fn cmd_list_contents(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    preset: Option<String>,
    author: Option<String>,
    content_type: Option<String>,
    year: Option<i32>,
    search: Option<String>,
    sort: String,
    limit: Option<i64>,
    offset: i64,
    output: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determina quale libreria usare
    let library_path = get_library_path(cli_library, app_settings)?;

    // Crea LibraryConfig e pool di connessioni
    let config = LibraryConfig::new(&library_path);

    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let pool = config.create_pool().await?;

    // Costruisci filtri (con supporto preset)
    let filters = if let Some(preset_name) = preset {
        // Carica filtri dal preset
        let preset = app_settings
            .presets
            .get_content_preset(&preset_name)
            .ok_or_else(|| format!("Preset '{}' non trovato", preset_name))?;

        // Merge preset con parametri CLI (i parametri CLI hanno priorità)
        ContentFilters {
            author: author.or(preset.filters.author.clone()),
            content_type: content_type.or(preset.filters.content_type.clone()),
            year: year.or(preset.filters.year),
            search: search.or(preset.filters.search.clone()),
            sort: ContentSortField::from_str(&sort),
            limit: limit.or(preset.filters.limit),
            offset,
        }
    } else {
        // Usa solo parametri CLI
        ContentFilters {
            author,
            content_type,
            year,
            search,
            sort: ContentSortField::from_str(&sort),
            limit,
            offset,
        }
    };

    // Esegui query
    let contents = execute_contents_query(&pool, &filters).await?;

    // Formatta output
    let output_format = OutputFormat::from_str(&output);
    let formatted = format_contents(&contents, &output_format);

    println!("{}", formatted);

    Ok(())
}

/// Helper: determina il path della libreria da usare
fn get_library_path(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(path) = cli_library {
        Ok(path.clone())
    } else if let Some(portable) = detect_portable_library() {
        Ok(portable)
    } else if let Some(path) = &app_settings.last_library_path {
        Ok(path.clone())
    } else {
        Err("Nessuna libreria configurata. Usa 'ritmo init' per inizializzare una libreria".into())
    }
}

/// Comando: save-preset - Salva un preset di filtri
#[allow(clippy::too_many_arguments)]
fn cmd_save_preset(
    app_settings: &mut AppSettings,
    settings_path: &PathBuf,
    preset_type: String,
    name: String,
    description: Option<String>,
    author: Option<String>,
    publisher: Option<String>,
    series: Option<String>,
    format: Option<String>,
    year: Option<i32>,
    isbn: Option<String>,
    search: Option<String>,
    content_type: Option<String>,
    sort: String,
    limit: Option<i64>,
    offset: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let preset_type_enum = PresetType::from_str(&preset_type).ok_or_else(|| {
        format!(
            "Tipo preset non valido: '{}'. Usa 'books' o 'contents'",
            preset_type
        )
    })?;

    match preset_type_enum {
        PresetType::Books => {
            let filters = BookFilterPreset {
                author,
                publisher,
                series,
                format,
                year,
                isbn,
                search,
                sort,
                limit,
                offset,
            };

            let preset = NamedPreset {
                name: name.clone(),
                description,
                filters,
            };

            app_settings.presets.add_book_preset(preset);
            app_settings.save(settings_path)?;

            println!("✓ Preset '{}' salvato per libri", name);
        }
        PresetType::Contents => {
            let filters = ContentFilterPreset {
                author,
                content_type,
                year,
                search,
                sort,
                limit,
                offset,
            };

            let preset = NamedPreset {
                name: name.clone(),
                description,
                filters,
            };

            app_settings.presets.add_content_preset(preset);
            app_settings.save(settings_path)?;

            println!("✓ Preset '{}' salvato per contenuti", name);
        }
    }

    Ok(())
}

/// Comando: list-presets - Lista tutti i preset salvati
fn cmd_list_presets(
    app_settings: &AppSettings,
    preset_type: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let show_books = preset_type.is_none()
        || preset_type.as_ref().map(|s| s.to_lowercase()) == Some("books".to_string())
        || preset_type.as_ref().map(|s| s.to_lowercase()) == Some("book".to_string());

    let show_contents = preset_type.is_none()
        || preset_type.as_ref().map(|s| s.to_lowercase()) == Some("contents".to_string())
        || preset_type.as_ref().map(|s| s.to_lowercase()) == Some("content".to_string());

    let mut found_any = false;

    if show_books && !app_settings.presets.books.is_empty() {
        println!("Preset per Libri:");
        println!("{}", "-".repeat(50));
        for (name, preset) in &app_settings.presets.books {
            println!("• {}", name);
            if let Some(desc) = &preset.description {
                println!("  Descrizione: {}", desc);
            }

            let mut filters = Vec::new();
            if let Some(a) = &preset.filters.author {
                filters.push(format!("autore={}", a));
            }
            if let Some(p) = &preset.filters.publisher {
                filters.push(format!("editore={}", p));
            }
            if let Some(s) = &preset.filters.series {
                filters.push(format!("serie={}", s));
            }
            if let Some(f) = &preset.filters.format {
                filters.push(format!("formato={}", f));
            }
            if let Some(y) = preset.filters.year {
                filters.push(format!("anno={}", y));
            }
            if let Some(i) = &preset.filters.isbn {
                filters.push(format!("isbn={}", i));
            }
            if let Some(s) = &preset.filters.search {
                filters.push(format!("ricerca={}", s));
            }
            filters.push(format!("ordina={}", preset.filters.sort));
            if let Some(l) = preset.filters.limit {
                filters.push(format!("limite={}", l));
            }

            println!("  Filtri: {}", filters.join(", "));
            println!();
        }
        found_any = true;
    }

    if show_contents && !app_settings.presets.contents.is_empty() {
        if found_any {
            println!();
        }
        println!("Preset per Contenuti:");
        println!("{}", "-".repeat(50));
        for (name, preset) in &app_settings.presets.contents {
            println!("• {}", name);
            if let Some(desc) = &preset.description {
                println!("  Descrizione: {}", desc);
            }

            let mut filters = Vec::new();
            if let Some(a) = &preset.filters.author {
                filters.push(format!("autore={}", a));
            }
            if let Some(t) = &preset.filters.content_type {
                filters.push(format!("tipo={}", t));
            }
            if let Some(y) = preset.filters.year {
                filters.push(format!("anno={}", y));
            }
            if let Some(s) = &preset.filters.search {
                filters.push(format!("ricerca={}", s));
            }
            filters.push(format!("ordina={}", preset.filters.sort));
            if let Some(l) = preset.filters.limit {
                filters.push(format!("limite={}", l));
            }

            println!("  Filtri: {}", filters.join(", "));
            println!();
        }
        found_any = true;
    }

    if !found_any {
        println!("Nessun preset salvato.");
        println!("Usa 'ritmo save-preset' per salvare un nuovo preset.");
    }

    Ok(())
}

/// Comando: delete-preset - Elimina un preset
fn cmd_delete_preset(
    app_settings: &mut AppSettings,
    settings_path: &PathBuf,
    preset_type: String,
    name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let preset_type_enum = PresetType::from_str(&preset_type).ok_or_else(|| {
        format!(
            "Tipo preset non valido: '{}'. Usa 'books' o 'contents'",
            preset_type
        )
    })?;

    let removed = match preset_type_enum {
        PresetType::Books => app_settings.presets.remove_book_preset(&name).is_some(),
        PresetType::Contents => app_settings.presets.remove_content_preset(&name).is_some(),
    };

    if removed {
        app_settings.save(settings_path)?;
        println!("✓ Preset '{}' eliminato", name);
    } else {
        println!("✗ Preset '{}' non trovato", name);
    }

    Ok(())
}
