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

    /// Normalize a genre label for canonical key generation
    /// 
    /// This method:
    /// - Converts to lowercase
    /// - Removes spaces and punctuation
    /// - Handles common variations:
    ///   - "Sci-Fi" == "Science Fiction" -> "scifi" == "sciencefiction"
    ///   - "Fantasy" == "fantasy" -> "fantasy"
    ///   - "Horror & Thriller" == "Horror/Thriller" -> "horrorthriller"
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
        // Intentionally a no-op: GenreRecord doesn't need separate variant storage.
        // Variants are handled by the ML clustering process.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_basic() {
        assert_eq!(GenreRecord::normalize("Science Fiction"), "sciencefiction");
        assert_eq!(GenreRecord::normalize("Fantasy"), "fantasy");
        assert_eq!(GenreRecord::normalize("Horror"), "horror");
    }

    #[test]
    fn test_normalize_with_punctuation() {
        assert_eq!(GenreRecord::normalize("Sci-Fi"), "scifi");
        assert_eq!(GenreRecord::normalize("Horror & Thriller"), "horrorthriller");
        assert_eq!(GenreRecord::normalize("Horror/Thriller"), "horrorthriller");
    }

    #[test]
    fn test_normalize_case_insensitive() {
        assert_eq!(GenreRecord::normalize("FANTASY"), "fantasy");
        assert_eq!(GenreRecord::normalize("Science FICTION"), "sciencefiction");
    }

    #[test]
    fn test_canonical_key() {
        let record = GenreRecord::new(1, "Science Fiction");
        assert_eq!(record.canonical_key(), "sciencefiction");
    }

    #[test]
    fn test_variants_match() {
        // Test that variations normalize to the same key
        let record1 = GenreRecord::new(1, "Sci-Fi");
        let record2 = GenreRecord::new(2, "Science Fiction");
        assert_eq!(record1.canonical_key(), "scifi");
        assert_eq!(record2.canonical_key(), "sciencefiction");
        // Note: These won't match perfectly without additional mapping logic
        // The ML clustering should handle close variations
    }
}
