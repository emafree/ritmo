use crate::people::parse_names::ParsedName;
use crate::utils::MLStringUtils;
use ritmo_errors::RitmoErr;
use crate::traits::MLProcessable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersonRecord {
    pub id: i64,
    pub original_input: String,
    pub parsed_name: ParsedName,
    pub normalized_key: String,
    pub confidence: f64,
    pub verified: bool,
//    pub aliases: Vec<String>,
}

impl PersonRecord {
    pub fn new(id: i64, input: &str, normalizer: &MLStringUtils) -> Result<Self, RitmoErr> {
        let parsed_name = ParsedName::from_string(input)?;
        let normalized_key = parsed_name.to_normalized_key(normalizer);
        
        Ok(PersonRecord {
            id,
            original_input: input.to_string(),
            parsed_name,
            normalized_key,
            confidence: 1.0,
            verified: false,
//            aliases: Vec::new(),
        })
    }
    
//    pub fn add_alias(&mut self, alias: &str) -> Result<(), RitmoErr> {
//        let normalized_alias = create_normalized_key_from_string(alias);
//        if !self.aliases.contains(&normalized_alias) {
//            self.aliases.push(normalized_alias);
//        }
//        Ok(())
//    }

    pub fn update_confidence(&mut self, new_confidence: f64) {
        self.confidence = new_confidence.clamp(0.0, 1.0);
    }
    
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.85
    }
    
    pub fn needs_verification(&self) -> bool {
        !self.verified && self.confidence < 0.90
    }

    pub fn all_canonical_keys(&self) -> Vec<String> {
        let keys = vec![self.normalized_key.clone()];
//        keys.extend(self.aliases.clone());
        keys
    }
}

impl MLProcessable for PersonRecord {
    fn id(&self) -> i64 {
        self.id
    }

    fn canonical_key(&self) -> std::string::String {
        self.normalized_key.clone()
    }

    fn variants(&self) -> Vec<String> {
        // In un'implementazione reale, questi potrebbero essere caricati dal DB
//        vec![self.name.clone()]
        Vec::new()
    }

    fn set_variants(&mut self, variants: Vec<String>) {
        // In un'implementazione reale, salveresti sul DB
        println!("Aggiornando varianti per {}: {:?}", self.original_input, variants);
    }
}
