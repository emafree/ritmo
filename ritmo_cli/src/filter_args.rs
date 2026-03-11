use clap::Args;
use ritmo_db_core::filters::{BookFilters, BookSortField, ContentFilters, ContentSortField};

/// Shared filter arguments for book operations
#[derive(Args, Clone, Debug)]
pub struct BookFilterArgs {
    /// Usa un preset salvato
    #[arg(long, short = 'p')]
    pub preset: Option<String>,

    /// Filtra per autore
    #[arg(long)]
    pub author: Option<String>,

    /// Filtra per editore
    #[arg(long)]
    pub publisher: Option<String>,

    /// Filtra per serie
    #[arg(long)]
    pub series: Option<String>,

    /// Filtra per formato (epub, pdf, mobi, etc.)
    #[arg(long)]
    pub format: Option<String>,

    /// Filtra per anno di pubblicazione
    #[arg(long)]
    pub year: Option<i32>,

    /// Filtra per ISBN
    #[arg(long)]
    pub isbn: Option<String>,

    /// Ricerca full-text (titolo, autori, note)
    #[arg(long, short)]
    pub search: Option<String>,

    /// Filtra libri acquisiti dopo questa data (YYYY-MM-DD)
    #[arg(long)]
    pub acquired_after: Option<String>,

    /// Filtra libri acquisiti prima di questa data (YYYY-MM-DD)
    #[arg(long)]
    pub acquired_before: Option<String>,

    /// Filtra libri acquisiti negli ultimi N giorni
    #[arg(long, conflicts_with = "acquired_after")]
    pub last_days: Option<i64>,

    /// Filtra libri acquisiti negli ultimi N mesi
    #[arg(long, conflicts_with = "acquired_after")]
    pub last_months: Option<i64>,

    /// Limita ai primi N libri acquisiti più recentemente
    #[arg(long)]
    pub recent_count: Option<i64>,

    /// Ordina per campo (title, author, year, date_added)
    #[arg(long, default_value = "title")]
    pub sort: String,

    /// Limita numero risultati
    #[arg(long)]
    pub limit: Option<i64>,

    /// Offset risultati (per paginazione)
    #[arg(long, default_value = "0")]
    pub offset: i64,
}

impl BookFilterArgs {
    /// Convert CLI args to BookFilters for use with query execution
    pub fn to_filters(&self) -> BookFilters {
        let mut filters = BookFilters::default();

        // Use helper methods for adding single values
        if let Some(author) = &self.author {
            filters = filters.with_author(author.clone());
        }
        if let Some(publisher) = &self.publisher {
            filters = filters.with_publisher(publisher.clone());
        }
        if let Some(series) = &self.series {
            filters = filters.with_series(series.clone());
        }
        if let Some(format) = &self.format {
            filters = filters.with_format(format.clone());
        }

        // Set other fields directly
        filters.year = self.year;
        filters.isbn = self.isbn.clone();
        filters.search = self.search.clone();
        filters.acquired_after = self.acquired_after.as_ref().and_then(|s| s.parse().ok());
        filters.acquired_before = self.acquired_before.as_ref().and_then(|s| s.parse().ok());
        filters.limit = self.limit;
        filters.offset = self.offset;
        filters.sort = BookSortField::from_str(&self.sort);

        // Handle relative date filters (last_days, last_months)
        if let Some(days) = self.last_days {
            use chrono::{Duration, Utc};
            let timestamp = (Utc::now() - Duration::days(days)).timestamp();
            filters.acquired_after = Some(timestamp);
        }
        if let Some(months) = self.last_months {
            use chrono::{Duration, Utc};
            let timestamp = (Utc::now() - Duration::days(months * 30)).timestamp();
            filters.acquired_after = Some(timestamp);
        }

        // Handle recent_count (implicitly sets sort to date_added DESC and limit)
        if let Some(count) = self.recent_count {
            filters.sort = BookSortField::DateAdded;
            filters.limit = Some(count);
        }

        filters
    }
}

