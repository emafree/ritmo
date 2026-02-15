// Initialize rust-i18n with translation files
rust_i18n::i18n!("../locales", fallback = "en");

mod commands;
mod confirmation;
mod filter_args;
mod formatter;
mod handlers;
mod helpers;

use clap::{Parser, Subcommand};
use commands::*;
use ritmo_config::{settings_file, AppSettings};
use ritmo_db::i18n_utils;
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
    /// Library management operations
    #[command(subcommand)]
    Libraries(LibrariesCommands),

    /// Language preference management
    #[command(subcommand)]
    Language(LanguageCommands),

    /// Filter presets management
    #[command(subcommand)]
    Presets(PresetsCommands),

    /// Book operations (CRUD + cleanup)
    #[command(subcommand)]
    Books(BooksCommands),

    /// Content operations (CRUD + link/unlink)
    #[command(subcommand)]
    Contents(ContentsCommands),

    /// Entity deduplication using ML
    #[command(subcommand)]
    Deduplicate(DeduplicateCommands),

    /// Metadata synchronization
    #[command(subcommand)]
    Sync(SyncCommands),

    /// Tag management (CRUD)
    #[command(subcommand)]
    Tags(TagsCommands),

    /// Publisher management (CRUD)
    #[command(subcommand)]
    Publishers(PublishersCommands),

    /// Series management (CRUD)
    #[command(subcommand)]
    Series(SeriesCommands),

    /// People management (CRUD)
    #[command(subcommand)]
    People(PeopleCommands),
}

/// Libraries subcommands
#[derive(Subcommand)]
enum LibrariesCommands {
    /// Initialize a new library
    New {
        /// Library path (default: ~/RitmoLibrary)
        path: Option<PathBuf>,
    },

    /// Duplicate the current portable library to a new location
    Duplicate {
        /// Output path for the duplicated library
        path: PathBuf,
    },

    /// Show current library information
    Info,

    /// List all recent libraries
    List,

    /// Set current library
    Set {
        /// Library path to set as current
        path: PathBuf,
    },
}

/// Language subcommands
#[derive(Subcommand)]
enum LanguageCommands {
    /// Set preferred language
    Set {
        /// Language code (en, it)
        lang: String,
    },

    /// Get current language preference
    Get,
}

/// Presets subcommands
#[derive(Subcommand)]
enum PresetsCommands {
    /// Save a filter preset
    Save {
        /// Preset type: books or contents
        preset_type: String,

        /// Preset name
        #[arg(long)]
        name: String,

        /// Description
        #[arg(long)]
        description: Option<String>,

        /// Save in current library instead of globally
        #[arg(long)]
        in_library: bool,

        #[command(flatten)]
        filters: filter_args::PresetFilterArgs,
    },

    /// List all saved presets
    List {
        /// Filter by type (books, contents)
        #[arg(long)]
        preset_type: Option<String>,

        /// Show global presets
        #[arg(long)]
        global: bool,
    },

    /// Delete a preset
    Delete {
        /// Preset name to delete
        name: String,

        /// Delete from library instead of global
        #[arg(long)]
        in_library: bool,
    },

    /// Set default preset for a library
    SetDefault {
        /// Preset name
        name: String,

        /// Preset type (books, contents)
        preset_type: String,
    },
}

/// Deduplication subcommands
#[derive(Subcommand)]
enum DeduplicateCommands {
    /// Find and merge duplicate people (authors, translators, etc.)
    People {
        /// Entity name to filter duplicates (optional)
        entity_name: Option<String>,

        /// Similarity threshold (0.0-1.0)
        #[arg(long, default_value = "0.85")]
        threshold: f64,

        /// Automatically merge duplicates without confirmation
        #[arg(long)]
        auto_merge: bool,

        /// Preview duplicates without merging
        #[arg(long)]
        dry_run: bool,

        /// Enable interactive mode to choose canonical entity
        #[arg(short = 'i', long)]
        interactive: bool,
    },

    /// Find and merge duplicate publishers
    Publishers {
        /// Entity name to filter duplicates (optional)
        entity_name: Option<String>,

        /// Similarity threshold (0.0-1.0)
        #[arg(long, default_value = "0.85")]
        threshold: f64,

        /// Automatically merge duplicates without confirmation
        #[arg(long)]
        auto_merge: bool,

        /// Preview duplicates without merging
        #[arg(long)]
        dry_run: bool,

        /// Enable interactive mode to choose canonical entity
        #[arg(short = 'i', long)]
        interactive: bool,
    },

