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

/// Test that verifies the bug fix: "Corona, Mauro" vs "Mauro Corona" produce the SAME canonical key
/// This ensures that regardless of input format (comma-separated or natural order),
/// the canonical key is always generated in the correct order: given_name + middle_names + surname
#[test]
fn test_corona_mauro_normalization() {
    let normalizer = MLStringUtils::default();
    
    println!("\n\n=== TEST: Corona, Mauro Normalization ===\n");
    
    // Test case 1: Basic name with comma vs natural order
    let name1 = "Corona, Mauro";
    let name2 = "Mauro Corona";
    
    let record1 = PersonRecord::new(1, name1, &normalizer).expect("Failed to parse name1");
    let record2 = PersonRecord::new(2, name2, &normalizer).expect("Failed to parse name2");
    
    let key1 = record1.canonical_key();
    let key2 = record2.canonical_key();
    
    println!("Input 1: '{}' → Canonical Key: '{}'", name1, key1);
    println!("  Parsed: given='{}', middle={:?}, surname='{}'", 
        record1.parsed_name.given_name, 
        record1.parsed_name.middle_names, 
        record1.parsed_name.surname);
    
    println!("\nInput 2: '{}' → Canonical Key: '{}'", name2, key2);
    println!("  Parsed: given='{}', middle={:?}, surname='{}'", 
        record2.parsed_name.given_name, 
        record2.parsed_name.middle_names, 
        record2.parsed_name.surname);
    
    println!("\n✓ Result: Both keys are IDENTICAL: '{}' == '{}'", key1, key2);
    println!("  Deduplication will correctly identify these as the same person.\n");
    
    assert_eq!(key1, key2, 
        "Canonical keys must be identical for 'Corona, Mauro' and 'Mauro Corona'");
}

/// Test that verifies: "Corona, Mauro P." vs "Mauro P. Corona" produce the SAME canonical key
/// This ensures middle names/initials are handled correctly regardless of input format
#[test]
fn test_corona_mauro_p_normalization() {
    let normalizer = MLStringUtils::default();
    
    println!("\n\n=== TEST: Corona, Mauro P. Normalization ===\n");
    
    // Test case 2: Name with middle initial, comma vs natural order
    let name1 = "Corona, Mauro P.";
    let name2 = "Mauro P. Corona";
    
    let record1 = PersonRecord::new(1, name1, &normalizer).expect("Failed to parse name1");
    let record2 = PersonRecord::new(2, name2, &normalizer).expect("Failed to parse name2");
    
    let key1 = record1.canonical_key();
    let key2 = record2.canonical_key();
    
    println!("Input 1: '{}' → Canonical Key: '{}'", name1, key1);
    println!("  Parsed: given='{}', middle={:?}, surname='{}'", 
        record1.parsed_name.given_name, 
        record1.parsed_name.middle_names, 
        record1.parsed_name.surname);
    
    println!("\nInput 2: '{}' → Canonical Key: '{}'", name2, key2);
    println!("  Parsed: given='{}', middle={:?}, surname='{}'", 
        record2.parsed_name.given_name, 
        record2.parsed_name.middle_names, 
        record2.parsed_name.surname);
    
    println!("\n✓ Result: Both keys are IDENTICAL: '{}' == '{}'", key1, key2);
    println!("  Deduplication will correctly identify these as the same person.\n");
    
    assert_eq!(key1, key2, 
        "Canonical keys must be identical for 'Corona, Mauro P.' and 'Mauro P. Corona'");
}

/// Comprehensive test that verifies all Corona name variants produce the same canonical key
#[test]
fn test_all_corona_variants() {
    let normalizer = MLStringUtils::default();
    
    println!("\n\n=== TEST: All Corona Variants ===\n");
    
    let test_cases = vec![
        "Corona, Mauro",
        "Mauro Corona",
        "Corona, Mauro P.",
        "Mauro P. Corona",
    ];
    
    let mut keys = Vec::new();
    
    for (i, name) in test_cases.iter().enumerate() {
        match PersonRecord::new(i as i64 + 1, name, &normalizer) {
            Ok(record) => {
                let key = record.canonical_key();
                println!("Input {}: {:<25} → Key: '{}'", i + 1, name, key);
                println!("         Parsed: given='{}', middle={:?}, surname='{}'", 
                    record.parsed_name.given_name,
                    record.parsed_name.middle_names,
                    record.parsed_name.surname);
                keys.push(key);
            }
            Err(e) => {
                panic!("Failed to parse '{}': {}", name, e);
            }
        }
    }
    
    println!("\n=== Verification ===");
    
    // Check that all keys are identical
    let first_key = &keys[0];
    for (i, key) in keys.iter().enumerate().skip(1) {
        assert_eq!(first_key, key, 
            "All Corona variants must produce the same canonical key. '{}' != '{}'",
            first_key, key);
        println!("✓ '{}' == '{}' (variant {} matches variant 1)", 
            test_cases[0], test_cases[i], i + 1);
    }
    
    println!("\n✓ SUCCESS: All variants produce the same canonical key: '{}'", first_key);
    println!("  This confirms that deduplication will work correctly!\n");
}
