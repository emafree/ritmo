use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tipo di preset: Books o Contents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetType {
    Books,
    Contents,
}

impl PresetType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "books" | "book" => Some(Self::Books),
            "contents" | "content" => Some(Self::Contents),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Books => "books",
            Self::Contents => "contents",
        }
    }
}

/// Preset per filtri libri
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BookFilterPreset {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    #[serde(default = "default_sort")]
    pub sort: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    #[serde(default)]
    pub offset: i64,
}

/// Preset per filtri contenuti
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentFilterPreset {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    #[serde(default = "default_sort")]
    pub sort: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    #[serde(default)]
    pub offset: i64,
}

fn default_sort() -> String {
    "title".to_string()
}

impl Default for BookFilterPreset {
    fn default() -> Self {
        Self {
            author: None,
            publisher: None,
            series: None,
            format: None,
            year: None,
            isbn: None,
            search: None,
            sort: default_sort(),
            limit: None,
            offset: 0,
        }
    }
}

impl Default for ContentFilterPreset {
    fn default() -> Self {
        Self {
            author: None,
            content_type: None,
            year: None,
            search: None,
            sort: default_sort(),
            limit: None,
            offset: 0,
        }
    }
}

/// Container per un preset con nome e descrizione
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedPreset<T> {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub filters: T,
}

/// Collezione di preset globali (salvati in ~/.config/ritmo/settings.toml)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalPresets {
    #[serde(default)]
    pub books: HashMap<String, NamedPreset<BookFilterPreset>>,

    #[serde(default)]
    pub contents: HashMap<String, NamedPreset<ContentFilterPreset>>,
}

impl GlobalPresets {
    pub fn new() -> Self {
        Self::default()
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_type_from_str() {
        assert_eq!(PresetType::from_str("books"), Some(PresetType::Books));
        assert_eq!(PresetType::from_str("book"), Some(PresetType::Books));
        assert_eq!(PresetType::from_str("contents"), Some(PresetType::Contents));
        assert_eq!(PresetType::from_str("content"), Some(PresetType::Contents));
        assert_eq!(PresetType::from_str("invalid"), None);
    }

    #[test]
    fn test_book_filter_preset_default() {
        let preset = BookFilterPreset::default();
        assert_eq!(preset.sort, "title");
        assert_eq!(preset.offset, 0);
        assert!(preset.author.is_none());
    }

    #[test]
    fn test_global_presets_add_and_get() {
        let mut presets = GlobalPresets::new();

        let book_preset = NamedPreset {
            name: "my_ebooks".to_string(),
            description: Some("All my EPUB books".to_string()),
            filters: BookFilterPreset {
                format: Some("epub".to_string()),
                ..Default::default()
            },
        };

        presets.add_book_preset(book_preset.clone());

        let retrieved = presets.get_book_preset("my_ebooks");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "my_ebooks");
        assert_eq!(retrieved.unwrap().filters.format, Some("epub".to_string()));
    }

    #[test]
    fn test_global_presets_remove() {
        let mut presets = GlobalPresets::new();

        let book_preset = NamedPreset {
            name: "test".to_string(),
            description: None,
            filters: BookFilterPreset::default(),
        };

        presets.add_book_preset(book_preset);
        assert_eq!(presets.books.len(), 1);

        let removed = presets.remove_book_preset("test");
        assert!(removed.is_some());
        assert_eq!(presets.books.len(), 0);
    }

    #[test]
    fn test_global_presets_list() {
        let mut presets = GlobalPresets::new();

        presets.add_book_preset(NamedPreset {
            name: "preset1".to_string(),
            description: None,
            filters: BookFilterPreset::default(),
        });

        presets.add_book_preset(NamedPreset {
            name: "preset2".to_string(),
            description: None,
            filters: BookFilterPreset::default(),
        });

        let list = presets.list_book_presets();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&&"preset1".to_string()));
        assert!(list.contains(&&"preset2".to_string()));
    }
}
