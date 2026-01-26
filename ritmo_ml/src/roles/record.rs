use crate::traits::MLProcessable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RoleRecord {
    pub id: i64,
    pub name: String,
    pub normalized_name: String,
}

impl RoleRecord {
    pub fn new(id: i64, name: &str) -> Self {
        let normalized_name = Self::normalize(name);
        Self {
            id,
            name: name.to_string(),
            normalized_name,
        }
    }

    pub fn normalize(name: &str) -> String {
        name.to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "")
    }
}

// Implementazione del trait MLProcessable per RoleRecord
impl MLProcessable for RoleRecord {
    fn id(&self) -> i64 {
        self.id
    }

    fn canonical_key(&self) -> String {
        self.normalized_name.clone()
    }

    fn variants(&self) -> Vec<String> {
        vec![self.name.clone()]
    }

    fn set_variants(&mut self, _variants: Vec<String>) {
        // Variants are set but RoleRecord doesn't store them separately
        // This is a no-op implementation for roles
    }
}