/// Book selector: either by ID or by filters
/// This allows commands to accept --id for single-item operations
/// or filters for bulk operations
#[derive(Args, Clone, Debug)]
pub struct BookSelector {
    /// Select by book ID (for single-item operations)
    #[arg(long, conflicts_with_all = ["author", "publisher", "series", "format", "year", "isbn", "search", "acquired_after", "acquired_before", "last_days", "last_months", "recent_count"])]
    pub id: Option<i64>,

    /// Filter arguments (for bulk operations)
    #[command(flatten)]
    pub filters: BookFilterArgs,
}

impl BookSelector {
    /// Check if this selector uses ID (single-item mode)
    pub fn is_single_item(&self) -> bool {
        self.id.is_some()
    }

    /// Check if this selector uses filters (bulk mode)
    pub fn is_bulk(&self) -> bool {
        self.id.is_none()
    }
}

/// Metadata update arguments for books
#[derive(Args, Clone, Debug)]
pub struct BookUpdateArgs {
    /// Nuovo titolo
    #[arg(long)]
    pub title: Option<String>,

    /// Nuovo titolo originale
    #[arg(long)]
    pub original_title: Option<String>,

    /// Nuove persone con ruoli (formato: "Nome:Ruolo", sostituisce tutte le persone esistenti)
    #[arg(long)]
    pub people: Vec<String>,

    /// Nuovo editore
    #[arg(long)]
    pub publisher: Option<String>,

    /// Nuovo anno di pubblicazione
    #[arg(long)]
    pub year: Option<i32>,

    /// Nuovo ISBN
    #[arg(long)]
    pub isbn: Option<String>,

    /// Nuovo formato
    #[arg(long)]
    pub format: Option<String>,

    /// Nuova serie
    #[arg(long)]
    pub series: Option<String>,

    /// Nuovo indice nella serie
    #[arg(long)]
    pub series_index: Option<i64>,

    /// Nuove note
    #[arg(long)]
    pub notes: Option<String>,

    /// Nuovi tags (sostituiscono tutti i tags esistenti)
    #[arg(long)]
    pub tags: Vec<String>,
}

impl BookUpdateArgs {
    /// Check if any update fields are specified
    pub fn has_updates(&self) -> bool {
        self.title.is_some()
            || self.original_title.is_some()
            || !self.people.is_empty()
            || self.publisher.is_some()
            || self.year.is_some()
            || self.isbn.is_some()
            || self.format.is_some()
            || self.series.is_some()
            || self.series_index.is_some()
            || self.notes.is_some()
            || !self.tags.is_empty()
    }
}

/// Book update selector for bulk operations: combines filters (to select books) and update values (to modify them)
/// Uses prefixed arguments to avoid conflicts: --filter-* for selection, --set-* for updates
#[derive(Args, Clone, Debug)]
pub struct BookBulkUpdateSelector {
    /// Select by ID (mutually exclusive with filters)
    #[arg(long, conflicts_with_all = ["filter_author", "filter_publisher", "filter_series", "filter_format", "filter_year", "filter_isbn", "filter_search"])]
    pub id: Option<i64>,

    // FILTER ARGUMENTS (for selecting books to update)
    /// Filtra per autore
    #[arg(long = "filter-author")]
    pub filter_author: Option<String>,

    /// Filtra per editore
    #[arg(long = "filter-publisher")]
    pub filter_publisher: Option<String>,

    /// Filtra per serie
    #[arg(long = "filter-series")]
    pub filter_series: Option<String>,

    /// Filtra per formato
    #[arg(long = "filter-format")]
    pub filter_format: Option<String>,

    /// Filtra per anno
    #[arg(long = "filter-year")]
    pub filter_year: Option<i32>,

    /// Filtra per ISBN
    #[arg(long = "filter-isbn")]
    pub filter_isbn: Option<String>,

