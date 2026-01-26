// ritmo_db/src/lib.rs

// Initialize i18n
rust_i18n::i18n!("../locales", fallback = "en");

pub mod error_i18n;
pub mod i18n_trait;
pub mod i18n_utils;
pub mod models;

// Re-export delle funzioni più comuni per comodità
pub use models::*;
