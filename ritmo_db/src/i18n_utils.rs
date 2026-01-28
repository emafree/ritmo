//! i18n utilities for locale detection and management
//!
//! This module provides utilities for detecting and setting the application locale.
//! It checks (in order):
//! 1. RITMO_LANG environment variable
//! 2. LANG environment variable
//! 3. System locale (if available)
//! 4. Fallback to "en"

use std::env;

/// Supported locales in the application
pub const SUPPORTED_LOCALES: &[&str] = &["en", "it"];

/// Default locale if none can be detected
pub const DEFAULT_LOCALE: &str = "en";

/// Detect the best locale to use based on environment variables and system settings
///
/// Priority order:
/// 1. RITMO_LANG env var (e.g., "it", "en")
/// 2. LANG env var (e.g., "it_IT.UTF-8" -> "it")
/// 3. Fallback to DEFAULT_LOCALE ("en")
///
/// # Examples
///
/// ```
/// use ritmo_db::i18n_utils::detect_locale;
///
/// let locale = detect_locale();
/// assert!(locale == "en" || locale == "it");
/// ```
pub fn detect_locale() -> &'static str {
    // 1. Check RITMO_LANG environment variable
    if let Ok(ritmo_lang) = env::var("RITMO_LANG") {
        let locale = ritmo_lang.to_lowercase();
        for &supported in SUPPORTED_LOCALES {
            if locale.starts_with(supported) {
                return supported;
            }
        }
    }

    // 2. Check LANG environment variable
    if let Ok(lang) = env::var("LANG") {
        let locale = lang.to_lowercase();
        for &supported in SUPPORTED_LOCALES {
            if locale.starts_with(supported) {
                return supported;
            }
        }
    }

    // 3. Fallback to default
    DEFAULT_LOCALE
}

/// Detect the best locale considering saved preferences
///
/// Priority order:
/// 1. RITMO_LANG env var (e.g., "it", "en") - temporary override
/// 2. Saved preference from config file
/// 3. LANG env var (e.g., "it_IT.UTF-8" -> "it") - system default
/// 4. Fallback to DEFAULT_LOCALE ("en")
///
/// # Arguments
///
/// * `saved_preference` - Optional saved language preference from config
///
/// # Examples
///
/// ```
/// use ritmo_db::i18n_utils::detect_locale_with_preference;
///
/// let locale = detect_locale_with_preference(Some("it"));
/// assert_eq!(locale, "it");
/// ```
pub fn detect_locale_with_preference(saved_preference: Option<&str>) -> &'static str {
    // 1. Check RITMO_LANG environment variable (temporary override)
    if let Ok(ritmo_lang) = env::var("RITMO_LANG") {
        let locale = ritmo_lang.to_lowercase();
        for &supported in SUPPORTED_LOCALES {
            if locale.starts_with(supported) {
                return supported;
            }
        }
    }

    // 2. Check saved preference
    if let Some(pref) = saved_preference {
        let locale = pref.to_lowercase();
        for &supported in SUPPORTED_LOCALES {
            if locale.starts_with(supported) {
                return supported;
            }
        }
    }

    // 3. Check LANG environment variable (system default)
    if let Ok(lang) = env::var("LANG") {
        let locale = lang.to_lowercase();
        for &supported in SUPPORTED_LOCALES {
            if locale.starts_with(supported) {
                return supported;
            }
        }
    }

    // 4. Fallback to default
    DEFAULT_LOCALE
}

/// Set the application locale
///
/// This sets the rust-i18n locale for the current thread.
///
/// # Arguments
///
/// * `locale` - The locale code (e.g., "en", "it")
///
/// # Examples
///
/// ```
/// use ritmo_db::i18n_utils::set_locale;
///
/// set_locale("it");
/// // Now all t!() calls will return Italian translations
/// ```
pub fn set_locale(locale: &str) {
    rust_i18n::set_locale(locale);
}

/// Get the current active locale
///
/// # Examples
///
/// ```
/// use ritmo_db::i18n_utils::get_locale;
///
/// let current = get_locale();
/// println!("Current locale: {}", current);
/// ```
pub fn get_locale() -> String {
    rust_i18n::locale().to_string()
}

/// Initialize i18n system with auto-detected locale
///
/// This should be called early in the application startup.
/// It detects the best locale and sets it as the active locale.
///
/// # Examples
///
/// ```
/// use ritmo_db::i18n_utils::init_i18n;
///
/// // Call this in main() or library initialization
/// init_i18n();
/// ```
pub fn init_i18n() {
    let locale = detect_locale();
    set_locale(locale);
}

/// Initialize i18n system considering saved preference
///
/// This should be called early in the application startup.
/// It detects the best locale (considering saved preference) and sets it as the active locale.
///
/// Priority order:
/// 1. RITMO_LANG env var (temporary override)
/// 2. Saved preference from config file
/// 3. LANG env var (system default)
/// 4. Default ("en")
///
/// # Arguments
///
/// * `saved_preference` - Optional saved language preference from config
///
/// # Examples
///
/// ```
/// use ritmo_db::i18n_utils::init_i18n_with_preference;
///
/// // With saved preference
/// init_i18n_with_preference(Some("it"));
///
/// // Without saved preference (falls back to env vars)
/// init_i18n_with_preference(None);
/// ```
pub fn init_i18n_with_preference(saved_preference: Option<&str>) {
    let locale = detect_locale_with_preference(saved_preference);
    set_locale(locale);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_detect_locale_default() {
        // Without env vars, should return default
        let locale = detect_locale();
        assert!(SUPPORTED_LOCALES.contains(&locale));
    }

    #[test]
    fn test_supported_locales() {
        assert!(SUPPORTED_LOCALES.contains(&"en"));
        assert!(SUPPORTED_LOCALES.contains(&"it"));
    }

    #[test]
    #[serial]
    fn test_set_and_get_locale() {
        set_locale("it");
        assert_eq!(get_locale(), "it".to_string());

        set_locale("en");
        assert_eq!(get_locale(), "en".to_string());
    }
}