    /// Ricerca full-text
    #[arg(long = "filter-search")]
    pub filter_search: Option<String>,

    // UPDATE VALUES (new values to set)
    /// Nuovo titolo
    #[arg(long = "set-title")]
    pub set_title: Option<String>,

    /// Nuovo titolo originale
    #[arg(long = "set-original-title")]
    pub set_original_title: Option<String>,

    /// Nuove persone con ruoli (formato: "Nome:Ruolo")
    #[arg(long = "set-people")]
    pub set_people: Vec<String>,

    /// Nuovo editore
    #[arg(long = "set-publisher")]
    pub set_publisher: Option<String>,

    /// Nuovo anno
    #[arg(long = "set-year")]
    pub set_year: Option<i32>,

    /// Nuovo ISBN
    #[arg(long = "set-isbn")]
    pub set_isbn: Option<String>,

    /// Nuovo formato
    #[arg(long = "set-format")]
    pub set_format: Option<String>,

    /// Nuova serie
    #[arg(long = "set-series")]
    pub set_series: Option<String>,

    /// Nuovo indice serie
    #[arg(long = "set-series-index")]
    pub set_series_index: Option<i64>,

    /// Nuove note
    #[arg(long = "set-notes")]
    pub set_notes: Option<String>,

    /// Nuovi tags
    #[arg(long = "set-tags")]
    pub set_tags: Vec<String>,
}

impl BookBulkUpdateSelector {
    /// Check if using ID mode (single book)
    pub fn is_id_mode(&self) -> bool {
        self.id.is_some()
    }

    /// Check if any filters are specified
    pub fn has_filters(&self) -> bool {
        self.filter_author.is_some()
            || self.filter_publisher.is_some()
            || self.filter_series.is_some()
            || self.filter_format.is_some()
            || self.filter_year.is_some()
            || self.filter_isbn.is_some()
            || self.filter_search.is_some()
    }

    /// Check if any update values are specified
    pub fn has_updates(&self) -> bool {
        self.set_title.is_some()
            || self.set_original_title.is_some()
            || !self.set_people.is_empty()
            || self.set_publisher.is_some()
            || self.set_year.is_some()
            || self.set_isbn.is_some()
            || self.set_format.is_some()
            || self.set_series.is_some()
            || self.set_series_index.is_some()
            || self.set_notes.is_some()
            || !self.set_tags.is_empty()
    }

    /// Convert filter args to BookFilters
    pub fn to_filters(&self) -> BookFilters {
        let mut filters = BookFilters::default();

        if let Some(author) = &self.filter_author {
            filters = filters.with_author(author.clone());
        }
        if let Some(publisher) = &self.filter_publisher {
            filters = filters.with_publisher(publisher.clone());
        }
        if let Some(series) = &self.filter_series {
            filters = filters.with_series(series.clone());
        }
        if let Some(format) = &self.filter_format {
            filters = filters.with_format(format.clone());
        }

        filters.year = self.filter_year;
        filters.isbn = self.filter_isbn.clone();
        filters.search = self.filter_search.clone();

        filters
    }
}

/// Shared filter arguments for content operations
#[derive(Args, Clone, Debug)]
pub struct ContentFilterArgs {
    /// Usa un preset salvato
    #[arg(long, short = 'p')]
    pub preset: Option<String>,

    /// Filtra per autore del contenuto
    #[arg(long)]
    pub author: Option<String>,

    /// Filtra per tipo (Romanzo, Racconto, Saggio, etc.)
    #[arg(long)]
    pub content_type: Option<String>,

    /// Filtra per genere
    #[arg(long)]
    pub genre: Option<String>,

    /// Filtra per anno di pubblicazione
    #[arg(long)]
    pub year: Option<i32>,

    /// Ricerca full-text (titolo, autori, note)
    #[arg(long, short)]
    pub search: Option<String>,

    /// Ordina per campo (title, author, year, type)
    #[arg(long, default_value = "title")]
    pub sort: String,

