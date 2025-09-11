use crate::people::parse_names::ParsedName;
use human_name::Name;
use strsim::levenshtein;
use unicode_normalization::UnicodeNormalization;

pub struct MLStringUtils {
    pub name_variants: std::collections::HashMap<String, Vec<String>>,
}

impl MLStringUtils {
    pub fn new(name_variants: std::collections::HashMap<String, Vec<String>>) -> Self {
        MLStringUtils { name_variants }
    }

    pub fn normalize_string(&self, text: &str) -> String {
        let normalized = text
            .nfc()
            .collect::<String>()
            .to_lowercase()
            .chars()
            .map(|c| match c {
                'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
                'è' | 'é' | 'ê' | 'ë' => 'e',
                'ì' | 'í' | 'î' | 'ï' => 'i',
                'ō' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
                'ù' | 'ú' | 'û' | 'ü' => 'u',
                'ç' => 'c',
                'ñ' => 'n',
                'ý' | 'ÿ' => 'y',
                'č' | 'ć' => 'c',
                'š' => 's',
                'ž' => 'z',
                'đ' => 'd',
                'ł' => 'l',
                c if c.is_alphabetic() || c.is_whitespace() => c,
                _ => ' ',
            })
            .collect::<String>();

        normalized
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn normalized_levenshtein(&self, s1: &str, s2: &str) -> f64 {
        let max_len = s1.len().max(s2.len()) as f64;
        if max_len == 0.0 {
            return 1.0;
        }
        1.0 - (levenshtein(s1, s2) as f64 / max_len)
    }

    pub fn are_known_variants(&self, name1: &str, name2: &str) -> bool {
        let norm1 = self.normalize_string(name1);
        let norm2 = self.normalize_string(name2);

        if let Some(variants) = self.name_variants.get(&norm1) {
            if variants.contains(&norm2) {
                return true;
            }
        }

        if let Some(variants) = self.name_variants.get(&norm2) {
            if variants.contains(&norm1) {
                return true;
            }
        }

        false
    }

    /// Parsing avanzato di un nome, compatibile con la logica precedente
    pub fn parse_name(input: &str) -> ParsedName {
        // Caso nome singolo (es: "Mozart")
        if input
            .split(|c: char| c.is_whitespace() || c == '.')
            .filter(|s| !s.is_empty())
            .count()
            == 1
        {
            return ParsedName {
                given_name: input.trim().to_string(),
                display_name: input.trim().to_string(),
                ..Default::default()
            };
        }

        // Usa la crate human-name per parsing avanzato
        let parsed = Name::parse(input);

        // Se il parsing fallisce, fallback a tutto input come display_name
        if let Some(p) = parsed {
            let given_name = p.given_name().unwrap_or("").to_string();
            let surname = p.surname().to_string();
            let middle_names: Vec<String> = p
                .middle_names()
                .map(|names| names.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default();
            let title = p.honorific_prefix().map(|s| s.to_string());
            let suffix = p.generational_suffix().map(|s| s.to_string());
            let display_name = p.display_first_last();
            ParsedName {
                given_name,
                surname,
                middle_names,
                title,
                suffix,
                display_name: display_name.to_string(),
            }
        } else {
            ParsedName {
                display_name: input.to_string(),
                ..Default::default()
            }
        }
    }
}
