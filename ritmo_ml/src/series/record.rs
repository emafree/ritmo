use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SeriesRecord {
    pub id: i64,
    pub title: String,
    pub normalized_title: String,
    // altri campi: year, description, ecc.
}

impl SeriesRecord {
    pub fn new(id: i64, title: &str) -> Self {
        let normalized_title = Self::normalize(title);
        Self {
            id,
            title: title.to_string(),
            normalized_title,
        }
    }

    pub fn normalize(title: &str) -> String {
        title.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "")
    }
}