    /// Limita numero risultati
    #[arg(long)]
    pub limit: Option<i64>,

    /// Offset risultati (per paginazione)
    #[arg(long, default_value = "0")]
    pub offset: i64,
}

impl ContentFilterArgs {
    /// Convert CLI args to ContentFilters for use with query execution
    pub fn to_filters(&self) -> ContentFilters {
        let mut filters = ContentFilters::default();

        // Use helper methods for adding single values
        if let Some(author) = &self.author {
            filters = filters.with_author(author.clone());
        }
        if let Some(content_type) = &self.content_type {
            filters = filters.with_content_type(content_type.clone());
        }
        if let Some(genre) = &self.genre {
            filters = filters.with_genre(genre.clone());
        }

        // Set other fields directly
        filters.year = self.year;
        filters.search = self.search.clone();
        filters.limit = self.limit;
        filters.offset = self.offset;
        filters.sort = ContentSortField::from_str(&self.sort);

        filters
    }
}

/// Content selector: either by ID or by filters
#[derive(Args, Clone, Debug)]
pub struct ContentSelector {
    /// Select by content ID (for single-item operations)
    #[arg(long, conflicts_with_all = ["author", "content_type", "genre", "year", "search"])]
    pub id: Option<i64>,

    /// Filter arguments (for bulk operations)
    #[command(flatten)]
    pub filters: ContentFilterArgs,
}

impl ContentSelector {
    /// Check if this selector uses ID (single-item mode)
    pub fn is_single_item(&self) -> bool {
        self.id.is_some()
    }

    /// Check if this selector uses filters (bulk mode)
    pub fn is_bulk(&self) -> bool {
        self.id.is_none()
    }
}

/// Metadata update arguments for contents
#[derive(Args, Clone, Debug)]
pub struct ContentUpdateArgs {
    /// Nuovo titolo
    #[arg(long)]
    pub title: Option<String>,

    /// Nuovo titolo originale
    #[arg(long)]
    pub original_title: Option<String>,

    /// Nuove persone con ruoli (formato: "Nome:Ruolo", sostituisce tutte le persone esistenti)
    #[arg(long)]
    pub people: Vec<String>,

    /// Nuovo tipo di contenuto
    #[arg(long)]
    pub content_type: Option<String>,

    /// Nuovo genere
    #[arg(long)]
    pub genre: Option<String>,

    /// Nuovo anno di pubblicazione
    #[arg(long)]
    pub year: Option<i32>,

    /// Nuove note
    #[arg(long)]
    pub notes: Option<String>,

    /// Nuovi tags (sostituiscono tutti i tags esistenti)
    #[arg(long)]
    pub tags: Vec<String>,

    /// Nuove languages (formato: "Nome:iso2:iso3:role")
    #[arg(long)]
    pub languages: Vec<String>,
}

impl ContentUpdateArgs {
    /// Check if any update fields are specified
    pub fn has_updates(&self) -> bool {
        self.title.is_some()
            || self.original_title.is_some()
            || !self.people.is_empty()
            || self.content_type.is_some()
            || self.genre.is_some()
            || self.year.is_some()
            || self.notes.is_some()
            || !self.tags.is_empty()
            || !self.languages.is_empty()
    }
}

/// Content update selector for bulk operations: combines filters (to select contents) and update values (to modify them)
/// Uses prefixed arguments to avoid conflicts: --filter-* for selection, --set-* for updates
#[derive(Args, Clone, Debug)]
pub struct ContentBulkUpdateSelector {
    /// Select by ID (mutually exclusive with filters)
    #[arg(long, conflicts_with_all = ["filter_author", "filter_content_type", "filter_genre", "filter_year", "filter_search"])]
    pub id: Option<i64>,

    // FILTER ARGUMENTS (for selecting contents to update)
    /// Filtra per autore
    #[arg(long = "filter-author")]
    pub filter_author: Option<String>,

