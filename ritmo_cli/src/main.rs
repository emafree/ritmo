mod formatter;

use clap::{Parser, Subcommand};
use formatter::{format_books, format_contents, OutputFormat};
use ritmo_config::{
    detect_portable_library, settings_file, AppSettings, BookFilterPreset, ContentFilterPreset,
    NamedPreset, PresetType,
};
use ritmo_core::service::{import_book, BookImportMetadata};
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

        /// Salva nella libreria corrente invece che globalmente
        #[arg(long)]
        in_library: bool,

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

        #[arg(long)]
        acquired_after: Option<String>,

        #[arg(long)]
        acquired_before: Option<String>,

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

    /// Imposta il preset di default per una libreria
    SetDefaultFilter {
        /// Tipo: books o contents
        preset_type: String,

        /// Nome del preset da impostare come default (usa 'none' per rimuovere)
        preset_name: String,
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

        /// Filtra libri acquisiti dopo questa data (YYYY-MM-DD)
        #[arg(long)]
        acquired_after: Option<String>,

        /// Filtra libri acquisiti prima di questa data (YYYY-MM-DD)
        #[arg(long)]
        acquired_before: Option<String>,

        /// Filtra libri acquisiti negli ultimi N giorni
        #[arg(long, conflicts_with = "acquired_after")]
        last_days: Option<i64>,

        /// Filtra libri acquisiti negli ultimi N mesi
        #[arg(long, conflicts_with = "acquired_after")]
        last_months: Option<i64>,

        /// Limita ai primi N libri acquisiti più recentemente (equivale a sort=date_added + limit)
        #[arg(long)]
        recent_count: Option<i64>,

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

    /// Importa un libro nella libreria
    Add {
        /// Percorso del file da importare
        file: PathBuf,

        /// Titolo del libro (richiesto)
        #[arg(long, short = 't')]
        title: String,

        /// Autore del libro
        #[arg(long, short = 'a')]
        author: Option<String>,

        /// Editore
        #[arg(long, short = 'p')]
        publisher: Option<String>,

        /// Anno di pubblicazione
        #[arg(long, short = 'y')]
        year: Option<i32>,

        /// ISBN
        #[arg(long)]
        isbn: Option<String>,

        /// Formato (epub, pdf, mobi, etc.) - rilevato automaticamente se omesso
        #[arg(long, short = 'f')]
        format: Option<String>,

        /// Serie
        #[arg(long, short = 's')]
        series: Option<String>,

        /// Indice nella serie
        #[arg(long)]
        series_index: Option<i64>,

        /// Note
        #[arg(long, short = 'n')]
        notes: Option<String>,
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
            in_library,
            author,
            publisher,
            series,
            format,
            year,
            isbn,
            search,
            acquired_after,
            acquired_before,
            content_type,
            sort,
            limit,
            offset,
        } => {
            cmd_save_preset(
                &cli.library,
                &mut app_settings,
                &settings_path,
                preset_type,
                name,
                in_library,
                description,
                author,
                publisher,
                series,
                format,
                year,
                isbn,
                search,
                acquired_after,
                acquired_before,
                content_type,
                sort,
                limit,
                offset,
            )?;
        }
        Commands::ListPresets { preset_type } => {
            cmd_list_presets(&cli.library, &app_settings, preset_type)?;
        }
        Commands::DeletePreset { preset_type, name } => {
            cmd_delete_preset(&mut app_settings, &settings_path, preset_type, name)?;
        }
        Commands::SetDefaultFilter {
            preset_type,
            preset_name,
        } => {
            cmd_set_default_filter(&cli.library, &app_settings, preset_type, preset_name)?;
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
            acquired_after,
            acquired_before,
            last_days,
            last_months,
            recent_count,
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
                acquired_after,
                acquired_before,
                last_days,
                last_months,
                recent_count,
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
        Commands::Add {
            file,
            title,
            author,
            publisher,
            year,
            isbn,
            format,
            series,
            series_index,
            notes,
        } => {
            cmd_add(
                &cli.library,
                &app_settings,
                file,
                title,
                author,
                publisher,
                year,
                isbn,
                format,
                series,
                series_index,
                notes,
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

    // Crea preset di esempio per la libreria (load_or_create crea automaticamente gli esempi)
    let _library_presets = config.load_library_presets()?;
    println!("✓ Preset di esempio creati (epub_only, pdf_only, novels)");

    // Aggiorna AppSettings
    app_settings.update_last_library(&library_path);
    app_settings.save(settings_path)?;
    println!("✓ Libreria impostata come corrente");

    println!("\n✓ Libreria inizializzata con successo!");
    println!("  Path: {}", library_path.display());
    println!("\nUsa 'ritmo list-presets' per vedere i preset disponibili.");

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
    acquired_after: Option<String>,
    acquired_before: Option<String>,
    last_days: Option<i64>,
    last_months: Option<i64>,
    recent_count: Option<i64>,
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

    // Gestisci filtri di data relativi
    let acquired_after_ts = if let Some(days) = last_days {
        // Usa filtro relativo: ultimi N giorni
        Some(timestamp_days_ago(days))
    } else if let Some(months) = last_months {
        // Usa filtro relativo: ultimi N mesi
        Some(timestamp_months_ago(months))
    } else if let Some(date_str) = &acquired_after {
        // Usa filtro assoluto
        Some(parse_date_to_timestamp(date_str)?)
    } else {
        None
    };

    let acquired_before_ts = if let Some(date_str) = &acquired_before {
        Some(parse_date_to_timestamp(date_str)?)
    } else {
        None
    };

    // Gestisci recent_count: override sort e limit
    let (final_sort, final_limit) = if let Some(count) = recent_count {
        ("date_added".to_string(), Some(count))
    } else {
        (sort, limit)
    };

    // Carica preset della libreria per resolution
    let library_presets = config.load_library_presets().ok();

    // Costruisci filtri (con supporto preset con resolution order)
    // Costruisci filtri usando builder pattern
    let mut filters = BookFilters::default();

    // Se c'è un preset, carica i suoi valori come base
    if let Some(preset_name) = preset {
        // Risolvi preset: library > global
        let preset = if let Some(ref lib_presets) = library_presets {
            lib_presets
                .get_book_preset(&preset_name)
                .or_else(|| app_settings.presets.get_book_preset(&preset_name))
        } else {
            app_settings.presets.get_book_preset(&preset_name)
        }
        .ok_or_else(|| format!("Preset '{}' non trovato", preset_name))?;

        // Applica valori dal preset (i parametri CLI sovrascriveranno questi)
        filters = filters
            .set_author_opt(preset.filters.author.clone())
            .set_publisher_opt(preset.filters.publisher.clone())
            .set_series_opt(preset.filters.series.clone())
            .set_format_opt(preset.filters.format.clone());

        filters.year = preset.filters.year;
        filters.isbn = preset.filters.isbn.clone();
        filters.search = preset.filters.search.clone();
        filters.acquired_after = preset.filters.acquired_after;
        filters.acquired_before = preset.filters.acquired_before;
        filters.limit = preset.filters.limit;
    }

    // Applica parametri CLI (hanno priorità su preset)
    filters = filters
        .set_author_opt(author)
        .set_publisher_opt(publisher)
        .set_series_opt(series)
        .set_format_opt(format);

    if let Some(y) = year {
        filters.year = Some(y);
    }
    if let Some(i) = isbn {
        filters.isbn = Some(i);
    }
    if let Some(s) = search {
        filters.search = Some(s);
    }
    if let Some(aa) = acquired_after_ts {
        filters.acquired_after = Some(aa);
    }
    if let Some(ab) = acquired_before_ts {
        filters.acquired_before = Some(ab);
    }

    filters.sort = BookSortField::from_str(&final_sort);
    filters.limit = final_limit;
    filters.offset = offset;

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

    // Carica preset della libreria per resolution
    let library_presets = config.load_library_presets().ok();

    // Costruisci filtri usando builder pattern
    let mut filters = ContentFilters::default();

    // Se c'è un preset, carica i suoi valori come base
    if let Some(preset_name) = preset {
        // Risolvi preset: library > global
        let preset = if let Some(ref lib_presets) = library_presets {
            lib_presets
                .get_content_preset(&preset_name)
                .or_else(|| app_settings.presets.get_content_preset(&preset_name))
        } else {
            app_settings.presets.get_content_preset(&preset_name)
        }
        .ok_or_else(|| format!("Preset '{}' non trovato", preset_name))?;

        // Applica valori dal preset
        filters = filters
            .set_author_opt(preset.filters.author.clone())
            .set_content_type_opt(preset.filters.content_type.clone());

        filters.year = preset.filters.year;
        filters.search = preset.filters.search.clone();
        filters.limit = preset.filters.limit;
    }

    // Applica parametri CLI (hanno priorità su preset)
    filters = filters
        .set_author_opt(author)
        .set_content_type_opt(content_type);

    if let Some(y) = year {
        filters.year = Some(y);
    }
    if let Some(s) = search {
        filters.search = Some(s);
    }

    filters.sort = ContentSortField::from_str(&sort);
    filters.limit = limit;
    filters.offset = offset;

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

// Helper: converte data YYYY-MM-DD in timestamp UNIX
fn parse_date_to_timestamp(date_str: &str) -> Result<i64, Box<dyn std::error::Error>> {
    use chrono::NaiveDate;

    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| format!("Formato data non valido: '{}'. Usa YYYY-MM-DD", date_str))?;

    // Converte a timestamp UNIX (inizio del giorno in UTC)
    Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
}

// Helper: calcola timestamp di N giorni fa
fn timestamp_days_ago(days: i64) -> i64 {
    use chrono::{Duration, Utc};

    let now = Utc::now();
    let past = now - Duration::days(days);
    past.timestamp()
}

// Helper: calcola timestamp di N mesi fa (approssimato a 30 giorni per mese)
fn timestamp_months_ago(months: i64) -> i64 {
    use chrono::{Duration, Utc};

    let now = Utc::now();
    let past = now - Duration::days(months * 30);
    past.timestamp()
}

/// Comando: save-preset - Salva un preset di filtri
#[allow(clippy::too_many_arguments)]
fn cmd_save_preset(
    cli_library: &Option<PathBuf>,
    app_settings: &mut AppSettings,
    settings_path: &PathBuf,
    preset_type: String,
    name: String,
    in_library: bool,
    description: Option<String>,
    author: Option<String>,
    publisher: Option<String>,
    series: Option<String>,
    format: Option<String>,
    year: Option<i32>,
    isbn: Option<String>,
    search: Option<String>,
    acquired_after: Option<String>,
    acquired_before: Option<String>,
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

    // Converti date da stringa a timestamp se presenti
    let acquired_after_ts = if let Some(date_str) = &acquired_after {
        Some(parse_date_to_timestamp(date_str)?)
    } else {
        None
    };

    let acquired_before_ts = if let Some(date_str) = &acquired_before {
        Some(parse_date_to_timestamp(date_str)?)
    } else {
        None
    };

    // Se in_library è true, salva nei preset della libreria
    if in_library {
        let library_path = get_library_path(cli_library, app_settings)?;
        let config = LibraryConfig::new(&library_path);

        if !config.exists() {
            return Err(format!("La libreria non esiste: {}", library_path.display()).into());
        }

        let mut library_presets = config.load_library_presets()?;

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
                    acquired_after: acquired_after_ts,
                    acquired_before: acquired_before_ts,
                    sort,
                    limit,
                    offset,
                };

                let preset = NamedPreset {
                    name: name.clone(),
                    description,
                    filters,
                };

                library_presets.add_book_preset(preset);
                config.save_library_presets(&library_presets)?;

                println!("✓ Preset '{}' salvato nella libreria per libri", name);
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

                library_presets.add_content_preset(preset);
                config.save_library_presets(&library_presets)?;

                println!("✓ Preset '{}' salvato nella libreria per contenuti", name);
            }
        }
    } else {
        // Salva nei preset globali
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
                    acquired_after: acquired_after_ts,
                    acquired_before: acquired_before_ts,
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

                println!("✓ Preset '{}' salvato globalmente per libri", name);
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

                println!("✓ Preset '{}' salvato globalmente per contenuti", name);
            }
        }
    }

    Ok(())
}

