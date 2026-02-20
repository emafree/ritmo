use ritmo_commands::books::ListBooksCommand;
use ritmo_commands::contents::ListContentsCommand;
use ritmo_commands::{Command, BookSummary, ContentSummary};
use ritmo_db_core::{LibraryConfig, filters::{BookFilters, ContentFilters}};
use ritmo_errors::reporter::SilentReporter;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Manages library data and database connection
pub struct LibraryState {
    config: LibraryConfig,
    pool: Option<Arc<SqlitePool>>,
    runtime: tokio::runtime::Runtime,
    
    // Cached data
    books: Vec<BookSummary>,
    contents: Vec<ContentSummary>,
}

impl LibraryState {
    /// Create a new library state
    pub fn new(library_path: std::path::PathBuf) -> anyhow::Result<Self> {
        let config = LibraryConfig::new(&library_path);
        let runtime = tokio::runtime::Runtime::new()?;
        
        Ok(Self {
            config,
            pool: None,
            runtime,
            books: Vec::new(),
            contents: Vec::new(),
        })
    }
    
    /// Initialize the library and database connection
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        // Initialize library structure
        if !self.config.exists() {
            self.config.initialize()?;
        }
        
        // Create database pool
        let mut reporter = SilentReporter;
        let pool = self.runtime.block_on(async {
            self.config.initialize_database().await?;
            self.config.create_pool(&mut reporter).await
        })?;
        
        self.pool = Some(Arc::new(pool));
        Ok(())
    }
    
    /// Get the library root directory path
    pub fn library_root(&self) -> &std::path::Path {
        &self.config.root_path
    }
    
    /// Get books with current filters
    pub fn get_books(&self) -> &[BookSummary] {
        &self.books
    }
    
    /// Get contents with current filters
    pub fn get_contents(&self) -> &[ContentSummary] {
        &self.contents
    }
    
    /// Refresh books list with filters
    pub fn refresh_books(&mut self, filters: BookFilters) -> anyhow::Result<()> {
        let pool = self.pool.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;
        
        let command = ListBooksCommand;
        let input = ritmo_commands::books::ListBooksInput { filters };
        
        let result = self.runtime.block_on(async {
            command.execute(&self.config, pool, input).await
        })?;
        
        self.books = result.books;
        Ok(())
    }
    
    /// Refresh contents list with filters
    pub fn refresh_contents(&mut self, filters: ContentFilters) -> anyhow::Result<()> {
        let pool = self.pool.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;
        
        let command = ListContentsCommand;
        let input = ritmo_commands::contents::ListContentsInput { filters };
        
        let result = self.runtime.block_on(async {
            command.execute(&self.config, pool, input).await
        })?;
        
        self.contents = result.contents;
        Ok(())
    }
    
    /// Get available authors for filtering (books)
    pub fn get_book_authors(&self) -> Vec<String> {
        // For now, extract unique authors from loaded books
        let mut authors: Vec<String> = self.books
            .iter()
            .flat_map(|b| b.authors.clone())
            .collect();
        authors.sort();
        authors.dedup();
        authors
    }
    
    /// Get available publishers for filtering
    pub fn get_publishers(&self) -> Vec<String> {
        let mut publishers: Vec<String> = self.books
            .iter()
            .filter_map(|b| b.publisher.clone())
            .collect();
        publishers.sort();
        publishers.dedup();
        publishers
    }
    
    /// Get available formats for filtering
    pub fn get_formats(&self) -> Vec<String> {
        let mut formats: Vec<String> = self.books
            .iter()
            .filter_map(|b| b.format.clone())
            .collect();
        formats.sort();
        formats.dedup();
        formats
    }
    
    /// Get available content types for filtering
    pub fn get_content_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self.contents
            .iter()
            .filter_map(|c| c.content_type.clone())
            .collect();
        types.sort();
        types.dedup();
        types
    }
    
    /// Get available authors for contents
    pub fn get_content_authors(&self) -> Vec<String> {
        let mut authors: Vec<String> = self.contents
            .iter()
            .flat_map(|c| c.authors.clone())
            .collect();
        authors.sort();
        authors.dedup();
        authors
    }

    /// Fetch the contents of a specific book with associated people.
    /// Intended to be called only when selection changes (not every frame).
    pub fn get_book_contents(&self, book_id: i64) -> anyhow::Result<Vec<ritmo_db::gui_queries::ContentWithPeople>> {
        let pool = self.pool.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;
        let result = self.runtime.block_on(async {
            ritmo_db::gui_queries::get_book_contents_by_id(pool, book_id).await
        })?;
        Ok(result)
    }

    /// Fetch the books that contain a specific content with associated authors.
    /// Intended to be called only when selection changes (not every frame).
    pub fn get_content_books(&self, content_id: i64) -> anyhow::Result<Vec<ritmo_db::gui_queries::BookBasicInfo>> {
        let pool = self.pool.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;
        let result = self.runtime.block_on(async {
            ritmo_db::gui_queries::get_content_books_by_id(pool, content_id).await
        })?;
        Ok(result)
    }
}
