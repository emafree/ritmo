//! i18n support for RitmoErr
//!
//! This module provides localization support for error messages.
//! It extends RitmoErr with a `localized_message()` method that
//! returns translated error messages based on the current locale.

use ritmo_errors::RitmoErr;
use rust_i18n::t;

/// Trait to add i18n support to error types
pub trait LocalizableError {
    /// Returns a localized error message based on the current locale
    fn localized_message(&self) -> String;
}

impl LocalizableError for RitmoErr {
    fn localized_message(&self) -> String {
        match self {
            // Database errors
            RitmoErr::DatabaseCreation(err) => {
                t!("error.database.creation", error = err).to_string()
            }
            RitmoErr::DatabaseConnection(err) => {
                t!("error.database.connection", error = err).to_string()
            }
            RitmoErr::DatabaseNotFound(path) => {
                t!("error.database.not_found", path = path).to_string()
            }
            RitmoErr::DatabaseQuery(err) => {
                t!("error.database.query", error = err).to_string()
            }
            RitmoErr::DatabaseMigration(err) => {
                t!("error.database.migration", error = err).to_string()
            }
            RitmoErr::DatabaseMigrationFailed(err) => {
                t!("error.database.migration_failed", error = err).to_string()
            }
            RitmoErr::DatabaseConnectionFailed(err) => {
                t!("error.database.connection_failed", error = err).to_string()
            }
            RitmoErr::DatabaseQueryFailed(err) => {
                t!("error.database.query_failed", error = err).to_string()
            }
            RitmoErr::DatabaseInsertFailed(err) => {
                t!("error.database.insert_failed", error = err).to_string()
            }
            RitmoErr::DatabaseDeleteFailed(err) => {
                t!("error.database.delete_failed", error = err).to_string()
            }
            RitmoErr::DatabaseError(err) => {
                t!("error.database.error", error = err).to_string()
            }
            RitmoErr::DatabaseTransactionError(err) => {
                t!("error.database.transaction_error", error = err).to_string()
            }
            RitmoErr::DatabaseCreationFailed(err) => {
                t!("error.database.creation_failed", error = err).to_string()
            }
            RitmoErr::DataIntegrityError(err) => {
                t!("error.database.integrity_error", error = err).to_string()
            }
            RitmoErr::InvalidTableName(name) => {
                t!("error.database.invalid_table", name = name).to_string()
            }
            RitmoErr::InvalidColumnName(name) => {
                t!("error.database.invalid_column", name = name).to_string()
            }
            RitmoErr::CommitFailed(err) => {
                t!("error.database.commit_failed", error = err).to_string()
            }

            // File errors
            RitmoErr::IoError(err) => {
                t!("error.file.io_error", error = err).to_string()
            }
            RitmoErr::FileAccessError(err) => {
                t!("error.file.access_error", error = err.to_string()).to_string()
            }
            RitmoErr::FileNotFound(path) => {
                t!("error.file.not_found", path = path).to_string()
            }
            RitmoErr::PathError(err) => {
                t!("error.generic.path_error", error = err).to_string()
            }

            // Import/Export errors
            RitmoErr::ImportError(err) => {
                t!("error.import.error", error = err).to_string()
            }
            RitmoErr::ExportError(err) => {
                t!("error.export.error", error = err).to_string()
            }

            // Config errors
            RitmoErr::ConfigDirNotFound => {
                t!("error.config.dir_not_found").to_string()
            }
            RitmoErr::ConfigParseError(err) => {
                t!("error.config.parse_error", error = err).to_string()
            }

            // ML errors
            RitmoErr::MLError(err) => {
                t!("error.ml.error", error = err).to_string()
            }
            RitmoErr::NameParsingError(err) => {
                t!("error.ml.name_parsing", error = err).to_string()
            }
            RitmoErr::MergeError(err) => {
                t!("error.ml.merge_error", error = err).to_string()
            }

            // Validation errors
            RitmoErr::InvalidInput(err) => {
                t!("error.validation.invalid_input", error = err).to_string()
            }

            // Search errors
            RitmoErr::SearchAndAddFailed(err) => {
                t!("error.search.failed", error = err).to_string()
            }
            RitmoErr::NoResultsError(query) => {
                t!("error.search.no_results", query = query).to_string()
            }

            // Record errors
            RitmoErr::RecordNotFound => {
                t!("error.record.not_found").to_string()
            }

            // Generic/Other errors
            RitmoErr::Generic(err) => {
                t!("error.generic.error", error = err).to_string()
            }
            RitmoErr::UnknownError(err) => {
                t!("error.generic.unknown", error = err).to_string()
            }
            RitmoErr::OtherError(err) => {
                t!("error.generic.other", error = err).to_string()
            }
            RitmoErr::SqlxError(err) => {
                t!("error.generic.sqlx_error", error = err.to_string()).to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n_utils::set_locale;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_database_error_localization_english() {
        set_locale("en");

        let err = RitmoErr::DatabaseNotFound("/path/to/db".to_string());
        assert_eq!(err.localized_message(), "Database not found: /path/to/db");
    }

    #[test]
    #[serial]
    fn test_database_error_localization_italian() {
        set_locale("it");

        let err = RitmoErr::DatabaseNotFound("/path/to/db".to_string());
        assert_eq!(err.localized_message(), "Database non trovato: /path/to/db");
    }

    #[test]
    #[serial]
    fn test_file_error_localization_english() {
        set_locale("en");

        let err = RitmoErr::FileNotFound("/path/to/file".to_string());
        assert_eq!(err.localized_message(), "File not found: /path/to/file");
    }

    #[test]
    #[serial]
    fn test_file_error_localization_italian() {
        set_locale("it");

        let err = RitmoErr::FileNotFound("/path/to/file".to_string());
        assert_eq!(err.localized_message(), "File non trovato: /path/to/file");
    }

    #[test]
    #[serial]
    fn test_ml_error_localization() {
        set_locale("en");
        let err = RitmoErr::MLError("Pattern matching failed".to_string());
        assert_eq!(err.localized_message(), "Machine Learning error: Pattern matching failed");

        set_locale("it");
        assert_eq!(err.localized_message(), "Errore di Machine Learning: Pattern matching failed");
    }

    #[test]
    #[serial]
    fn test_config_error_localization() {
        set_locale("en");
        let err = RitmoErr::ConfigDirNotFound;
        assert_eq!(err.localized_message(), "Configuration directory not found");

        set_locale("it");
        assert_eq!(err.localized_message(), "Directory di configurazione non trovata");
    }
}