/// Comando: list-presets - Lista tutti i preset salvati
fn cmd_list_presets(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    preset_type: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let show_books = preset_type.is_none()
        || preset_type.as_ref().map(|s| s.to_lowercase()) == Some("books".to_string())
        || preset_type.as_ref().map(|s| s.to_lowercase()) == Some("book".to_string());

    let show_contents = preset_type.is_none()
        || preset_type.as_ref().map(|s| s.to_lowercase()) == Some("contents".to_string())
        || preset_type.as_ref().map(|s| s.to_lowercase()) == Some("content".to_string());

    // Prova a caricare preset della libreria
    let library_presets = if let Ok(library_path) = get_library_path(cli_library, app_settings) {
        let config = LibraryConfig::new(&library_path);
        if config.exists() {
            config.load_library_presets().ok()
        } else {
            None
        }
    } else {
        None
    };

    let mut found_any = false;

    // Mostra preset della libreria
    if let Some(ref lib_presets) = library_presets {
        if show_books && !lib_presets.books.is_empty() {
            println!("Preset per Libri (Libreria):");
            println!("{}", "-".repeat(50));
            for (name, preset) in &lib_presets.books {
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

            // Mostra default se presente
            if let Some(default) = lib_presets.get_default_books_preset() {
                println!("Default: {}", default);
                println!();
            }

            found_any = true;
        }

        if show_contents && !lib_presets.contents.is_empty() {
            println!("Preset per Contenuti (Libreria):");
            println!("{}", "-".repeat(50));
            for (name, preset) in &lib_presets.contents {
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

            // Mostra default se presente
            if let Some(default) = lib_presets.get_default_contents_preset() {
                println!("Default: {}", default);
                println!();
            }

            found_any = true;
        }
    }

    // Mostra preset globali
    if show_books && !app_settings.presets.books.is_empty() {
        println!("Preset per Libri (Globali):");
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
        println!("Preset per Contenuti (Globali):");
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

/// Comando: set-default-filter - Imposta il preset di default per una libreria
fn cmd_set_default_filter(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    preset_type: String,
    preset_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let preset_type_enum = PresetType::from_str(&preset_type).ok_or_else(|| {
        format!(
            "Tipo preset non valido: '{}'. Usa 'books' o 'contents'",
            preset_type
        )
    })?;

    let library_path = get_library_path(cli_library, app_settings)?;
    let config = LibraryConfig::new(&library_path);

    if !config.exists() {
        return Err(format!("La libreria non esiste: {}", library_path.display()).into());
    }

    let mut library_presets = config.load_library_presets()?;

    // Controlla se rimuovere il default
    if preset_name.to_lowercase() == "none" {
        match preset_type_enum {
            PresetType::Books => {
                library_presets.set_default_books_preset(None);
                println!("✓ Preset di default rimosso per libri");
            }
            PresetType::Contents => {
                library_presets.set_default_contents_preset(None);
                println!("✓ Preset di default rimosso per contenuti");
            }
        }
    } else {
        // Verifica che il preset esista
        let exists = match preset_type_enum {
            PresetType::Books => library_presets.get_book_preset(&preset_name).is_some(),
            PresetType::Contents => library_presets.get_content_preset(&preset_name).is_some(),
        };

        if !exists {
            return Err(format!(
                "Preset '{}' non trovato nella libreria. Usa 'ritmo save-preset --in-library' per crearlo.",
                preset_name
            )
            .into());
        }

        match preset_type_enum {
            PresetType::Books => {
                library_presets.set_default_books_preset(Some(preset_name.clone()));
                println!(
                    "✓ Preset '{}' impostato come default per libri",
                    preset_name
                );
            }
            PresetType::Contents => {
                library_presets.set_default_contents_preset(Some(preset_name.clone()));
                println!(
                    "✓ Preset '{}' impostato come default per contenuti",
                    preset_name
                );
            }
        }
    }

    config.save_library_presets(&library_presets)?;

    Ok(())
}

/// Comando: add - Importa un libro nella libreria
async fn cmd_add(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    file: PathBuf,
    title: String,
    author: Option<String>,
    publisher: Option<String>,
    year: Option<i32>,
    isbn: Option<String>,
    format: Option<String>,
    series: Option<String>,
    series_index: Option<i64>,
    notes: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determina quale libreria usare
    let library_path = if let Some(path) = cli_library {
        path.clone()
    } else if let Some(portable) = detect_portable_library() {
        portable
    } else if let Some(path) = &app_settings.last_library_path {
        path.clone()
    } else {
        println!("✗ Nessuna libreria configurata");
        println!("  Usa 'ritmo init' per inizializzare una libreria");
        return Ok(());
    };

    // Verifica che il file esista
    if !file.exists() {
        println!("✗ File non trovato: {}", file.display());
        return Ok(());
    }

    println!("Importazione libro: {}", file.display());
    println!("  Titolo: {}", title);
    if let Some(ref a) = author {
        println!("  Autore: {}", a);
    }
    if let Some(ref p) = publisher {
        println!("  Editore: {}", p);
    }
    if let Some(y) = year {
        println!("  Anno: {}", y);
    }

    // Crea config e pool
    let config = LibraryConfig::new(&library_path);
    let pool = config.create_pool().await?;

    // Prepara metadati
    let metadata = BookImportMetadata {
        title,
        author,
        publisher,
        year,
        isbn,
        format,
        series,
        series_index,
        notes,
    };

    // Importa il libro
    match import_book(&config, &pool, &file, metadata).await {
        Ok(book_id) => {
            println!("✓ Libro importato con successo!");
            println!("  ID: {}", book_id);
        }
        Err(e) => {
            println!("✗ Errore durante l'importazione: {}", e);
        }
    }

    Ok(())
}
