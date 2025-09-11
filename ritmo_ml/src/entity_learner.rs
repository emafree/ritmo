use std::collections::HashMap;
use strsim::jaro_winkler;

/// Tipo di pattern generico
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum VariantPatternType {
    Suffix,
    Prefix,
    Transliteration,
    Abbreviation,
    Compound,
    Typo,
    Other,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EntityVariantPattern {
    pub base_form: String,
    pub variant_form: String,
    pub pattern_type: VariantPatternType,
    pub confidence: f64,
    pub frequency: usize,
    pub edit_distance: usize,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EntityCluster {
    pub centroid: String,
    pub members: Vec<String>,
    pub confidence: f64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MLEntityLearner {
    pub clusters: Vec<EntityCluster>,
    pub learned_patterns: Vec<EntityVariantPattern>,
    pub pattern_frequency: HashMap<String, usize>,
    pub minimum_confidence: f64,
    pub minimum_frequency: usize,
}

impl MLEntityLearner {
    pub fn new() -> Self {
        Self {
            clusters: Vec::new(),
            learned_patterns: Vec::new(),
            pattern_frequency: HashMap::new(),
            minimum_confidence: 0.85,
            minimum_frequency: 3,
        }
    }

    pub fn create_clusters(&mut self, items: &[String]) {
        let mut clusters = Vec::new();
        let mut used = vec![false; items.len()];
        let threshold = 0.85;
        for (i, a) in items.iter().enumerate() {
            if used[i] { continue; }
            let mut group = vec![a.clone()];
            used[i] = true;
            for (j, b) in items.iter().enumerate().skip(i + 1) {
                if !used[j] && jaro_winkler(a, b) > threshold {
                    group.push(b.clone());
                    used[j] = true;
                }
            }
            if group.len() > 1 {
                let centroid = Self::find_centroid(&group);
                let confidence = Self::calc_group_confidence(&group);
                clusters.push(EntityCluster { centroid, members: group, confidence });
            }
        }
        self.clusters = clusters;
    }

    pub fn identify_variant_patterns(
        &mut self,
        classify_fn: &dyn Fn(&str, &str, usize) -> VariantPatternType,
        confidence_fn: &dyn Fn(&str, &str, &VariantPatternType, f64) -> f64,
    ) {
        for cluster in &self.clusters {
            for i in 0..cluster.members.len() {
                for j in (i + 1)..cluster.members.len() {
                    let a = &cluster.members[i];
                    let b = &cluster.members[j];
                    let edit_dist = strsim::levenshtein(a, b);
                    let sim = jaro_winkler(a, b);
                    if sim < 0.7 { continue; }
                    let pattern_type = classify_fn(a, b, edit_dist);
                    let confidence = confidence_fn(a, b, &pattern_type, sim);
                    let pattern_key = format!("{:?}->{:?}", pattern_type, &a);
                    *self.pattern_frequency.entry(pattern_key.clone()).or_insert(0) += 1;
                    if confidence >= self.minimum_confidence && self.pattern_frequency[&pattern_key] >= self.minimum_frequency {
                        self.learned_patterns.push(EntityVariantPattern {
                            base_form: a.to_string(),
                            variant_form: b.to_string(),
                            pattern_type,
                            confidence,
                            frequency: 1,
                            edit_distance: edit_dist,
                        });
                    }
                }
            }
        }
        self.learned_patterns.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        self.learned_patterns.dedup_by(|a, b| a.base_form == b.base_form && a.variant_form == b.variant_form);
    }

    fn find_centroid(group: &[String]) -> String {
        let mut min_dist_sum = std::f64::MAX;
        let mut centroid = group[0].clone();
        for candidate in group {
            let sum: f64 = group.iter().map(|other| 1.0 - jaro_winkler(candidate, other)).sum();
            if sum < min_dist_sum {
                min_dist_sum = sum;
                centroid = candidate.clone();
            }
        }
        centroid
    }

    fn calc_group_confidence(group: &[String]) -> f64 {
        if group.len() < 2 { return 1.0; }
        let mut sum = 0.0;
        let mut count = 0;
        for (i, x) in group.iter().enumerate() {
            for y in group.iter().skip(i + 1) {
                sum += jaro_winkler(x, y);
                count += 1;
            }
        }
        if count > 0 { sum / (count as f64) } else { 1.0 }
    }
}