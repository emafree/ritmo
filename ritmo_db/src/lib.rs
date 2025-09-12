// ritmo_db/src/lib.rs
pub mod models;
pub const DB_TEMPLATE: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/template.db"));

// Re-export delle funzioni più comuni per comodità
pub use models::*;
pub use ritmo_db_core::Database;
