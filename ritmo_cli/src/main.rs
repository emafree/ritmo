use clap::{Parser, Subcommand};
use ritmo_config::{detect_portable_library, settings_file, AppSettings};
use ritmo_db_core::{BookFilters, BookSortField, ContentFilters, ContentSortField, LibraryConfig};
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

    /// Lista libri con filtri
    ListBooks {
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
    },

    /// Lista contenuti con filtri
    ListContents {
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
        Commands::ListBooks {
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
        } => {
            cmd_list_books(
                &cli.library,
                &app_settings,
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
            )
            .await?;
        }
        Commands::ListContents {
            author,
            content_type,
            year,
            search,
            sort,
            limit,
            offset,
        } => {
            cmd_list_contents(
                &cli.library,
                &app_settings,
                author,
                content_type,
                year,
                search,
                sort,
                limit,
                offset,
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
) -> Result<(), Box<dyn std::error::Error>> {
    // Determina quale libreria usare
    let library_path = get_library_path(cli_library, app_settings)?;

    println!("Ricerca libri nella libreria: {}", library_path.display());

    // TODO: Implementare query database
    println!("\nFiltri attivi:");
    if let Some(a) = &author {
        println!("  Autore: {}", a);
    }
    if let Some(p) = &publisher {
        println!("  Editore: {}", p);
    }
    if let Some(s) = &series {
        println!("  Serie: {}", s);
    }
    if let Some(f) = &format {
        println!("  Formato: {}", f);
    }
    if let Some(y) = year {
        println!("  Anno: {}", y);
    }
    if let Some(i) = &isbn {
        println!("  ISBN: {}", i);
    }
    if let Some(s) = &search {
        println!("  Ricerca: {}", s);
    }
    println!("  Ordinamento: {}", sort);
    if let Some(l) = limit {
        println!("  Limite: {}", l);
    }
    if offset > 0 {
        println!("  Offset: {}", offset);
    }

    println!("\n(Implementazione query in corso...)");

    Ok(())
}

/// Comando: list-contents - Lista contenuti con filtri
async fn cmd_list_contents(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    author: Option<String>,
    content_type: Option<String>,
    year: Option<i32>,
    search: Option<String>,
    sort: String,
    limit: Option<i64>,
    offset: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determina quale libreria usare
    let library_path = get_library_path(cli_library, app_settings)?;

    println!(
        "Ricerca contenuti nella libreria: {}",
        library_path.display()
    );

    // TODO: Implementare query database
    println!("\nFiltri attivi:");
    if let Some(a) = &author {
        println!("  Autore: {}", a);
    }
    if let Some(t) = &content_type {
        println!("  Tipo: {}", t);
    }
    if let Some(y) = year {
        println!("  Anno: {}", y);
    }
    if let Some(s) = &search {
        println!("  Ricerca: {}", s);
    }
    println!("  Ordinamento: {}", sort);
    if let Some(l) = limit {
        println!("  Limite: {}", l);
    }
    if offset > 0 {
        println!("  Offset: {}", offset);
    }

    println!("\n(Implementazione query in corso...)");

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
