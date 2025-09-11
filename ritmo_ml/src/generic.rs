use crate::feedback::Feedback;
use crate::traits::MLProcessable;
use std::collections::{HashMap, HashSet};

/// Raggruppa entità per chiave canonica e aggiorna le varianti di ciascun gruppo.
/// (Esempio: deduplica nomi, editori, tag, ecc. usando la stessa logica)
pub fn cluster_by_canonical_key<T: MLProcessable>(records: &mut [T]) {
    // Raggruppa gli ID delle entità che condividono la stessa chiave canonica
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();

    for (idx, record) in records.iter().enumerate() {
        groups
            .entry(record.canonical_key().to_owned())
            .or_insert_with(Vec::new)
            .push(idx);
    }

    // Per ogni gruppo, unisci tutte le varianti e aggiorna ogni entità del gruppo
    for indices in groups.values() {
        let mut all_variants = Vec::new();
        for &idx in indices.iter() {
            all_variants.extend(records[idx].variants());
        }
        all_variants.sort();
        all_variants.dedup();

        // Applica le varianti deduplicate a tutti i membri del gruppo
        for &idx in indices.iter() {
            records[idx].set_variants(all_variants.clone());
        }
    }
}

/// Deduplica entità usando una funzione di similarità fuzzy tra le chiavi canoniche.
/// Unisce le varianti dei record simili.
pub fn deduplicate_fuzzy<T, F>(records: &mut [T], is_similar: F)
where
    T: MLProcessable,
    F: Fn(&str, &str) -> bool,
{
    let mut merged_indices = vec![false; records.len()];

    for i in 0..records.len() {
        if merged_indices[i] {
            continue;
        }
        let mut variants = records[i].variants();
        for j in (i + 1)..records.len() {
            if !merged_indices[j]
                && is_similar(&records[i].canonical_key().as_str(), &records[j].canonical_key().as_str())
            {
                variants.extend(records[j].variants());
                merged_indices[j] = true;
            }
        }
        variants.sort();
        variants.dedup();
        records[i].set_variants(variants);
    }
}

/// Rimuove dalle varianti i match esplicitamente esclusi dal feedback.
pub fn apply_negative_feedback<T: MLProcessable>(
    records: &mut [T],
    forbidden_pairs: &HashSet<(String, String)>,
) {
    for record in records.iter_mut() {
        let filtered: Vec<String> = record
            .variants()
            .into_iter()
            .filter(|v| {
                !forbidden_pairs.contains(&(record.canonical_key().to_owned(), v.to_owned()))
            })
            .collect();
        record.set_variants(filtered);
    }
}

/// Unisce le varianti di entità che hanno feedback positivo (da considerare equivalenti)
pub fn apply_positive_feedback<T: MLProcessable>(records: &mut [T], feedback: &Feedback) {
    // Mappa da canonical_key a indices
    use std::collections::HashMap;
    let mut key_to_indices: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, r) in records.iter().enumerate() {
        key_to_indices
            .entry(r.canonical_key().to_string())
            .or_default()
            .push(i);
    }
    // Per ogni coppia positiva, unisci le varianti
    for (a, b) in feedback.positive_pairs.iter() {
        if let (Some(ai), Some(bi)) = (key_to_indices.get(a), key_to_indices.get(b)) {
            let mut all_variants = vec![];
            for &idx in ai.iter().chain(bi.iter()) {
                all_variants.extend(records[idx].variants());
            }
            all_variants.sort();
            all_variants.dedup();
            for &idx in ai.iter().chain(bi.iter()) {
                records[idx].set_variants(all_variants.clone());
            }
        }
    }
}

/// Restituisce le coppie di indici con score di similarità superiore a una certa soglia
pub fn find_similar_pairs<T, F>(
    records: &[T],
    similarity: F,
    threshold: f64,
) -> Vec<(usize, usize, f64)>
where
    T: MLProcessable,
    F: Fn(&str, &str) -> f64,
{
    let mut result = Vec::new();
    for i in 0..records.len() {
        for j in (i + 1)..records.len() {
            let score = similarity(records[i].canonical_key().as_str(), records[j].canonical_key().as_str());
            if score >= threshold {
                result.push((i, j, score));
            }
        }
    }
    result
}
