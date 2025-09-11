#![allow(unused)]

// ritmo_errors/src/lib.rs
pub type RitmoResult<T> = Result<T, RitmoErr>;

use std::fmt;
use sqlx::Error as SqlxError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RitmoErr {

    #[error("Errore durante la creazione del database: {0}")]
    DatabaseCreation(String),
    
    #[error("Errore di connessione al database: {0}")]
    DatabaseConnection(String),
    
    #[error("Database non trovato: {0}")]
    DatabaseNotFound(String),
    
    #[error("Errore nella query al database: {0}")]
    DatabaseQuery(String),
    
    #[error("Errore nella migrazione del database: {0}")]
    DatabaseMigration(String),
    
    #[error("Errore generico: {0}")]
    Generic(String),
    


    #[error("Migration failed: {0}")]
    DatabaseMigrationFailed(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("sqlx error: {0}")]
    SqlxError(sqlx::Error),
    #[error("Database connection failed: {0}")]
    DatabaseConnectionFailed(String),
    #[error("Database query failed: {0}")]
    DatabaseQueryFailed(String),
    #[error("Database insert failed: {0}")]
    DatabaseInsertFailed(String),
    #[error("Database delete failed: {0}")]
    DatabaseDeleteFailed(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("DB   failed: {0}")]
    DatabaseTransactionError(String),
    #[error("File access failed: {0}")]
    FileAccessError(#[from] std::io::Error),
    #[error("No result found: {0}")]
    NoResultsError(String),
    #[error("Database integrity fail : {0}")]
    DataIntegrityError(String),
    #[error("Import error: {0}")]
    ImportError(String),
    #[error("Export error: {0}")]
    ExportError(String),
    #[error("Unknown error: {0}")]
    UnknownError(String),
    #[error("Path error: {0}")]
    PathError(String),
    #[error("Database creation fail: {0}")]
    DatabaseCreationFailed(String),
    #[error("Other error: {0}")]
    OtherError(String),
    #[error("Invalid table name: {0}")]
    InvalidTableName(String),
    #[error("Invalid column name: {0}")]
    InvalidColumnName(String),
    #[error("Record not found")]
    RecordNotFound,
    #[error("Search and add operation failed: {0}")]
    SearchAndAddFailed(String),
    #[error("Search and add invalid input: {0}")]
    InvalidInput(String),
    #[error("  commit failed: {0}")]
    CommitFailed(String),
    #[error("Name parsing error: {0}")]
    NameParsingError(String),
    #[error("Name merging error: {0}")]
    MergeError(String),
    #[error("Errore di Machine Learning: {0}")]
    MLError(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
}

impl From<sqlx::Error> for RitmoErr {
    fn from(err: sqlx::Error) -> Self {
        RitmoErr::DatabaseError(format!("Database operation failed: {}", err))
    }
}

impl From<serde_json::Error> for RitmoErr {
    fn from(err: serde_json::Error) -> Self {
        RitmoErr::MLError(format!("Serialization/Deserialization error: {}", err))
    }
}
