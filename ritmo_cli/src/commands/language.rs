//! Language preference commands

use ritmo_config::AppSettings;
use ritmo_db::i18n_utils::{self, SUPPORTED_LOCALES};
use ritmo_errors::RitmoErr;
use rust_i18n::t;
use std::path::Path;

/// Set the preferred language for the application
pub fn cmd_set_language(
    language: String,
    app_settings: &mut AppSettings,
    settings_path: &Path,
) -> Result<(), RitmoErr> {
    let language = language.to_lowercase();

    // Validate language is supported
    if !SUPPORTED_LOCALES.contains(&language.as_str()) {
        eprintln!(
            "{}",
            t!(
                "cli.language.unsupported",
                language = language,
                supported = SUPPORTED_LOCALES.join(", ")
            )
        );
        return Err(RitmoErr::Generic(format!(
            "Unsupported language: {}. Supported: {}",
            language,
            SUPPORTED_LOCALES.join(", ")
        )));
    }

    // Update settings
    app_settings.set_language(language.clone());
    app_settings.save(settings_path)?;

    // Apply the new language immediately
    i18n_utils::set_locale(&language);

    println!("{}", t!("cli.language.set_success", language = language));

    Ok(())
}

/// Get the current language preference
pub fn cmd_get_language(app_settings: &AppSettings) {
    let current_language = app_settings.get_language();
    let active_language = i18n_utils::get_locale();

    println!("{}", t!("cli.language.saved_preference", language = current_language));
    println!("{}", t!("cli.language.active_language", language = active_language));

    // Show if there's an environment variable override
    if current_language != active_language {
        println!("{}", t!("cli.language.env_override"));
    }

    // Show available languages
    println!("\n{}", t!("cli.language.available"));
    for locale in SUPPORTED_LOCALES {
        let marker = if *locale == active_language { "→" } else { " " };
        println!("  {} {}", marker, locale);
    }
}
