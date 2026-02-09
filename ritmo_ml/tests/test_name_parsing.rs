// Test per verificare come vengono parsati e normalizzati i nomi
use ritmo_ml::people::record::PersonRecord;
use ritmo_ml::traits::MLProcessable;
use ritmo_ml::utils::MLStringUtils;
use strsim::jaro_winkler;

#[test]
fn test_name_canonical_keys() {
    let normalizer = MLStringUtils::default();

    let test_cases = vec![
        ("Stephen King", 1),
        ("Stephen Edwin King", 2),
        ("Stephen E. King", 3),
        ("King, Stephen", 4),
        ("J.K. Rowling", 5),
        ("J. K. Rowling", 6),
        ("Joanne K. Rowling", 7),
        ("Isaac Asimov", 8),
        ("Asimov, Isaac", 9),
        ("I. Asimov", 10),
    ];

    println!("\n\n=== NAME PARSING AND CANONICAL KEYS ===\n");

    let mut keys = Vec::new();
    for (name, id) in &test_cases {
        match PersonRecord::new(*id, name, &normalizer) {
            Ok(record) => {
                let key = record.canonical_key();
                println!("Input: {:<25} → Canonical: {}", name, key);
                keys.push((name, key));
            }
            Err(e) => {
                println!("Input: {:<25} → ERROR: {}", name, e);
            }
        }
    }

    println!("\n\n=== JARO-WINKLER SIMILARITIES ===\n");

    for (i, (name1, key1)) in keys.iter().enumerate() {
        for (name2, key2) in keys.iter().skip(i + 1) {
            let sim = jaro_winkler(key1, key2);
            if sim > 0.70 {
                println!("{:<25} vs {:<25}", name1, name2);
                println!("  Keys: {:<25} vs {:<25}", key1, key2);
                println!("  Similarity: {:.4}\n", sim);
            }
        }
    }

    println!("\n=== SIMILARITY GROUPINGS ===\n");
    println!("Threshold 0.85:");
    for (i, (name1, key1)) in keys.iter().enumerate() {
        let mut group = vec![*name1];
        for (name2, key2) in keys.iter().skip(i + 1) {
            if jaro_winkler(key1, key2) > 0.85 {
                group.push(*name2);
            }
        }
        if group.len() > 1 {
            println!("  Group: {:?}", group);
        }
    }

    println!("\nThreshold 0.75:");
    for (i, (name1, key1)) in keys.iter().enumerate() {
        let mut group = vec![*name1];
        for (name2, key2) in keys.iter().skip(i + 1) {
            if jaro_winkler(key1, key2) > 0.75 {
                group.push(*name2);
            }
        }
        if group.len() > 1 {
            println!("  Group: {:?}", group);
        }
    }
}
