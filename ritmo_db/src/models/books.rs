use sha2::Digest;
use chrono::Utc;
use ritmo_core::dto::BookDto;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Default)]
pub struct Book {
    /// Il campo 'id' è Option perchè quando il libro viene creato il suo valore è None, e viene creato alla memorizzazione.
    pub id: Option<i64>,
    pub name: String,
    pub original_title: Option<String>,
    pub publisher_id: Option<i64>,
    pub format_id: Option<i64>,
    pub series_id: Option<i64>,
    pub series_index: Option<i64>,
    pub publication_date: Option<i64>,
    pub last_modified_date: i64,
    pub isbn: Option<String>,
    pub pages: Option<i64>,
    pub notes: Option<String>,
    pub has_cover: i64,
    pub has_paper: i64,
    pub file_link: Option<String>,
    pub file_size: Option<i64>,
    pub file_hash: Option<String>,
    pub created_at: i64,
}

impl Book {
    // Metodo per la conversione da DTO al modello
    pub fn from_dto(dto: &mut BookDto) -> Self {
        let now = Utc::now().timestamp();

        let mut book = Self {
            name: dto.name.clone(),
            original_title: dto.original_title.clone(),
            publisher_id: dto.publisher_id,
            format_id: dto.format_id,
            series_id: dto.series_id,
            series_index: dto.series_index,
            publication_date: dto.publication_date,
            last_modified_date: now,
            isbn: dto.isbn.clone(),
            notes: dto.notes.clone(),
            has_paper: if dto.has_paper { 1 } else { 0 },
            has_cover: if dto.has_cover { 1 } else { 0 },
            created_at: now,
            ..Default::default()
        };
        book.set_book_persistence();
        book
    }

    pub async fn save(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query!(
            "INSERT INTO books (
                name, original_title, publisher_id, format_id, series_id, series_index,
                publication_date, last_modified_date, isbn, notes,
                has_cover, has_paper, file_link, file_size, file_hash, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            self.name,
            self.original_title,
            self.publisher_id,
            self.format_id,
            self.series_id,
            self.series_index,
            self.publication_date,
            now,
            self.isbn,
            self.notes,
            self.has_cover,
            self.has_paper,
            self.file_link,
            self.file_size,
            self.file_hash,
            now
            )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Book>, sqlx::Error> {
        let book = sqlx::query_as!(
            Book,
            "SELECT * FROM books WHERE id = ?",
            id
            )
            .fetch_optional(pool)
            .await?;
        Ok(book)
    }

    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM books WHERE id = ?",
            id
            )
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_all(pool: &sqlx::SqlitePool) -> Result<Vec<Book>, sqlx::Error> {
        let all = sqlx::query_as!(
            Book,
            "SELECT * FROM books ORDER BY name"
            )
            .fetch_all(pool)
            .await?;
        Ok(all)
    }

    pub async fn search(pool: &sqlx::SqlitePool, pattern: &str) -> Result<Vec<Book>, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let found = sqlx::query_as!(
            Book,
            "SELECT * FROM books WHERE name LIKE ? OR original_title LIKE ? OR notes LIKE ? OR isbn LIKE ? ORDER BY name",
            search_pattern,
            search_pattern,
            search_pattern,
            search_pattern
            )
        .fetch_all(pool)
        .await?;
        Ok(found)
    }

    pub fn set_book_persistence(&mut self) {
        // Generiamo un hash basato sui metadati del libro
        let mut hasher = sha2::Sha256::new();
        
        // Aggiungiamo i metadati essenziali per generare un hash unico
        hasher.update(self.name.as_bytes());
        
        if let Some(ref title) = self.original_title {
            hasher.update(title.as_bytes());
        }
        
        if let Some(ref isbn) = self.isbn {
            hasher.update(isbn.as_bytes());
        }
        
        // Aggiungiamo la data di creazione per ulteriore unicità
        hasher.update(self.created_at.to_be_bytes());
        
        // Per libri di una serie, aggiungiamo anche queste informazioni
        if let Some(series_id) = self.series_id {
            hasher.update(series_id.to_be_bytes());
            
            if let Some(index) = self.series_index {
                hasher.update(index.to_be_bytes());
            }
        }
        
        // Generiamo il hash completo
        let hash_result = hasher.finalize();
        
        // Prendiamo i primi 16 byte dell'hash (128 bit) per un identificativo conciso ma unico
        let hash_bytes = &hash_result[..16];
        let hash_hex = hex::encode(hash_bytes);
        
        // Memorizziamo l'hash generato
        self.file_hash = Some(hash_hex.clone());
        
        // Generiamo un percorso file standardizzato basato sull'hash
        // Assumiamo come formato default "epub" per i nuovi libri
        let file_path = format!("books/{}/{}/{}.epub", 
                               &hash_hex[0..2],     // Primi 2 caratteri per la prima directory
                               &hash_hex[2..4],     // Successivi 2 caratteri per la sottodirectory
                               hash_hex);           // Nome file basato sull'hash completo
        
        // Impostiamo il link al file
        self.file_link = Some(file_path);
        
        // Per ora il file size è 0 perché il file non è stato ancora effettivamente creato
        self.file_size = Some(0);
    }
}
