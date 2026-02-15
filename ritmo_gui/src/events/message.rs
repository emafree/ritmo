/// Messages for UI events and state changes
#[derive(Debug, Clone)]
pub enum Message {
    // Tab navigation
    TabSelected(TabState),
    
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
    
    // UI events
    ThemeChanged(Theme),
}

/// Represents which tab is active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabState {
    Books,
    Contents,
}

/// Available filter fields
#[derive(Debug, Clone, PartialEq, Eq)]
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
