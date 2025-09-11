use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PublisherRecord {
    pub id: i64,
    pub name: String,
    pub normalized_name: String,
    // altri campi: location, founded_year, ecc.
}

impl PublisherRecord {
    pub fn new(id: i64, name: &str) -> Self {
        let normalized_name = Self::normalize(name);
        Self {
            id,
            name: name.to_string(),
            normalized_name,
        }
    }

    pub fn normalize(name: &str) -> String {
        name.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "")
    }
}