    /// Find and merge duplicate series
    Series {
        /// Entity name to filter duplicates (optional)
        entity_name: Option<String>,

        /// Similarity threshold (0.0-1.0)
        #[arg(long, default_value = "0.85")]
        threshold: f64,

        /// Automatically merge duplicates without confirmation
        #[arg(long)]
        auto_merge: bool,

        /// Preview duplicates without merging
        #[arg(long)]
        dry_run: bool,

        /// Enable interactive mode to choose canonical entity
        #[arg(short = 'i', long)]
        interactive: bool,
    },

    /// Find and merge duplicate tags
    Tags {
        /// Entity name to filter duplicates (optional)
        entity_name: Option<String>,

        /// Similarity threshold (0.0-1.0)
        #[arg(long, default_value = "0.85")]
        threshold: f64,

        /// Automatically merge duplicates without confirmation
        #[arg(long)]
        auto_merge: bool,

        /// Preview duplicates without merging
        #[arg(long)]
        dry_run: bool,

        /// Enable interactive mode to choose canonical entity
        #[arg(short = 'i', long)]
        interactive: bool,
    },

    /// Find and merge duplicate roles
    Roles {
        /// Entity name to filter duplicates (optional)
        entity_name: Option<String>,

        /// Similarity threshold (0.0-1.0)
        #[arg(long, default_value = "0.85")]
        threshold: f64,

        /// Automatically merge duplicates without confirmation
        #[arg(long)]
        auto_merge: bool,

        /// Preview duplicates without merging
        #[arg(long)]
        dry_run: bool,

        /// Enable interactive mode to choose canonical entity
        #[arg(short = 'i', long)]
        interactive: bool,
    },

    /// Find and merge all duplicate entities
    All {
        /// Similarity threshold (0.0-1.0)
        #[arg(long, default_value = "0.85")]
        threshold: f64,

        /// Automatically merge duplicates without confirmation
        #[arg(long)]
        auto_merge: bool,

        /// Preview duplicates without merging
        #[arg(long)]
        dry_run: bool,
    },
}

/// Sync subcommands
#[derive(Subcommand)]
enum SyncCommands {
    /// Sync EPUB metadata with database
    Metadata {
        /// Show status without syncing
        #[arg(long)]
        status: bool,

        /// Preview changes without applying
        #[arg(long)]
        dry_run: bool,
    },
}

