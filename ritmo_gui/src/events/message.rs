use crate::config::theme::{ThemePreset, CustomTheme};
use crate::config::ViewMode;

/// Messages for UI events and state changes
#[derive(Debug, Clone)]
pub enum Message {
    // Tab navigation
    TabSelected(TabState),
    
    // View mode
    ViewModeChanged(ViewMode),
    
    // Books operations
    BookSelected(i64),
    BookDoubleClicked(i64),
    
    // Contents operations
    ContentSelected(i64),
    ContentDoubleClicked(i64),
    
    // Filter operations  
    FilterFieldSelected(FilterField),
    FilterValueSelected(FilterValue),
    FilterAdded,
    FilterRemoved(usize),
    FilterCleared,
    
    // Menu operations
    MenuAddBook,
    MenuRemoveBook,
    MenuDuplicateBook,
    MenuOpenSettings,
    MenuExit,
    
    // Theme operations
    ThemePresetSelected(ThemePreset),
    CustomThemeSelected(String),
    ThemeEditorOpened,
    ThemeEditorClosed,
    ThemeManagerOpened,
    ThemeManagerClosed,
    CreateCustomTheme,
    SaveCustomTheme(CustomTheme),
    DeleteCustomTheme(String),
}

/// Represents which tab is active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabState {
    Books,
    Contents,
}

/// Available filter fields
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterField {
    // Books
    BookAuthor,
    BookPublisher,
    BookSeries,
    BookFormat,
    BookYear,
    
    // Contents
    ContentAuthor,
    ContentType,
    ContentYear,
}

impl FilterField {
    pub fn display_name(&self) -> &str {
        match self {
            FilterField::BookAuthor => "Author",
            FilterField::BookPublisher => "Publisher",
            FilterField::BookSeries => "Series",
            FilterField::BookFormat => "Format",
            FilterField::BookYear => "Year",
            FilterField::ContentAuthor => "Author",
            FilterField::ContentType => "Type",
            FilterField::ContentYear => "Year",
        }
    }
}

/// Filter value selection
#[derive(Debug, Clone, PartialEq)]
pub enum FilterValue {
    /// Exclude items without this field
    Nessuno,
    /// Include items that have this field
    AlmenoUno,
    /// Specific value
    Specific(String),
}

impl FilterValue {
    pub fn display_name(&self) -> String {
        match self {
            FilterValue::Nessuno => "NESSUNO".to_string(),
            FilterValue::AlmenoUno => "ALMENO UNO".to_string(),
            FilterValue::Specific(s) => s.clone(),
        }
    }
}

/// Theme selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Theme {
    Dark,
    Light,
}

/// Filter matching mode used in the filter popup row
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    /// Exact match
    Uguale,
    /// Non-matching
    Diverso,
    /// Fuzzy / similar
    Simile,
    /// Contains substring
    Contiene,
}

impl Default for FilterMode {
    fn default() -> Self {
        FilterMode::Contiene
    }
}

impl FilterMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            FilterMode::Uguale => "uguale",
            FilterMode::Diverso => "diverso",
            FilterMode::Simile => "simile",
            FilterMode::Contiene => "contiene",
        }
    }

    pub fn all() -> &'static [FilterMode] {
        &[
            FilterMode::Uguale,
            FilterMode::Diverso,
            FilterMode::Simile,
            FilterMode::Contiene,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_mode_display_names() {
        assert_eq!(FilterMode::Uguale.display_name(), "uguale");
        assert_eq!(FilterMode::Diverso.display_name(), "diverso");
        assert_eq!(FilterMode::Simile.display_name(), "simile");
        assert_eq!(FilterMode::Contiene.display_name(), "contiene");
    }

    #[test]
    fn test_filter_mode_default() {
        assert_eq!(FilterMode::default(), FilterMode::Contiene);
    }

    #[test]
    fn test_filter_mode_all() {
        let modes = FilterMode::all();
        assert_eq!(modes.len(), 4);
        assert!(modes.contains(&FilterMode::Uguale));
        assert!(modes.contains(&FilterMode::Diverso));
        assert!(modes.contains(&FilterMode::Simile));
        assert!(modes.contains(&FilterMode::Contiene));
    }
}
