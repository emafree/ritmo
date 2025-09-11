// ritmo_db/src/lib.rs
pub mod models;

// Re-export delle funzioni più comuni per comodità
pub use models::*;
pub use ritmo_db_core::Database;

