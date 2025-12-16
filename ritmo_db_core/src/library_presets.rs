use ritmo_config::presets::{BookFilterPreset, ContentFilterPreset, NamedPreset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Preset specifici di una libreria (salvati in library/config/filters.toml)
/// Questi preset sono portabili con la libreria
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibraryPresets {
    /// Preset per filtri libri
    #[serde(default)]
    pub books: HashMap<String, NamedPreset<BookFilterPreset>>,

    /// Preset per filtri contenuti
    #[serde(default)]
    pub contents: HashMap<String, NamedPreset<ContentFilterPreset>>,

    /// Preset di default per la vista libri (opzionale)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_books_preset: Option<String>,

    /// Preset di default per la vista contenuti (opzionale)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_contents_preset: Option<String>,
}

impl LibraryPresets {
    pub fn new() -> Self {
        Self::default()
    }

    /// Carica i preset dal file filters.toml, crea file vuoto se non esiste
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();

        if path.exists() {
            let content = fs::read_to_string(path)?;
            let presets: Self = toml::from_str(&content)?;
            Ok(presets)
        } else {
            // Crea preset di default
            let presets = Self::with_examples();
            presets.save(path)?;
            Ok(presets)
        }
    }

    /// Crea preset con esempi predefiniti
    pub fn with_examples() -> Self {
        let mut presets = Self::new();

        // Esempio: preset per libri EPUB
        presets.add_book_preset(NamedPreset {
            name: "epub_only".to_string(),
            description: Some("Solo libri in formato EPUB".to_string()),
            filters: BookFilterPreset {
                format: Some("epub".to_string()),
                sort: "title".to_string(),
                ..Default::default()
            },
        });

        // Esempio: preset per libri PDF
        presets.add_book_preset(NamedPreset {
            name: "pdf_only".to_string(),
            description: Some("Solo libri in formato PDF".to_string()),
            filters: BookFilterPreset {
                format: Some("pdf".to_string()),
                sort: "title".to_string(),
                ..Default::default()
            },
        });

        // Esempio: preset per contenuti tipo romanzo
        presets.add_content_preset(NamedPreset {
            name: "novels".to_string(),
            description: Some("Solo romanzi".to_string()),
            filters: ContentFilterPreset {
                content_type: Some("Romanzo".to_string()),
                sort: "title".to_string(),
                ..Default::default()
            },
        });

        presets
    }

    /// Salva i preset su file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();

        // Crea directory se non esiste
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Aggiunge un preset per libri
    pub fn add_book_preset(&mut self, preset: NamedPreset<BookFilterPreset>) {
        self.books.insert(preset.name.clone(), preset);
    }

    /// Aggiunge un preset per contenuti
    pub fn add_content_preset(&mut self, preset: NamedPreset<ContentFilterPreset>) {
        self.contents.insert(preset.name.clone(), preset);
    }

    /// Rimuove un preset per libri
    pub fn remove_book_preset(&mut self, name: &str) -> Option<NamedPreset<BookFilterPreset>> {
        self.books.remove(name)
    }

    /// Rimuove un preset per contenuti
    pub fn remove_content_preset(
        &mut self,
        name: &str,
    ) -> Option<NamedPreset<ContentFilterPreset>> {
        self.contents.remove(name)
    }

    /// Ottiene un preset per libri
    pub fn get_book_preset(&self, name: &str) -> Option<&NamedPreset<BookFilterPreset>> {
        self.books.get(name)
    }

    /// Ottiene un preset per contenuti
    pub fn get_content_preset(&self, name: &str) -> Option<&NamedPreset<ContentFilterPreset>> {
        self.contents.get(name)
    }

    /// Lista tutti i nomi dei preset per libri
    pub fn list_book_presets(&self) -> Vec<&String> {
        self.books.keys().collect()
    }

    /// Lista tutti i nomi dei preset per contenuti
    pub fn list_content_presets(&self) -> Vec<&String> {
        self.contents.keys().collect()
    }

    /// Imposta il preset di default per la vista libri
    pub fn set_default_books_preset(&mut self, name: Option<String>) {
        self.default_books_preset = name;
    }

    /// Imposta il preset di default per la vista contenuti
    pub fn set_default_contents_preset(&mut self, name: Option<String>) {
        self.default_contents_preset = name;
    }

    /// Ottiene il preset di default per la vista libri
    pub fn get_default_books_preset(&self) -> Option<&str> {
        self.default_books_preset.as_deref()
    }

    /// Ottiene il preset di default per la vista contenuti
    pub fn get_default_contents_preset(&self) -> Option<&str> {
        self.default_contents_preset.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_library_presets_default() {
        let presets = LibraryPresets::new();
        assert!(presets.books.is_empty());
        assert!(presets.contents.is_empty());
        assert!(presets.default_books_preset.is_none());
    }

    #[test]
    fn test_library_presets_with_examples() {
        let presets = LibraryPresets::with_examples();
        assert!(!presets.books.is_empty());
        assert!(!presets.contents.is_empty());
        assert!(presets.get_book_preset("epub_only").is_some());
        assert!(presets.get_book_preset("pdf_only").is_some());
        assert!(presets.get_content_preset("novels").is_some());
    }

    #[test]
    fn test_add_and_get_preset() {
        let mut presets = LibraryPresets::new();

        let book_preset = NamedPreset {
            name: "test".to_string(),
            description: Some("Test preset".to_string()),
            filters: BookFilterPreset {
                format: Some("mobi".to_string()),
                ..Default::default()
            },
        };

        presets.add_book_preset(book_preset.clone());

        let retrieved = presets.get_book_preset("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test");
        assert_eq!(retrieved.unwrap().filters.format, Some("mobi".to_string()));
    }

    #[test]
    fn test_remove_preset() {
        let mut presets = LibraryPresets::with_examples();
        assert!(presets.get_book_preset("epub_only").is_some());

        let removed = presets.remove_book_preset("epub_only");
        assert!(removed.is_some());
        assert!(presets.get_book_preset("epub_only").is_none());
    }

    #[test]
    fn test_save_and_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut presets = LibraryPresets::new();
        presets.add_book_preset(NamedPreset {
            name: "test".to_string(),
            description: None,
            filters: BookFilterPreset::default(),
        });

        presets.save(path).unwrap();

        let loaded = LibraryPresets::load_or_create(path).unwrap();
        assert_eq!(loaded.books.len(), 1);
        assert!(loaded.get_book_preset("test").is_some());
    }

    #[test]
    fn test_default_presets() {
        let mut presets = LibraryPresets::new();
        assert!(presets.get_default_books_preset().is_none());

        presets.set_default_books_preset(Some("epub_only".to_string()));
        assert_eq!(presets.get_default_books_preset(), Some("epub_only"));

        presets.set_default_books_preset(None);
        assert!(presets.get_default_books_preset().is_none());
    }

    #[test]
    fn test_list_presets() {
        let presets = LibraryPresets::with_examples();

        let book_presets = presets.list_book_presets();
        assert!(book_presets.len() >= 2);
        assert!(book_presets.contains(&&"epub_only".to_string()));
        assert!(book_presets.contains(&&"pdf_only".to_string()));

        let content_presets = presets.list_content_presets();
        assert!(content_presets.len() >= 1);
        assert!(content_presets.contains(&&"novels".to_string()));
    }
}
