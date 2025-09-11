use std::collections::HashSet;

pub struct Feedback {
    // Coppie di chiavi canoniche da NON unire
    pub negative_pairs: HashSet<(String, String)>,
    // (Opzionale) Coppie di chiavi canoniche da forzare come equivalenti
    pub positive_pairs: HashSet<(String, String)>,
}

impl Feedback {
    pub fn new() -> Self {
        Self {
            negative_pairs: HashSet::new(),
            positive_pairs: HashSet::new(),
        }
    }
    pub fn add_negative(&mut self, a: &str, b: &str) {
        self.negative_pairs.insert((a.to_owned(), b.to_owned()));
        self.negative_pairs.insert((b.to_owned(), a.to_owned())); // simmetrico
    }
    pub fn add_positive(&mut self, a: &str, b: &str) {
        self.positive_pairs.insert((a.to_owned(), b.to_owned()));
        self.positive_pairs.insert((b.to_owned(), a.to_owned()));
    }
}