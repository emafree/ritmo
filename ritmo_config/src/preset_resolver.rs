use crate::presets::{BookFilterPreset, ContentFilterPreset, GlobalPresets, NamedPreset};

/// Helper per risolvere i preset con ordine di priorità: library > global
pub struct PresetResolver {
    global_presets: GlobalPresets,
    library_presets: Option<LibraryPresetsHolder>,
}

/// Holder temporaneo per i preset della libreria (evita dipendenza circolare)
/// I preset della libreria vengono passati come parametro quando necessario
pub struct LibraryPresetsHolder {
    pub books: std::collections::HashMap<String, NamedPreset<BookFilterPreset>>,
    pub contents: std::collections::HashMap<String, NamedPreset<ContentFilterPreset>>,
    pub default_books_preset: Option<String>,
    pub default_contents_preset: Option<String>,
}

impl PresetResolver {
    /// Crea un nuovo resolver con solo i preset globali
    pub fn new(global_presets: GlobalPresets) -> Self {
        Self {
            global_presets,
            library_presets: None,
        }
    }

    /// Crea un resolver con preset globali e della libreria
    pub fn with_library(
        global_presets: GlobalPresets,
        library_presets: LibraryPresetsHolder,
    ) -> Self {
        Self {
            global_presets,
            library_presets: Some(library_presets),
        }
    }

    /// Risolve un preset per libri seguendo l'ordine: library > global
    pub fn resolve_book_preset(&self, name: &str) -> Option<&NamedPreset<BookFilterPreset>> {
        // 1. Cerca nei preset della libreria
        if let Some(ref lib_presets) = self.library_presets {
            if let Some(preset) = lib_presets.books.get(name) {
                return Some(preset);
            }
        }

        // 2. Cerca nei preset globali
        self.global_presets.get_book_preset(name)
    }

    /// Risolve un preset per contenuti seguendo l'ordine: library > global
    pub fn resolve_content_preset(&self, name: &str) -> Option<&NamedPreset<ContentFilterPreset>> {
        // 1. Cerca nei preset della libreria
        if let Some(ref lib_presets) = self.library_presets {
            if let Some(preset) = lib_presets.contents.get(name) {
                return Some(preset);
            }
        }

        // 2. Cerca nei preset globali
        self.global_presets.get_content_preset(name)
    }

    /// Ottiene il preset di default per libri dalla libreria (se presente)
    pub fn get_default_books_preset(&self) -> Option<&str> {
        self.library_presets
            .as_ref()
            .and_then(|p| p.default_books_preset.as_deref())
    }

    /// Ottiene il preset di default per contenuti dalla libreria (se presente)
    pub fn get_default_contents_preset(&self) -> Option<&str> {
        self.library_presets
            .as_ref()
            .and_then(|p| p.default_contents_preset.as_deref())
    }

    /// Lista tutti i preset per libri disponibili (unione di library + global)
    pub fn list_all_book_presets(&self) -> Vec<(String, PresetSource)> {
        let mut presets = Vec::new();

        // Prima i preset della libreria
        if let Some(ref lib_presets) = self.library_presets {
            for name in lib_presets.books.keys() {
                presets.push((name.clone(), PresetSource::Library));
            }
        }

        // Poi i preset globali (evitando duplicati)
        for name in self.global_presets.list_book_presets() {
            if !presets.iter().any(|(n, _)| n == name) {
                presets.push((name.clone(), PresetSource::Global));
            }
        }

        presets
    }

    /// Lista tutti i preset per contenuti disponibili (unione di library + global)
    pub fn list_all_content_presets(&self) -> Vec<(String, PresetSource)> {
        let mut presets = Vec::new();

        // Prima i preset della libreria
        if let Some(ref lib_presets) = self.library_presets {
            for name in lib_presets.contents.keys() {
                presets.push((name.clone(), PresetSource::Library));
            }
        }

        // Poi i preset globali (evitando duplicati)
        for name in self.global_presets.list_content_presets() {
            if !presets.iter().any(|(n, _)| n == name) {
                presets.push((name.clone(), PresetSource::Global));
            }
        }

        presets
    }
}

/// Indica da dove proviene un preset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetSource {
    Library,
    Global,
}

impl PresetSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Library => "library",
            Self::Global => "global",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_global_presets() -> GlobalPresets {
        let mut global = GlobalPresets::new();
        global.add_book_preset(NamedPreset {
            name: "global_preset".to_string(),
            description: Some("Global preset".to_string()),
            filters: BookFilterPreset::default(),
        });
        global
    }

    fn create_test_library_presets() -> LibraryPresetsHolder {
        let mut books = HashMap::new();
        books.insert(
            "library_preset".to_string(),
            NamedPreset {
                name: "library_preset".to_string(),
                description: Some("Library preset".to_string()),
                filters: BookFilterPreset::default(),
            },
        );

        LibraryPresetsHolder {
            books,
            contents: HashMap::new(),
            default_books_preset: Some("library_preset".to_string()),
            default_contents_preset: None,
        }
    }

    #[test]
    fn test_resolver_with_only_global() {
        let global = create_test_global_presets();
        let resolver = PresetResolver::new(global);

        assert!(resolver.resolve_book_preset("global_preset").is_some());
        assert!(resolver.resolve_book_preset("library_preset").is_none());
        assert!(resolver.get_default_books_preset().is_none());
    }

    #[test]
    fn test_resolver_with_library() {
        let global = create_test_global_presets();
        let library = create_test_library_presets();
        let resolver = PresetResolver::with_library(global, library);

        assert!(resolver.resolve_book_preset("global_preset").is_some());
        assert!(resolver.resolve_book_preset("library_preset").is_some());
        assert_eq!(resolver.get_default_books_preset(), Some("library_preset"));
    }

    #[test]
    fn test_preset_priority() {
        let mut global = GlobalPresets::new();
        global.add_book_preset(NamedPreset {
            name: "same_name".to_string(),
            description: Some("Global version".to_string()),
            filters: BookFilterPreset::default(),
        });

        let mut books = HashMap::new();
        books.insert(
            "same_name".to_string(),
            NamedPreset {
                name: "same_name".to_string(),
                description: Some("Library version".to_string()),
                filters: BookFilterPreset::default(),
            },
        );

        let library = LibraryPresetsHolder {
            books,
            contents: HashMap::new(),
            default_books_preset: None,
            default_contents_preset: None,
        };

        let resolver = PresetResolver::with_library(global, library);

        // Deve restituire la versione della libreria (priorità maggiore)
        let preset = resolver.resolve_book_preset("same_name").unwrap();
        assert_eq!(preset.description, Some("Library version".to_string()));
    }

    #[test]
    fn test_list_all_presets() {
        let global = create_test_global_presets();
        let library = create_test_library_presets();
        let resolver = PresetResolver::with_library(global, library);

        let all_presets = resolver.list_all_book_presets();
        assert_eq!(all_presets.len(), 2);

        // Il preset della libreria deve venire prima
        assert_eq!(all_presets[0].0, "library_preset");
        assert_eq!(all_presets[0].1, PresetSource::Library);

        assert_eq!(all_presets[1].0, "global_preset");
        assert_eq!(all_presets[1].1, PresetSource::Global);
    }
}
