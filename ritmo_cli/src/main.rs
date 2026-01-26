mod commands;
mod formatter;
mod helpers;

use clap::{Parser, Subcommand};
use commands::*;
use ritmo_config::{settings_file, AppSettings};
use ritmo_errors::reporter::RitmoReporter;
use std::path::PathBuf;

/// CLI Reporter implementation that prints to stdout/stderr
struct CliReporter;

impl RitmoReporter for CliReporter {
    fn status(&mut self, message: &str) {
        println!("{}", message);
    }

    fn progress(&mut self, message: &str) {
        println!("{}", message);
    }

    fn error(&mut self, message: &str) {
        eprintln!("Error: {}", message);
    }
}

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

        /// Titolo originale
        #[arg(long)]
        original_title: Option<String>,

        /// Persone con ruoli (formato: "Nome:Ruolo", es. "Stephen King:Autore", può essere specificato più volte)
        #[arg(long)]
        people: Vec<String>,

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

        /// Numero di pagine
        #[arg(long)]
        pages: Option<i64>,

        /// Note
        #[arg(long, short = 'n')]
        notes: Option<String>,

        /// Tags (può essere specificato più volte)
        #[arg(long)]
        tags: Vec<String>,
    },

    /// Aggiorna metadati di un libro esistente
    UpdateBook {
        /// ID del libro da aggiornare
        id: i64,

        /// Nuovo titolo
        #[arg(long)]
        title: Option<String>,

        /// Nuovo titolo originale
        #[arg(long)]
        original_title: Option<String>,

        /// Nuove persone con ruoli (formato: "Nome:Ruolo", sostituisce tutte le persone esistenti)
        #[arg(long)]
        people: Vec<String>,

        /// Nuovo editore
        #[arg(long)]
        publisher: Option<String>,

        /// Nuovo anno di pubblicazione
        #[arg(long)]
        year: Option<i32>,

        /// Nuovo ISBN
        #[arg(long)]
        isbn: Option<String>,

        /// Nuovo formato
        #[arg(long)]
        format: Option<String>,

        /// Nuova serie
        #[arg(long)]
        series: Option<String>,

        /// Nuovo indice nella serie
        #[arg(long)]
        series_index: Option<i64>,

        /// Nuove note
        #[arg(long)]
        notes: Option<String>,

        /// Numero di pagine
        #[arg(long)]
        pages: Option<i64>,

        /// Nuovi tags (sostituiscono tutti i tags esistenti, può essere specificato più volte)
        #[arg(long)]
        tags: Vec<String>,
    },

    /// Elimina un libro dal database
    DeleteBook {
        /// ID del libro da eliminare
        id: i64,

        /// Elimina anche il file fisico dallo storage
        #[arg(long)]
        delete_file: bool,

        /// Forza l'eliminazione anche in caso di errori filesystem
        #[arg(long)]
        force: bool,
    },

    /// Aggiorna metadati di un contenuto esistente
    UpdateContent {
        /// ID del contenuto da aggiornare
        id: i64,

        /// Nuovo titolo
        #[arg(long)]
        title: Option<String>,

        /// Nuovo titolo originale
        #[arg(long)]
        original_title: Option<String>,

        /// Nuove persone con ruoli (formato: "Nome:Ruolo", sostituisce tutte le persone esistenti)
        #[arg(long)]
        people: Vec<String>,

        /// Nuovo tipo di contenuto
        #[arg(long)]
        content_type: Option<String>,

        /// Nuovo anno di pubblicazione
        #[arg(long)]
        year: Option<i32>,

        /// Nuove note
        #[arg(long)]
        notes: Option<String>,

        /// Numero di pagine
        #[arg(long)]
        pages: Option<i64>,

        /// Nuovi tags (sostituiscono tutti i tags esistenti, può essere specificato più volte)
        #[arg(long)]
        tags: Vec<String>,

        /// Nuove languages (sostituiscono tutte le lingue esistenti, formato: "Nome:iso2:iso3:role")
        #[arg(long)]
        languages: Vec<String>,
    },

    /// Crea un nuovo contenuto
    AddContent {
        /// Titolo del contenuto (richiesto)
        #[arg(long, short = 't')]
        title: String,

        /// Titolo originale
        #[arg(long)]
        original_title: Option<String>,

        /// Persone con ruoli (formato: "Nome:Ruolo", es. "Stephen King:Autore", può essere specificato più volte)
        #[arg(long)]
        people: Vec<String>,

        /// Tipo di contenuto (Romanzo, Racconto, Saggio, etc.)
        #[arg(long)]
        content_type: Option<String>,

        /// Anno di pubblicazione
        #[arg(long, short = 'y')]
        year: Option<i32>,

        /// Numero di pagine
        #[arg(long)]
        pages: Option<i64>,

        /// Note
        #[arg(long, short = 'n')]
        notes: Option<String>,

        /// ID del libro a cui associare il contenuto (opzionale)
        #[arg(long, short = 'b')]
        book_id: Option<i64>,

        /// Tags (può essere specificato più volte)
        #[arg(long)]
        tags: Vec<String>,

        /// Languages (formato: "Nome:iso2:iso3:role", es. "Italian:it:ita:Original")
        #[arg(long)]
        languages: Vec<String>,
    },

    /// Elimina un contenuto dal database
    DeleteContent {
        /// ID del contenuto da eliminare
        id: i64,
    },

    /// Associa un contenuto a un libro
    LinkContent {
        /// ID del contenuto
        #[arg(long, short = 'c')]
        content_id: i64,

        /// ID del libro
        #[arg(long, short = 'b')]
        book_id: i64,
    },

    /// Rimuovi l'associazione tra un contenuto e un libro
    UnlinkContent {
        /// ID del contenuto
        #[arg(long, short = 'c')]
        content_id: i64,

        /// ID del libro
        #[arg(long, short = 'b')]
        book_id: i64,
    },

    /// Pulisci entità orfane (autori, editori, serie non referenziati)
    Cleanup {
        /// Mostra cosa verrebbe eliminato senza applicare modifiche
        #[arg(long)]
        dry_run: bool,
    },

    /// Find and merge duplicate authors using ML
    DeduplicateAuthors {
        /// Minimum confidence threshold (0.0-1.0)
        #[arg(long, short = 't', default_value = "0.85")]
        threshold: f64,

        /// Automatically merge high-confidence duplicates
        #[arg(long)]
        auto_merge: bool,

        /// Show what would be merged without making changes (default: true)
        #[arg(long)]
        dry_run: bool,
    },

    /// Find and merge duplicate publishers using ML
    DeduplicatePublishers {
        /// Minimum confidence threshold (0.0-1.0)
        #[arg(long, short = 't', default_value = "0.85")]
        threshold: f64,

        /// Automatically merge high-confidence duplicates
        #[arg(long)]
        auto_merge: bool,

        /// Show what would be merged without making changes (default: true)
        #[arg(long)]
        dry_run: bool,
    },

    /// Find and merge duplicate series using ML
    DeduplicateSeries {
        /// Minimum confidence threshold (0.0-1.0)
        #[arg(long, short = 't', default_value = "0.85")]
        threshold: f64,

        /// Automatically merge high-confidence duplicates
        #[arg(long)]
        auto_merge: bool,

        /// Show what would be merged without making changes (default: true)
        #[arg(long)]
        dry_run: bool,
    },

    /// Find and merge duplicate tags using ML
    DeduplicateTags {
        /// Minimum confidence threshold (0.0-1.0)
        #[arg(long, short = 't', default_value = "0.85")]
        threshold: f64,

        /// Automatically merge high-confidence duplicates
        #[arg(long)]
        auto_merge: bool,

        /// Show what would be merged without making changes (default: true)
        #[arg(long)]
        dry_run: bool,
    },

    /// Find and merge duplicate roles using ML
    DeduplicateRoles {
        /// Minimum confidence threshold (0.0-1.0)
        #[arg(long, short = 't', default_value = "0.85")]
        threshold: f64,

        /// Automatically merge high-confidence duplicates
        #[arg(long)]
        auto_merge: bool,

        /// Show what would be merged without making changes (default: true)
        #[arg(long)]
        dry_run: bool,
    },

    /// Find and merge all duplicate entities (authors, publishers, series, tags, roles) using ML
    DeduplicateAll {
        /// Minimum confidence threshold (0.0-1.0)
        #[arg(long, short = 't', default_value = "0.85")]
        threshold: f64,

        /// Automatically merge high-confidence duplicates
        #[arg(long)]
        auto_merge: bool,

        /// Show what would be merged without making changes (default: true)
        #[arg(long)]
        dry_run: bool,
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
            original_title,
            people,
            publisher,
            year,
            isbn,
            format,
            series,
            series_index,
            pages,
            notes,
            tags,
        } => {
            cmd_add(
                &cli.library,
                &app_settings,
                file,
                title,
                original_title,
                people,
                publisher,
                year,
                isbn,
                format,
                series,
                series_index,
                pages,
                notes,
                tags,
            )
            .await?;
        }
        Commands::UpdateBook {
            id,
            title,
            original_title,
            people,
            publisher,
            year,
            isbn,
            format,
            series,
            series_index,
            notes,
            pages,
            tags,
        } => {
            cmd_update_book(
                &cli.library,
                &app_settings,
                id,
                title,
                original_title,
                people,
                publisher,
                year,
                isbn,
                format,
                series,
                series_index,
                notes,
                pages,
                tags,
            )
            .await?;
        }
        Commands::DeleteBook {
            id,
            delete_file,
            force,
        } => {
            cmd_delete_book(&cli.library, &app_settings, id, delete_file, force).await?;
        }
        Commands::AddContent {
            title,
            original_title,
            people,
            content_type,
            year,
            pages,
            notes,
            book_id,
            tags,
            languages,
        } => {
            cmd_add_content(
                &cli.library,
                &app_settings,
                title,
                original_title,
                people,
                content_type,
                year,
                pages,
                notes,
                book_id,
                tags,
                languages,
            )
            .await?;
        }
        Commands::UpdateContent {
            id,
            title,
            original_title,
            people,
            content_type,
            year,
            notes,
            pages,
            tags,
            languages,
        } => {
            cmd_update_content(
                &cli.library,
                &app_settings,
                id,
                title,
                original_title,
                people,
                content_type,
                year,
                notes,
                pages,
                tags,
                languages,
            )
            .await?;
        }
        Commands::DeleteContent { id } => {
            cmd_delete_content(&cli.library, &app_settings, id).await?;
        }
        Commands::LinkContent {
            content_id,
            book_id,
        } => {
            cmd_link_content(&cli.library, &app_settings, content_id, book_id).await?;
        }
        Commands::UnlinkContent {
            content_id,
            book_id,
        } => {
            cmd_unlink_content(&cli.library, &app_settings, content_id, book_id).await?;
        }
        Commands::Cleanup { dry_run } => {
            cmd_cleanup(&cli.library, &app_settings, dry_run).await?;
        }
        Commands::DeduplicateAuthors {
            threshold,
            auto_merge,
            dry_run,
        } => {
            cmd_deduplicate_authors(&cli.library, &app_settings, threshold, auto_merge, dry_run)
                .await?;
        }
        Commands::DeduplicatePublishers {
            threshold,
            auto_merge,
            dry_run,
        } => {
            cmd_deduplicate_publishers(
                &cli.library,
                &app_settings,
                threshold,
                auto_merge,
                dry_run,
            )
            .await?;
        }
        Commands::DeduplicateSeries {
            threshold,
            auto_merge,
            dry_run,
        } => {
            cmd_deduplicate_series(&cli.library, &app_settings, threshold, auto_merge, dry_run)
                .await?;
        }
        Commands::DeduplicateTags {
            threshold,
            auto_merge,
            dry_run,
        } => {
            cmd_deduplicate_tags(&cli.library, &app_settings, threshold, auto_merge, dry_run)
                .await?;
        }
        Commands::DeduplicateRoles {
            threshold,
            auto_merge,
            dry_run,
        } => {
            cmd_deduplicate_roles(&cli.library, &app_settings, threshold, auto_merge, dry_run)
                .await?;
        }
        Commands::DeduplicateAll {
            threshold,
            auto_merge,
            dry_run,
        } => {
            cmd_deduplicate_all(&cli.library, &app_settings, threshold, auto_merge, dry_run)
                .await?;
        }
    }

    Ok(())
}
