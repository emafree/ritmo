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
                    if let FilterValue::Specific(authors) = &filter.value {
                        for author in authors {
                            content_filters = content_filters.with_author(author);
                        }
                    }
                }
                FilterField::ContentType => {
                    if let FilterValue::Specific(ctypes) = &filter.value {
                        for ctype in ctypes {
                            content_filters = content_filters.with_content_type(ctype);
                        }
                    }
                }
                FilterField::ContentYear => {
                    if let FilterValue::Specific(years) = &filter.value {
                        // ContentFilters.year is a single exact-match value; use the first entry only.
                        if let Some(year_str) = years.first() {
                            if let Ok(year) = year_str.parse::<i32>() {
                                content_filters.year = Some(year);
                            }
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
        // Convert to a simpler serializable format
        let simple_filters: Vec<SimpleFilter> = self.filters.iter().map(|f| SimpleFilter {
            field: format!("{:?}", f.field),
            value: match &f.value {
                FilterValue::Nessuno => "NESSUNO".to_string(),
                FilterValue::AlmenoUno => "ALMENO_UNO".to_string(),
                FilterValue::Specific(vals) => format!("SPECIFIC:{}", vals.join("|")),
            },
        }).collect();
        
        serde_json::to_string(&simple_filters).ok()
    }
    
    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Self {
        let simple_filters: Vec<SimpleFilter> = serde_json::from_str(json).unwrap_or_default();
        
        let filters = simple_filters.into_iter().filter_map(|sf| {
            let field = match sf.field.as_str() {
                "ContentAuthor" => FilterField::ContentAuthor,
                "ContentType" => FilterField::ContentType,
                "ContentYear" => FilterField::ContentYear,
                _ => return None,
            };
            
            let value = match sf.value.as_str() {
                "NESSUNO" => FilterValue::Nessuno,
                "ALMENO_UNO" => FilterValue::AlmenoUno,
                s if s.starts_with("SPECIFIC:") => {
                        let raw = s.strip_prefix("SPECIFIC:").unwrap();
                        let values: Vec<String> = raw.split('|').map(|v| v.to_string()).collect();
                        FilterValue::Specific(values)
                    }
                _ => return None,
            };
            
            Some(ActiveFilter { field, value })
        }).collect();
        
        Self { filters }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SimpleFilter {
    field: String,
    value: String,
}
