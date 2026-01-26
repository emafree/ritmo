//! Trait for i18n support in database models
//!
//! This trait provides a consistent interface for translating
//! canonical keys (e.g., "role.author", "language_role.original")
//! into localized display strings based on the current UI language.
//!
//! # Examples
//!
//! ```
//! use ritmo_db::{Role, i18n_trait::I18nDisplayable};
//! use ritmo_db::i18n_utils::set_locale;
//!
//! let role = Role {
//!     id: Some(1),
//!     key: "role.author".to_string(),
//!     created_at: 1234567890,
//! };
//!
//! set_locale("en");
//! assert_eq!(role.translate(), "Author");
//!
//! set_locale("it");
//! assert_eq!(role.translate(), "Autore");
//! ```

/// Trait for models that have i18n-translatable keys
///
/// This trait provides a unified interface for models that store
/// canonical keys (like "role.author" or "language_role.original")
/// and need to translate them to localized strings.
pub trait I18nDisplayable {
    /// Returns the canonical i18n key for this model instance
    ///
    /// Examples:
    /// - Role: "role.author"
    /// - RunningLanguages: "language_role.original"
    fn i18n_key(&self) -> &str;

    /// Returns the namespace prefix for translation keys
    ///
    /// Default is "db", which means keys are mapped as:
    /// - "role.author" -> "db.role.author"
    /// - "language_role.original" -> "db.language_role.original"
    fn i18n_namespace(&self) -> &str {
        "db"
    }

    /// Translates the i18n key to a localized string
    ///
    /// Uses the rust-i18n `t!()` macro to look up the translation
    /// for the key in the current locale.
    ///
    /// # Returns
    /// The translated string in the current locale, or the key itself
    /// if no translation is found.
    fn translate(&self) -> String {
        use rust_i18n::t;

        // Build full translation key: "db.role.author"
        let translation_key = format!("{}.{}", self.i18n_namespace(), self.i18n_key());
        t!(&translation_key).to_string()
    }
}
