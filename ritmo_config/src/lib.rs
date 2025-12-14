mod app_settings;
mod portable;
mod presets;

pub use app_settings::AppSettings;
pub use portable::{detect_portable_library, is_running_portable};
pub use presets::{BookFilterPreset, ContentFilterPreset, GlobalPresets, NamedPreset, PresetType};
pub use ritmo_errors::RitmoErr;

use std::path::PathBuf;

/// Ottiene il percorso della directory di configurazione di Ritmo
/// Default: ~/.config/ritmo su Linux/Mac, %APPDATA%/ritmo su Windows
pub fn config_dir() -> Result<PathBuf, RitmoErr> {
    dirs::config_dir()
        .map(|p| p.join("ritmo"))
        .ok_or(RitmoErr::ConfigDirNotFound)
}

/// Ottiene il percorso del file delle impostazioni globali
pub fn settings_file() -> Result<PathBuf, RitmoErr> {
    Ok(config_dir()?.join("settings.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir() {
        let dir = config_dir();
        assert!(dir.is_ok());
        let path = dir.unwrap();
        assert!(path.to_string_lossy().contains("ritmo"));
    }

    #[test]
    fn test_settings_file() {
        let file = settings_file();
        assert!(file.is_ok());
        let path = file.unwrap();
        assert!(path.to_string_lossy().contains("settings.toml"));
    }
}
