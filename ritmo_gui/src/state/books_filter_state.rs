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
                    if let FilterValue::Specific(authors) = &filter.value {
                        for author in authors {
                            book_filters = book_filters.with_author(author);
                        }
                    }
                }
                FilterField::BookPublisher => {
                    if let FilterValue::Specific(publishers) = &filter.value {
                        for pub_name in publishers {
                            book_filters = book_filters.with_publisher(pub_name);
                        }
                    }
                }
                FilterField::BookSeries => {
                    if let FilterValue::Specific(series_vals) = &filter.value {
                        for series in series_vals {
                            book_filters = book_filters.with_series(series);
                        }
                    }
                }
                FilterField::BookFormat => {
                    if let FilterValue::Specific(formats) = &filter.value {
                        for format in formats {
                            book_filters = book_filters.with_format(format);
                        }
                    }
                }
                FilterField::BookYear => {
                    if let FilterValue::Specific(years) = &filter.value {
                        // BookFilters.year is a single exact-match value; use the first entry only.
                        if let Some(year_str) = years.first() {
                            if let Ok(year) = year_str.parse::<i32>() {
                                book_filters.year = Some(year);
                            }
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
                "BookAuthor" => FilterField::BookAuthor,
                "BookPublisher" => FilterField::BookPublisher,
                "BookSeries" => FilterField::BookSeries,
                "BookFormat" => FilterField::BookFormat,
                "BookYear" => FilterField::BookYear,
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
