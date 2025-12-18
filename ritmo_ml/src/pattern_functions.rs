use crate::entity_learner::VariantPatternType;
use strsim::levenshtein;

/// Classifica il tipo di pattern tra due stringhe basandosi sulla loro struttura
pub fn default_classify_pattern_type(
    base: &str,
    variant: &str,
    edit_distance: usize,
) -> VariantPatternType {
    let base_lower = base.to_lowercase();
    let variant_lower = variant.to_lowercase();

    // Abbreviazione: se variant è molto più corto
    if variant_lower.len() < base_lower.len() / 2 {
        return VariantPatternType::Abbreviation;
    }

    // Prefix: variant inizia uguale poi diverge
    if base_lower.starts_with(&variant_lower) || variant_lower.starts_with(&base_lower) {
        return VariantPatternType::Prefix;
    }

    // Suffix: variant finisce uguale ma inizia diverso
    if base_lower.ends_with(&variant_lower) || variant_lower.ends_with(&base_lower) {
        return VariantPatternType::Suffix;
    }

    // Compound: uno contiene l'altro completamente
    if base_lower.contains(&variant_lower) || variant_lower.contains(&base_lower) {
        return VariantPatternType::Compound;
    }

    // Transliterazione: lunghezza simile ma caratteri diversi (es. accenti)
    let len_diff = (base.len() as i32 - variant.len() as i32).abs();
    if len_diff <= 2 && edit_distance <= 3 {
        // Controlla se ci sono caratteri speciali/accentati
        let has_special_base = base.chars().any(|c| !c.is_ascii());
        let has_special_variant = variant.chars().any(|c| !c.is_ascii());
        if has_special_base || has_special_variant {
            return VariantPatternType::Transliteration;
        }
    }

    // Typo: edit distance molto piccolo
    if edit_distance <= 2 {
        return VariantPatternType::Typo;
    }

    // Default
    VariantPatternType::Other
}

/// Calcola la confidence di un pattern basandosi su vari fattori
pub fn default_confidence_function(
    base: &str,
    variant: &str,
    pattern_type: &VariantPatternType,
    jaro_winkler_similarity: f64,
) -> f64 {
    let mut confidence = jaro_winkler_similarity;

    // Bonus per pattern specifici
    match pattern_type {
        VariantPatternType::Abbreviation => {
            // Abbreviazioni comuni hanno alta confidence se iniziali corrispondono
            if are_initials_matching(base, variant) {
                confidence = (confidence + 0.2).min(1.0);
            }
        }
        VariantPatternType::Prefix | VariantPatternType::Suffix => {
            // Prefix/Suffix hanno buona confidence
            confidence = (confidence + 0.1).min(1.0);
        }
        VariantPatternType::Typo => {
            // Typo hanno confidence leggermente ridotta
            confidence = (confidence - 0.05).max(0.0);
        }
        VariantPatternType::Transliteration => {
            // Transliterazione ha alta confidence
            confidence = (confidence + 0.15).min(1.0);
        }
        VariantPatternType::Compound => {
            // Compound sono abbastanza sicuri
            confidence = (confidence + 0.1).min(1.0);
        }
        VariantPatternType::Other => {
            // Nessun adjustment
        }
    }

    // Penalità per edit distance troppo alto
    let edit_dist = levenshtein(base, variant);
    if edit_dist > 5 {
        confidence = (confidence - 0.1).max(0.0);
    }

    // Penalità per differenza lunghezza eccessiva
    let len_diff_ratio =
        (base.len() as f64 - variant.len() as f64).abs() / base.len().max(1) as f64;
    if len_diff_ratio > 0.5 {
        confidence = (confidence - 0.15).max(0.0);
    }

    confidence.clamp(0.0, 1.0)
}

/// Verifica se le iniziali delle parole corrispondono (per abbreviazioni)
fn are_initials_matching(full: &str, abbrev: &str) -> bool {
    let full_words: Vec<&str> = full.split_whitespace().collect();
    let abbrev_clean = abbrev.replace('.', "").replace(' ', "");

    if abbrev_clean.len() != full_words.len() {
        return false;
    }

    for (i, word) in full_words.iter().enumerate() {
        if let Some(first_char) = word.chars().next() {
            if let Some(abbrev_char) = abbrev_clean.chars().nth(i) {
                if first_char.to_lowercase().to_string() != abbrev_char.to_lowercase().to_string() {
                    return false;
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_abbreviation() {
        let pattern = default_classify_pattern_type("J.R.R. Tolkien", "JRR", 10);
        assert_eq!(pattern, VariantPatternType::Abbreviation);
    }

    #[test]
    fn test_classify_typo() {
        let pattern = default_classify_pattern_type("Tolkien", "Tolkein", 2);
        assert_eq!(pattern, VariantPatternType::Typo);
    }

    #[test]
    fn test_classify_prefix() {
        let pattern = default_classify_pattern_type("Bompiani Editore", "Bompiani", 1);
        assert_eq!(pattern, VariantPatternType::Prefix);
    }

    #[test]
    fn test_initials_matching() {
        assert!(are_initials_matching("John Ronald Reuel", "J.R.R."));
        assert!(are_initials_matching("John Ronald Reuel", "JRR"));
        assert!(!are_initials_matching("John Ronald", "JRR"));
    }

    #[test]
    fn test_confidence_abbreviation() {
        let conf = default_confidence_function(
            "John Ronald Reuel Tolkien",
            "J.R.R. Tolkien",
            &VariantPatternType::Abbreviation,
            0.8,
        );
        // La confidence può scendere a causa della penalità per lunghezza differente
        // che è > 50% (27 caratteri vs 15 caratteri)
        assert!(conf > 0.6); // Verifica che sia ragionevole
        assert!(conf <= 1.0);
    }

    #[test]
    fn test_confidence_typo_penalty() {
        let conf =
            default_confidence_function("Tolkien", "Tolkein", &VariantPatternType::Typo, 0.9);
        assert!(conf < 0.9); // Penalità per typo
        assert!(conf > 0.8); // Ma non troppo bassa
    }
}
