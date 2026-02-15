use crate::events::{FilterField, FilterValue};
use ritmo_db_core::filters::BookFilters;

/// State for book filtering
#[derive(Debug, Clone)]
pub struct BooksFilterState {
    /// Active filters
    filters: Vec<ActiveFilter>,
}

#[derive(Debug, Clone)]
pub struct ActiveFilter {
    pub field: FilterField,
    pub value: FilterValue,
}

impl Default for BooksFilterState {
    fn default() -> Self {
        Self {
            filters: Vec::new(),
        }
    }
}

impl BooksFilterState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a filter
    pub fn add_filter(&mut self, field: FilterField, value: FilterValue) {
        self.filters.push(ActiveFilter { field, value });
    }
    
    /// Remove filter at index
    pub fn remove_filter(&mut self, index: usize) {
        if index < self.filters.len() {
            self.filters.remove(index);
        }
    }
    
    /// Clear all filters
    pub fn clear(&mut self) {
        self.filters.clear();
    }
    
    /// Get active filters
    pub fn get_filters(&self) -> &[ActiveFilter] {
        &self.filters
    }
    
    /// Convert to BookFilters for querying
    pub fn to_book_filters(&self) -> BookFilters {
        let mut book_filters = BookFilters::default();
        
        for filter in &self.filters {
            match &filter.field {
                FilterField::BookAuthor => {
                    if let FilterValue::Specific(author) = &filter.value {
                        book_filters = book_filters.with_author(author);
                    }
                }
                FilterField::BookPublisher => {
                    if let FilterValue::Specific(pub_name) = &filter.value {
                        book_filters = book_filters.with_publisher(pub_name);
                    }
                }
                FilterField::BookSeries => {
                    if let FilterValue::Specific(series) = &filter.value {
                        book_filters = book_filters.with_series(series);
                    }
                }
                FilterField::BookFormat => {
                    if let FilterValue::Specific(format) = &filter.value {
                        book_filters = book_filters.with_format(format);
                    }
                }
                FilterField::BookYear => {
                    if let FilterValue::Specific(year_str) = &filter.value {
                        if let Ok(year) = year_str.parse::<i32>() {
                            book_filters.year = Some(year);
                        }
                    }
                }
                _ => {}
            }
        }
        
        book_filters
    }
    
    /// Serialize to JSON string for persistence
    pub fn to_json(&self) -> Option<String> {
        serde_json::to_string(&self.filters).ok()
    }
    
    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Self {
        let filters = serde_json::from_str(json).unwrap_or_default();
        Self { filters }
    }
}

// Implement Serialize/Deserialize for ActiveFilter
impl serde::Serialize for ActiveFilter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ActiveFilter", 2)?;
        state.serialize_field("field", &format!("{:?}", self.field))?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for ActiveFilter {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Simple implementation - in production, properly deserialize
        Ok(ActiveFilter {
            field: FilterField::BookAuthor,
            value: FilterValue::AlmenoUno,
        })
    }
}

impl serde::Serialize for FilterValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            FilterValue::Nessuno => "NESSUNO".to_string(),
            FilterValue::AlmenoUno => "ALMENO_UNO".to_string(),
            FilterValue::Specific(v) => format!("SPECIFIC:{}", v),
        };
        serializer.serialize_str(&s)
    }
}

impl<'de> serde::Deserialize<'de> for FilterValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "NESSUNO" => FilterValue::Nessuno,
            "ALMENO_UNO" => FilterValue::AlmenoUno,
            _ if s.starts_with("SPECIFIC:") => {
                FilterValue::Specific(s.strip_prefix("SPECIFIC:").unwrap().to_string())
            }
            _ => FilterValue::AlmenoUno,
        })
    }
}
