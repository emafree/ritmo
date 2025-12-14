use clap::{Parser, Subcommand};
use ritmo_config::{detect_portable_library, settings_file, AppSettings};
use ritmo_db_core::LibraryConfig;
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