    /// Filtra per tipo di contenuto
    #[arg(long = "filter-content-type")]
    pub filter_content_type: Option<String>,

    /// Filtra per genere
    #[arg(long = "filter-genre")]
    pub filter_genre: Option<String>,

    /// Filtra per anno
    #[arg(long = "filter-year")]
    pub filter_year: Option<i32>,

    /// Ricerca full-text
    #[arg(long = "filter-search")]
    pub filter_search: Option<String>,

    // UPDATE VALUES (new values to set)
    /// Nuovo titolo
    #[arg(long = "set-title")]
    pub set_title: Option<String>,

    /// Nuovo titolo originale
    #[arg(long = "set-original-title")]
    pub set_original_title: Option<String>,

    /// Nuove persone con ruoli (formato: "Nome:Ruolo")
    #[arg(long = "set-people")]
    pub set_people: Vec<String>,

    /// Nuovo tipo di contenuto
    #[arg(long = "set-content-type")]
    pub set_content_type: Option<String>,

    /// Nuovo genere
    #[arg(long = "set-genre")]
    pub set_genre: Option<String>,

    /// Nuovo anno
    #[arg(long = "set-year")]
    pub set_year: Option<i32>,

    /// Nuove note
    #[arg(long = "set-notes")]
    pub set_notes: Option<String>,

    /// Nuovi tags
    #[arg(long = "set-tags")]
    pub set_tags: Vec<String>,

    /// Nuove lingue (formato: "it:original" o "en:actual")
    #[arg(long = "set-languages")]
    pub set_languages: Vec<String>,
}

impl ContentBulkUpdateSelector {
    /// Check if using ID mode (single content)
    pub fn is_id_mode(&self) -> bool {
        self.id.is_some()
    }

    /// Check if any filters are specified
    pub fn has_filters(&self) -> bool {
        self.filter_author.is_some()
            || self.filter_content_type.is_some()
            || self.filter_genre.is_some()
            || self.filter_year.is_some()
            || self.filter_search.is_some()
    }

    /// Check if any update values are specified
    pub fn has_updates(&self) -> bool {
        self.set_title.is_some()
            || self.set_original_title.is_some()
            || !self.set_people.is_empty()
            || self.set_content_type.is_some()
            || self.set_genre.is_some()
            || self.set_year.is_some()
            || self.set_notes.is_some()
            || !self.set_tags.is_empty()
            || !self.set_languages.is_empty()
    }

    /// Convert filter args to ContentFilters
    pub fn to_filters(&self) -> ContentFilters {
        let mut filters = ContentFilters::default();

        if let Some(author) = &self.filter_author {
            filters = filters.with_author(author.clone());
        }
        if let Some(content_type) = &self.filter_content_type {
            filters = filters.with_content_type(content_type.clone());
        }
        if let Some(genre) = &self.filter_genre {
            filters = filters.with_genre(genre.clone());
        }

        filters.year = self.filter_year;
        filters.search = self.filter_search.clone();

        filters
    }
}

/// Combined filter arguments for presets (supports both book and content filters)
#[derive(Args, Clone, Debug)]
pub struct PresetFilterArgs {
    // Book filters
    #[arg(long)]
    pub author: Option<String>,

    #[arg(long)]
    pub publisher: Option<String>,

    #[arg(long)]
    pub series: Option<String>,

    #[arg(long)]
    pub format: Option<String>,

    #[arg(long)]
    pub year: Option<i32>,

    #[arg(long)]
    pub isbn: Option<String>,

    #[arg(long)]
    pub search: Option<String>,

    #[arg(long)]
    pub acquired_after: Option<String>,

    #[arg(long)]
    pub acquired_before: Option<String>,

    // Content-specific filter
    #[arg(long)]
    pub content_type: Option<String>,

    #[arg(long, default_value = "title")]
    pub sort: String,

    #[arg(long)]
    pub limit: Option<i64>,

    #[arg(long, default_value = "0")]
    pub offset: i64,
}
