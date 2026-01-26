use crate::presets::GlobalPresets;
use ritmo_errors::RitmoErr;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_RECENT_LIBRARIES: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Ultima libreria aperta
    pub last_library_path: Option<PathBuf>,

    /// Librerie recenti (max 10)
    #[serde(default)]
    pub recent_libraries: Vec<PathBuf>,

    /// Preferenze UI
    #[serde(default)]
    pub preferences: Preferences,

    /// Preset globali per filtri
    #[serde(default)]
    pub presets: GlobalPresets,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    /// Lingua interfaccia (es. "it", "en")
    #[serde(default = "default_language")]
    pub ui_language: String,

    /// Tema UI (es. "light", "dark")
    #[serde(default = "default_theme")]
    pub ui_theme: String,
}

fn default_language() -> String {
    "it".to_string()
}

fn default_theme() -> String {
    "light".to_string()
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            ui_language: default_language(),
            ui_theme: default_theme(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            last_library_path: None,
            recent_libraries: Vec::new(),
            preferences: Preferences::default(),
            presets: GlobalPresets::default(),
        }
    }
}

impl AppSettings {
    /// Carica le impostazioni dal file, crea default se non esiste
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Result<Self, RitmoErr> {
        let path = path.as_ref();

        if path.exists() {
            let content = fs::read_to_string(path)?;
            let settings: Self = toml::from_str(&content)?;
            Ok(settings)
        } else {
            // Crea settings di default e salvale
            let settings = Self::default();
            settings.save(path)?;
            Ok(settings)
        }
    }

    /// Salva le impostazioni su file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), RitmoErr> {
        let path = path.as_ref();

        // Crea directory se non esiste
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Aggiorna l'ultima libreria usata e la aggiunge ai recenti
    pub fn update_last_library<P: AsRef<Path>>(&mut self, library_path: P) {
        let path = library_path.as_ref().to_path_buf();

        // Aggiorna last_library_path
        self.last_library_path = Some(path.clone());

        // Rimuovi dalla lista se già presente
        self.recent_libraries.retain(|p| p != &path);

        // Aggiungi all'inizio
        self.recent_libraries.insert(0, path);

        // Mantieni solo le ultime MAX_RECENT_LIBRARIES
        self.recent_libraries.truncate(MAX_RECENT_LIBRARIES);
    }

    /// Rimuovi una libreria dalla lista recenti
    pub fn remove_from_recent<P: AsRef<Path>>(&mut self, library_path: P) {
        let path = library_path.as_ref();
        self.recent_libraries.retain(|p| p != path);

        // Se era l'ultima libreria, resetta
        if let Some(last) = &self.last_library_path {
            if last == path {
                self.last_library_path = self.recent_libraries.first().cloned();
            }
        }
    }

    /// Ottieni la libreria da usare (priorità: portable > last_library > None)
    pub fn get_library_to_use(&self) -> Option<PathBuf> {
        // 1. Controlla se in modalità portabile
        if let Some(portable_lib) = crate::portable::detect_portable_library() {
            return Some(portable_lib);
        }

        // 2. Usa last_library_path se presente
        self.last_library_path.clone()
    }

    /// Update the UI language preference
    pub fn set_language(&mut self, language: String) {
        self.preferences.ui_language = language;
    }

    /// Get the current UI language preference
    pub fn get_language(&self) -> &str {
        &self.preferences.ui_language
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert!(settings.last_library_path.is_none());
        assert!(settings.recent_libraries.is_empty());
        assert_eq!(settings.preferences.ui_language, "it");
    }

    #[test]
    fn test_save_and_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut settings = AppSettings::default();
        settings.update_last_library("/test/library");

        settings.save(path).unwrap();

        let loaded = AppSettings::load_or_create(path).unwrap();
        assert_eq!(
            loaded.last_library_path,
            Some(PathBuf::from("/test/library"))
        );
        assert_eq!(loaded.recent_libraries.len(), 1);
    }

    #[test]
    fn test_update_last_library() {
        let mut settings = AppSettings::default();

        settings.update_last_library("/lib1");
        assert_eq!(settings.recent_libraries.len(), 1);
        assert_eq!(settings.last_library_path, Some(PathBuf::from("/lib1")));

        settings.update_last_library("/lib2");
        assert_eq!(settings.recent_libraries.len(), 2);
        assert_eq!(settings.last_library_path, Some(PathBuf::from("/lib2")));
        assert_eq!(settings.recent_libraries[0], PathBuf::from("/lib2"));
        assert_eq!(settings.recent_libraries[1], PathBuf::from("/lib1"));
    }

    #[test]
    fn test_recent_libraries_limit() {
        let mut settings = AppSettings::default();

        // Aggiungi più di MAX_RECENT_LIBRARIES
        for i in 0..15 {
            settings.update_last_library(format!("/lib{}", i));
        }

        // Deve mantenere solo le ultime 10
        assert_eq!(settings.recent_libraries.len(), MAX_RECENT_LIBRARIES);
        assert_eq!(settings.recent_libraries[0], PathBuf::from("/lib14"));
    }

    #[test]
    fn test_remove_from_recent() {
        let mut settings = AppSettings::default();

        settings.update_last_library("/lib1");
        settings.update_last_library("/lib2");
        settings.update_last_library("/lib3");

        settings.remove_from_recent("/lib2");
        assert_eq!(settings.recent_libraries.len(), 2);
        assert!(!settings.recent_libraries.contains(&PathBuf::from("/lib2")));
    }
}