/// Tags subcommands
#[derive(Subcommand)]
enum TagsCommands {
    /// List all tags
    List {
        /// Formato output (table, json, simple)
        #[arg(long, short = 'o', default_value = "table")]
        output: String,
    },
    /// Create a new tag
    Create {
        /// Tag name (required)
        #[arg(long, short = 'n')]
        name: String,
        /// Optional description
        #[arg(long, short = 'd')]
        description: Option<String>,
    },
    /// Update an existing tag
    Update {
        /// Tag ID to update
        #[arg(long)]
        id: i64,
        /// New tag name (optional)
        #[arg(long, short = 'n')]
        name: Option<String>,
        /// New description (optional)
        #[arg(long, short = 'd')]
        description: Option<String>,
    },
    /// Delete a tag
    Delete {
        /// Tag ID to delete
        #[arg(long)]
        id: i64,
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

/// Publishers subcommands
#[derive(Subcommand)]
enum PublishersCommands {
    /// List all publishers
    List {
        /// Formato output (table, json, simple)
        #[arg(long, short = 'o', default_value = "table")]
        output: String,
    },
    /// Create a new publisher
    Create {
        /// Publisher name (required)
        #[arg(long, short = 'n')]
        name: String,
        /// Country (optional)
        #[arg(long, short = 'c')]
        country: Option<String>,
        /// Website (optional)
        #[arg(long, short = 'w')]
        website: Option<String>,
        /// Notes (optional)
        #[arg(long)]
        notes: Option<String>,
    },
    /// Update an existing publisher
    Update {
        /// Publisher ID to update
        #[arg(long)]
        id: i64,
        /// New publisher name (optional)
        #[arg(long, short = 'n')]
        name: Option<String>,
        /// New country (optional)
        #[arg(long, short = 'c')]
        country: Option<String>,
        /// New website (optional)
        #[arg(long, short = 'w')]
        website: Option<String>,
        /// New notes (optional)
        #[arg(long)]
        notes: Option<String>,
    },
    /// Delete a publisher
    Delete {
        /// Publisher ID to delete
        #[arg(long)]
        id: i64,
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

/// Series subcommands
#[derive(Subcommand)]
enum SeriesCommands {
    /// List all series
    List {
        /// Formato output (table, json, simple)
        #[arg(long, short = 'o', default_value = "table")]
        output: String,
    },
    /// Create a new series
    Create {
        /// Series name (required)
        #[arg(long, short = 'n')]
        name: String,
        /// Description (optional)
        #[arg(long, short = 'd')]
        description: Option<String>,
        /// Total number of books in series (optional)
        #[arg(long, short = 't')]
        total_books: Option<i64>,
        /// Whether the series is completed
        #[arg(long, short = 'c')]
        completed: bool,
    },
    /// Update an existing series
    Update {
        /// Series ID to update
        #[arg(long)]
        id: i64,
        /// New series name (optional)
        #[arg(long, short = 'n')]
        name: Option<String>,
        /// New description (optional)
        #[arg(long, short = 'd')]
        description: Option<String>,
        /// New total books (optional)
        #[arg(long, short = 't')]
        total_books: Option<i64>,
        /// New completion status (optional)
        #[arg(long, short = 'c')]
        completed: Option<bool>,
    },
    /// Delete a series
    Delete {
        /// Series ID to delete
        #[arg(long)]
        id: i64,
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

/// People subcommands
#[derive(Subcommand)]
enum PeopleCommands {
    /// List all people
    List {
        /// Formato output (table, json, simple)
        #[arg(long, short = 'o', default_value = "table")]
        output: String,
    },
    /// Create a new person
    Create {
        /// Person name (required)
        #[arg(long, short = 'n')]
        name: String,
        /// Display name (optional)
        #[arg(long)]
        display_name: Option<String>,
        /// Given name (optional)
        #[arg(long)]
        given_name: Option<String>,
        /// Surname (optional)
        #[arg(long)]
        surname: Option<String>,
        /// Nationality (optional)
        #[arg(long)]
        nationality: Option<String>,
        /// Biography (optional)
        #[arg(long)]
        biography: Option<String>,
    },
    /// Update an existing person
    Update {
        /// Person ID to update
        #[arg(long)]
        id: i64,
        /// New person name (optional)
        #[arg(long, short = 'n')]
        name: Option<String>,
        /// New display name (optional)
        #[arg(long)]
        display_name: Option<String>,
        /// New given name (optional)
        #[arg(long)]
        given_name: Option<String>,
        /// New surname (optional)
        #[arg(long)]
        surname: Option<String>,
        /// New nationality (optional)
        #[arg(long)]
        nationality: Option<String>,
        /// New biography (optional)
        #[arg(long)]
        biography: Option<String>,
    },
    /// Delete a person
    Delete {
        /// Person ID to delete
        #[arg(long)]
        id: i64,
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

/// Books subcommands
#[derive(Subcommand)]
enum BooksCommands {
    /// Add single book with metadata
    Add {
        /// Percorso del file da importare
        file: PathBuf,

        /// Titolo del libro (richiesto)
        #[arg(long, short = 't')]
        title: String,

        /// Titolo originale
        #[arg(long)]
        original_title: Option<String>,

        /// Persone con ruoli (formato: "Nome:Ruolo", es. "Stephen King:Autore")
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

    /// Batch import from JSON
    AddBatch {
        /// Percorso del file JSON con metadata (opzionale, legge da stdin se omesso)
        #[arg(long, short = 'i')]
        input: Option<PathBuf>,

        /// Continua su errori invece di fermarsi al primo errore
        #[arg(long)]
        continue_on_error: bool,

        /// Modalità dry-run: valida il JSON senza importare
        #[arg(long)]
        dry_run: bool,
    },

    /// List books with filters
    List {
        #[command(flatten)]
        filters: filter_args::BookFilterArgs,

        /// Formato output (table, json, simple)
        #[arg(long, short = 'o', default_value = "table")]
        output: String,
    },

    /// Update book(s) by ID or filters with bulk support
    Update {
        #[command(flatten)]
        selector: filter_args::BookBulkUpdateSelector,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,

        /// Preview changes without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Delete book(s) by ID or filters
    Delete {
        /// Book ID to delete (mutually exclusive with filter args)
        #[arg(long, conflicts_with_all = ["author", "publisher", "series", "format", "year", "isbn", "search"])]
        id: Option<i64>,

        #[command(flatten)]
        filters: filter_args::BookFilterArgs,

        /// Elimina anche il file fisico dallo storage
        #[arg(long)]
        delete_file: bool,

        /// Forza l'eliminazione anche in caso di errori filesystem
        #[arg(long)]
        force: bool,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,

        /// Preview changes without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Clean up orphaned entities
    Cleanup {
        /// Mostra cosa verrebbe eliminato senza applicare modifiche
        #[arg(long)]
        dry_run: bool,
    },
}

/// Contents subcommands
#[derive(Subcommand)]
enum ContentsCommands {
    /// Add new content with metadata
    Add {
        /// Titolo del contenuto (richiesto)
        #[arg(long, short = 't')]
        title: String,

        /// Titolo originale
        #[arg(long)]
        original_title: Option<String>,

        /// Persone con ruoli (formato: "Nome:Ruolo")
        #[arg(long)]
        people: Vec<String>,

        /// Tipo di contenuto (Romanzo, Racconto, Saggio, etc.)
        #[arg(long)]
        content_type: Option<String>,

        /// Anno di pubblicazione
        #[arg(long, short = 'y')]
        year: Option<i32>,

        /// Note
        #[arg(long, short = 'n')]
        notes: Option<String>,

        /// Numero di pagine
        #[arg(long)]
        pages: Option<i64>,

        /// Tags
        #[arg(long)]
        tags: Vec<String>,

        /// Lingue (formato: "it:original" o "en:actual")
        #[arg(long)]
        languages: Vec<String>,

        /// ID del libro a cui associare il contenuto (opzionale)
        #[arg(long)]
        book_id: Option<i64>,
    },

    /// List contents with filters
    List {
        #[command(flatten)]
        filters: filter_args::ContentFilterArgs,

        /// Formato output (table, json, simple)
        #[arg(long, short = 'o', default_value = "table")]
        output: String,
    },

    /// Update content(s) by ID or filters with bulk support
    Update {
        #[command(flatten)]
        selector: filter_args::ContentBulkUpdateSelector,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,

        /// Preview changes without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Delete content(s) by ID or filters
    Delete {
        /// Content ID to delete (mutually exclusive with filter args)
        #[arg(long, conflicts_with_all = ["author", "content_type", "year", "search"])]
        id: Option<i64>,

        #[command(flatten)]
        filters: filter_args::ContentFilterArgs,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,

        /// Preview changes without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Link content to book
    Link {
        /// Content ID
        #[arg(long)]
        content_id: i64,

        /// Book ID
        #[arg(long)]
        book_id: i64,
    },

    /// Unlink content from book
    Unlink {
        /// Content ID
        #[arg(long)]
        content_id: i64,

        /// Book ID
        #[arg(long)]
        book_id: i64,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Carica o crea AppSettings
    let settings_path = settings_file()?;
    let mut app_settings = AppSettings::load_or_create(&settings_path)?;

    // Initialize i18n system (priority: RITMO_LANG env var > saved preference > LANG env var > default)
    i18n_utils::init_i18n_with_preference(Some(app_settings.get_language()));

    match cli.command {
        // === NEW NESTED COMMANDS ===
        Commands::Libraries(lib_cmd) => match lib_cmd {
            LibrariesCommands::New { path } => {
                cmd_new(path, &mut app_settings, &settings_path).await?;
            }
            LibrariesCommands::Duplicate { path } => {
                cmd_duplicate(path, &mut app_settings, &settings_path).await?;
            }
            LibrariesCommands::Info => {
                cmd_info(&cli.library, &app_settings).await?;
            }
            LibrariesCommands::List => {
                cmd_list_libraries(&app_settings)?;
            }
            LibrariesCommands::Set { path } => {
                cmd_set_library(path, &mut app_settings, &settings_path)?;
            }
        },

        Commands::Language(lang_cmd) => match lang_cmd {
            LanguageCommands::Set { lang } => {
                cmd_set_language(lang, &mut app_settings, &settings_path)?;
            }
            LanguageCommands::Get => {
                cmd_get_language(&app_settings);
            }
        },

        Commands::Presets(preset_cmd) => match preset_cmd {
            PresetsCommands::Save {
                preset_type,
                name,
                description,
                in_library,
                filters,
            } => {
                cmd_save_preset(
                    &cli.library,
                    &mut app_settings,
                    &settings_path,
                    preset_type,
                    name,
                    in_library,
                    description,
                    filters.author,
                    filters.publisher,
                    filters.series,
                    filters.format,
                    filters.year,
                    filters.isbn,
                    filters.search,
                    filters.acquired_after,
                    filters.acquired_before,
                    filters.content_type,
                    filters.sort,
                    filters.limit,
                    filters.offset,
                )?;
            }
            PresetsCommands::List { preset_type, .. } => {
                cmd_list_presets(&cli.library, &app_settings, preset_type)?;
            }
            PresetsCommands::Delete { name, .. } => {
                // Note: in_library not used in original cmd, preset_type derived from context
                cmd_delete_preset(&mut app_settings, &settings_path, "books".to_string(), name)?;
            }
            PresetsCommands::SetDefault {
                name,
                preset_type,
            } => {
                cmd_set_default_filter(&cli.library, &app_settings, preset_type, name)?;
            }
        },

        Commands::Deduplicate(dedup_cmd) => match dedup_cmd {
            DeduplicateCommands::People {
                entity_name,
                threshold,
                auto_merge,
                dry_run,
                interactive,
            } => {
                cmd_deduplicate_people(&cli.library, &app_settings, entity_name, threshold, auto_merge, dry_run, interactive).await?;
            }
            DeduplicateCommands::Publishers {
                entity_name,
                threshold,
                auto_merge,
                dry_run,
                interactive,
            } => {
                cmd_deduplicate_publishers(&cli.library, &app_settings, entity_name, threshold, auto_merge, dry_run, interactive).await?;
            }
            DeduplicateCommands::Series {
                entity_name,
                threshold,
                auto_merge,
                dry_run,
                interactive,
            } => {
                cmd_deduplicate_series(&cli.library, &app_settings, entity_name, threshold, auto_merge, dry_run, interactive).await?;
            }
            DeduplicateCommands::Tags {
                entity_name,
                threshold,
                auto_merge,
                dry_run,
                interactive,
            } => {
                cmd_deduplicate_tags(&cli.library, &app_settings, entity_name, threshold, auto_merge, dry_run, interactive).await?;
            }
            DeduplicateCommands::Roles {
                entity_name,
                threshold,
                auto_merge,
                dry_run,
                interactive,
            } => {
                cmd_deduplicate_roles(&cli.library, &app_settings, entity_name, threshold, auto_merge, dry_run, interactive).await?;
            }
            DeduplicateCommands::All {
                threshold,
                auto_merge,
                dry_run,
            } => {
                cmd_deduplicate_all(&cli.library, &app_settings, threshold, auto_merge, dry_run).await?;
            }
        },

        Commands::Sync(sync_cmd) => match sync_cmd {
            SyncCommands::Metadata { status, dry_run } => {
                if status {
                    cmd_sync_status(&cli.library, &app_settings).await?;
                } else if dry_run {
                    cmd_sync_dry_run(&cli.library, &app_settings).await?;
                } else {
                    cmd_sync_metadata(&cli.library, &app_settings).await?;
                }
            }
        },

        Commands::Tags(tags_cmd) => match tags_cmd {
            TagsCommands::List { output } => {
                handlers::tags::handle_tags_list(&cli.library, &app_settings, &output).await?;
            }
            TagsCommands::Create { name, description } => {
                handlers::tags::handle_tags_create(
                    &cli.library,
                    &app_settings,
                    &name,
                    &description,
                )
                .await?;
            }
            TagsCommands::Update {
                id,
                name,
                description,
            } => {
                handlers::tags::handle_tags_update(
                    &cli.library,
                    &app_settings,
                    &id,
                    &name,
                    &description,
                )
                .await?;
            }
            TagsCommands::Delete { id, yes } => {
                handlers::tags::handle_tags_delete(&cli.library, &app_settings, &id, &yes).await?;
            }
        },

        Commands::Publishers(publishers_cmd) => match publishers_cmd {
            PublishersCommands::List { output } => {
                handlers::publishers::handle_publishers_list(&cli.library, &app_settings, &output).await?;
            }
            PublishersCommands::Create { name, country, website, notes } => {
                handlers::publishers::handle_publishers_create(&cli.library, &app_settings, &name, &country, &website, &notes).await?;
            }
            PublishersCommands::Update { id, name, country, website, notes } => {
                handlers::publishers::handle_publishers_update(&cli.library, &app_settings, &id, &name, &country, &website, &notes).await?;
            }
            PublishersCommands::Delete { id, yes } => {
                handlers::publishers::handle_publishers_delete(&cli.library, &app_settings, &id, &yes).await?;
            }
        },

        Commands::Series(series_cmd) => match series_cmd {
            SeriesCommands::List { output } => {
                handlers::series::handle_series_list(&cli.library, &app_settings, &output).await?;
            }
            SeriesCommands::Create { name, description, total_books, completed } => {
                handlers::series::handle_series_create(&cli.library, &app_settings, &name, &description, &total_books, &completed).await?;
            }
            SeriesCommands::Update { id, name, description, total_books, completed } => {
                handlers::series::handle_series_update(&cli.library, &app_settings, &id, &name, &description, &total_books, &completed).await?;
            }
            SeriesCommands::Delete { id, yes } => {
                handlers::series::handle_series_delete(&cli.library, &app_settings, &id, &yes).await?;
            }
        },

        Commands::People(people_cmd) => match people_cmd {
            PeopleCommands::List { output } => {
                handlers::people::handle_people_list(&cli.library, &app_settings, &output).await?;
            }
            PeopleCommands::Create { name, display_name, given_name, surname, nationality, biography } => {
                handlers::people::handle_people_create(&cli.library, &app_settings, &name, &display_name, &given_name, &surname, &nationality, &biography).await?;
            }
            PeopleCommands::Update { id, name, display_name, given_name, surname, nationality, biography } => {
                handlers::people::handle_people_update(&cli.library, &app_settings, &id, &name, &display_name, &given_name, &surname, &nationality, &biography).await?;
            }
            PeopleCommands::Delete { id, yes } => {
                handlers::people::handle_people_delete(&cli.library, &app_settings, &id, &yes).await?;
            }
        },

        Commands::Books(books_cmd) => match books_cmd {
            BooksCommands::Add {
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
                handlers::books::handle_books_add(
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

            BooksCommands::AddBatch {
                input,
                continue_on_error,
                dry_run,
            } => {
                handlers::books::handle_books_add_batch(
                    &cli.library,
                    &app_settings,
                    input,
                    continue_on_error,
                    dry_run,
                )
                .await?;
            }

            BooksCommands::List { filters, output } => {
                handlers::books::handle_books_list(&cli.library, &app_settings, filters, output)
                    .await?;
            }

            BooksCommands::Update {
                selector,
                yes,
                dry_run,
            } => {
                handlers::books::handle_books_update(
                    &cli.library,
                    &app_settings,
                    selector,
                    yes,
                    dry_run,
                )
                .await?;
            }

            BooksCommands::Delete {
                id,
                filters,
                delete_file,
                force,
                yes,
                dry_run,
            } => {
                handlers::books::handle_books_delete(
                    &cli.library,
                    &app_settings,
                    id,
                    filters,
                    delete_file,
                    force,
                    yes,
                    dry_run,
                )
                .await?;
            }

            BooksCommands::Cleanup { dry_run } => {
                handlers::books::handle_books_cleanup(&cli.library, &app_settings, dry_run)
                    .await?;
            }
        },

        Commands::Contents(contents_cmd) => match contents_cmd {
            ContentsCommands::Add {
                title,
                original_title,
                people,
                content_type,
                year,
                notes,
                pages,
                tags,
                languages,
                book_id,
            } => {
                handlers::contents::handle_contents_add(
                    &cli.library,
                    &app_settings,
                    title,
                    original_title,
                    people,
                    content_type,
                    year,
                    notes,
                    pages,
                    tags,
                    languages,
                    book_id,
                )
                .await?;
            }

            ContentsCommands::List { filters, output } => {
                handlers::contents::handle_contents_list(&cli.library, &app_settings, filters, output)
                    .await?;
            }

            ContentsCommands::Update {
                selector,
                yes,
                dry_run,
            } => {
                handlers::contents::handle_contents_update(
                    &cli.library,
                    &app_settings,
                    selector,
                    yes,
                    dry_run,
                )
                .await?;
            }

            ContentsCommands::Delete {
                id,
                filters,
                yes,
                dry_run,
            } => {
                handlers::contents::handle_contents_delete(
                    &cli.library,
                    &app_settings,
                    id,
                    filters,
                    yes,
                    dry_run,
                )
                .await?;
            }

            ContentsCommands::Link {
                content_id,
                book_id,
            } => {
                handlers::contents::handle_contents_link(
                    &cli.library,
                    &app_settings,
                    content_id,
                    book_id,
                )
                .await?;
            }

            ContentsCommands::Unlink {
                content_id,
                book_id,
            } => {
                handlers::contents::handle_contents_unlink(
                    &cli.library,
                    &app_settings,
                    content_id,
                    book_id,
                )
                .await?;
            }
        },
    }

    Ok(())
}
