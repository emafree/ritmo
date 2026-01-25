use crate::entity_learner::VariantPatternType;
use strsim::levenshtein;

/// Classifies the pattern type between two strings based on their structure
pub fn default_classify_pattern_type(
    base: &str,
    variant: &str,
    edit_distance: usize,
) -> VariantPatternType {
    let base_lower = base.to_lowercase();
    let variant_lower = variant.to_lowercase();

    // Abbreviation: if variant is much shorter
    if variant_lower.len() < base_lower.len() / 2 {
        return VariantPatternType::Abbreviation;
    }

    // Prefix: variant starts the same then diverges
    if base_lower.starts_with(&variant_lower) || variant_lower.starts_with(&base_lower) {
        return VariantPatternType::Prefix;
    }

    // Suffix: variant ends the same but starts different
    if base_lower.ends_with(&variant_lower) || variant_lower.ends_with(&base_lower) {
        return VariantPatternType::Suffix;
    }

    // Compound: one contains the other completely
    if base_lower.contains(&variant_lower) || variant_lower.contains(&base_lower) {
        return VariantPatternType::Compound;
    }

    // Transliteration: similar length but different characters (e.g., accents)
    let len_diff = (base.len() as i32 - variant.len() as i32).abs();
    if len_diff <= 2 && edit_distance <= 3 {
        // Check if there are special/accented characters
        let has_special_base = base.chars().any(|c| !c.is_ascii());
        let has_special_variant = variant.chars().any(|c| !c.is_ascii());
        if has_special_base || has_special_variant {
            return VariantPatternType::Transliteration;
        }
    }

    // Typo: very small edit distance
    if edit_distance <= 2 {
        return VariantPatternType::Typo;
    }

    // Default
    VariantPatternType::Other
}

/// Calculates the confidence of a pattern based on various factors
pub fn default_confidence_function(
    base: &str,
    variant: &str,
    pattern_type: &VariantPatternType,
    jaro_winkler_similarity: f64,
) -> f64 {
    let mut confidence = jaro_winkler_similarity;

    // Bonus for specific patterns
    match pattern_type {
        VariantPatternType::Abbreviation => {
            // Common abbreviations have high confidence if initials match
            if are_initials_matching(base, variant) {
                confidence = (confidence + 0.2).min(1.0);
            }
        }
        VariantPatternType::Prefix | VariantPatternType::Suffix => {
            // Prefix/Suffix have good confidence
            confidence = (confidence + 0.1).min(1.0);
        }
        VariantPatternType::Typo => {
            // Typos have slightly reduced confidence
            confidence = (confidence - 0.05).max(0.0);
        }
        VariantPatternType::Transliteration => {
            // Transliteration has high confidence
            confidence = (confidence + 0.15).min(1.0);
        }
        VariantPatternType::Compound => {
            // Compounds are fairly safe
            confidence = (confidence + 0.1).min(1.0);
        }
        VariantPatternType::Other => {
            // No adjustment
        }
    }

    // Penalty for edit distance too high
    let edit_dist = levenshtein(base, variant);
    if edit_dist > 5 {
        confidence = (confidence - 0.1).max(0.0);
    }

    // Penalty for excessive length difference
    let len_diff_ratio =
        (base.len() as f64 - variant.len() as f64).abs() / base.len().max(1) as f64;
    if len_diff_ratio > 0.5 {
        confidence = (confidence - 0.15).max(0.0);
    }

    confidence.clamp(0.0, 1.0)
}

/// Checks if word initials match (for abbreviations)
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
        // Confidence may drop due to length difference penalty
        // which is > 50% (27 characters vs 15 characters)
        assert!(conf > 0.6); // Check that it's reasonable
        assert!(conf <= 1.0);
    }

    #[test]
    fn test_confidence_typo_penalty() {
        let conf =
            default_confidence_function("Tolkien", "Tolkein", &VariantPatternType::Typo, 0.9);
        assert!(conf < 0.9); // Penalty for typo
        assert!(conf > 0.8); // But not too low
    }
}
