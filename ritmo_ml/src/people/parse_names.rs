use crate::utils::MLStringUtils;
use ritmo_errors::RitmoErr;
use human_name::Name;
use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[derive(PartialEq)]
pub struct ParsedName {
    pub given_name: String,
    pub surname: String,
    pub middle_names: Vec<String>,
    pub title: Option<String>,
    pub suffix: Option<String>,
    pub display_name: String,
}

impl ParsedName {
    pub fn from_string(input: &str) -> Result<Self, RitmoErr> {
        // Gestione caso singola parola
        if input.split(|c: char| c.is_whitespace() || c == '.')
           .filter(|s| !s.is_empty())
           .count() == 1 {
            return Ok(ParsedName {
                given_name: input.trim().to_string(),
                display_name: input.trim().to_string(),
                ..Default::default()
            });
        }

        let pparsed = Name::parse(input);
        if let Some(parsed) = pparsed {
            let given_name = parsed.given_name().unwrap_or("").to_string();
            let surname = parsed.surname().to_string();
            let middle_names: Vec<String> = parsed.middle_names()
                .map(|names| names.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default();
            let title = parsed.honorific_prefix().map(|s| s.to_string());
            let suffix = parsed.generational_suffix().map(|s| s.to_string());
            let display_name = parsed.display_first_last();

            Ok(ParsedName {
                given_name,
                surname,
                middle_names,
                title,
                suffix,
                display_name: display_name.to_string(),
            })
        }
        else {
            Ok(ParsedName {
                display_name: input.to_string(),
                ..Default::default()
            })
        }
    }

    pub fn to_normalized_key(&self, normalizer: &MLStringUtils) -> String {
        let mut full_name_parts = Vec::new();
        
        if !self.given_name.is_empty() {
            full_name_parts.push(self.given_name.as_str());
        }
        
        for middle_name in &self.middle_names {
            if !middle_name.is_empty() {
                full_name_parts.push(middle_name.as_str());
            }
        }
        
        if !self.surname.is_empty() {
            full_name_parts.push(self.surname.as_str());
        }
        
        let combined_name = full_name_parts.join(" ");
        normalizer.normalize_string(&combined_name)
    }
}