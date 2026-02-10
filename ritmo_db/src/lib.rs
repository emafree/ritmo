// ritmo_db/src/lib.rs

pub mod crud_trait;

pub use crud_trait::{
    crud_delete, crud_get, crud_list_all, crud_search, get_or_create, CrudModel, GetOrCreateModel,
};

// Initialize i18n
rust_i18n::i18n!("../locales", fallback = "en");

pub mod error_i18n;
pub mod gui_queries;
pub mod i18n_trait;
pub mod i18n_utils;
pub mod models;

// Re-export delle funzioni più comuni per comodità
pub use gui_queries::*;
pub use models::*;
