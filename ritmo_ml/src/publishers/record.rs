use crate::traits::MLProcessable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PublisherRecord {
    pub id: i64,
    pub name: String,
    pub normalized_name: String,
    pub variants: Vec<String>,
}

impl PublisherRecord {
    pub fn new(id: i64, name: &str) -> Self {
        let normalized_name = Self::normalize(name);
        Self {
            id,
            name: name.to_string(),
            normalized_name,
            variants: vec![name.to_string()],
        }
    }

    pub fn normalize(name: &str) -> String {
        name.to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "")
    }
}

impl MLProcessable for PublisherRecord {
    fn id(&self) -> i64 {
        self.id
    }

    fn canonical_key(&self) -> String {
        self.normalized_name.clone()
    }

    fn variants(&self) -> Vec<String> {
        self.variants.clone()
    }

    fn set_variants(&mut self, variants: Vec<String>) {
        self.variants = variants;
    }
}
