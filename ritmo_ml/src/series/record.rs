use crate::traits::MLProcessable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SeriesRecord {
    pub id: i64,
    pub title: String,
    pub normalized_title: String,
    pub variants: Vec<String>,
}

impl SeriesRecord {
    pub fn new(id: i64, title: &str) -> Self {
        let normalized_title = Self::normalize(title);
        Self {
            id,
            title: title.to_string(),
            normalized_title,
            variants: vec![title.to_string()],
        }
    }

    pub fn normalize(title: &str) -> String {
        title
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "")
    }
}

impl MLProcessable for SeriesRecord {
    fn id(&self) -> i64 {
        self.id
    }

    fn canonical_key(&self) -> String {
        self.normalized_title.clone()
    }

    fn variants(&self) -> Vec<String> {
        self.variants.clone()
    }

    fn set_variants(&mut self, variants: Vec<String>) {
        self.variants = variants;
    }
}
