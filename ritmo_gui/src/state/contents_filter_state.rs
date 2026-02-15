use crate::events::{FilterField, FilterValue};
use crate::state::books_filter_state::ActiveFilter;
use ritmo_db_core::filters::ContentFilters;

/// State for content filtering
#[derive(Debug, Clone)]
pub struct ContentsFilterState {
    /// Active filters
    filters: Vec<ActiveFilter>,
}

impl Default for ContentsFilterState {
    fn default() -> Self {
        Self {
            filters: Vec::new(),
        }
    }
}

impl ContentsFilterState {
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
    
    /// Convert to ContentFilters for querying
    pub fn to_content_filters(&self) -> ContentFilters {
        let mut content_filters = ContentFilters::default();
        
        for filter in &self.filters {
            match &filter.field {
                FilterField::ContentAuthor => {
                    if let FilterValue::Specific(author) = &filter.value {
                        content_filters = content_filters.with_author(author);
                    }
                }
                FilterField::ContentType => {
                    if let FilterValue::Specific(ctype) = &filter.value {
                        content_filters = content_filters.with_content_type(ctype);
                    }
                }
                FilterField::ContentYear => {
                    if let FilterValue::Specific(year_str) = &filter.value {
                        if let Ok(year) = year_str.parse::<i32>() {
                            content_filters.year = Some(year);
                        }
                    }
                }
                _ => {}
            }
        }
        
        content_filters
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
