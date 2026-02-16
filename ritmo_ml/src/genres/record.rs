use crate::traits::MLProcessable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GenreRecord {
    pub id: i64,
    pub label: String,
    pub normalized_label: String,
}

impl GenreRecord {
    pub fn new(id: i64, label: &str) -> Self {
        let normalized_label = Self::normalize(label);
        Self {
            id,
            label: label.to_string(),
            normalized_label,
        }
    }

    pub fn normalize(label: &str) -> String {
        label
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "")
    }
}

// Implementation of the MLProcessable trait for GenreRecord
impl MLProcessable for GenreRecord {
    fn id(&self) -> i64 {
        self.id
    }

    fn canonical_key(&self) -> String {
        self.normalized_label.clone()
    }

    fn variants(&self) -> Vec<String> {
        vec![self.label.clone()]
    }

    fn set_variants(&mut self, _variants: Vec<String>) {
        // Variants are set but GenreRecord doesn't store them separately
        // This is a no-op implementation for genres
    }
}
