use ritmo_ml::people::parse_names::ParsedName;
use ritmo_ml::people::record::PersonRecord;
use ritmo_ml::traits::MLProcessable;
use ritmo_ml::utils::MLStringUtils;

fn main() {
    let normalizer = MLStringUtils::default();
    
    println!("\n=== DIRECT PARSING TEST ===\n");
    
    let test_cases = vec![
        "Corona, Mauro",
        "Mauro Corona", 
        "Corona, Mauro P.",
        "Mauro P. Corona",
    ];
    
    for name in &test_cases {
        match ParsedName::from_string(name) {
            Ok(parsed) => {
                println!("Input: '{}'", name);
                println!("  Given: '{}', Middle: {:?}, Surname: '{}'", 
                    parsed.given_name, parsed.middle_names, parsed.surname);
                let key = parsed.to_normalized_key(&normalizer);
                println!("  Canonical Key: '{}'\n", key);
            }
            Err(e) => {
                println!("Input: '{}' → ERROR: {}\n", name, e);
            }
        }
    }
    
    println!("\n=== USING PersonRecord ===\n");
    
    let mut id = 1;
    let mut records = Vec::new();
    for name in &test_cases {
        match PersonRecord::new(id, name, &normalizer) {
            Ok(record) => {
                println!("Input: '{}' → Key: '{}'", name, record.canonical_key());
                records.push((name, record.canonical_key()));
                id += 1;
            }
            Err(e) => {
                println!("Input: '{}' → ERROR: {}", name, e);
            }
        }
    }
    
    println!("\n=== KEY COMPARISON ===\n");
    for i in 0..records.len() {
        for j in (i+1)..records.len() {
            let (name1, key1) = &records[i];
            let (name2, key2) = &records[j];
            if key1 == key2 {
                println!("✓ MATCH: '{}' == '{}'", name1, name2);
                println!("  Keys: '{}' == '{}'", key1, key2);
            } else {
                println!("✗ DIFFERENT: '{}' != '{}'", name1, name2);
                println!("  Keys: '{}' != '{}'", key1, key2);
            }
        }
    }
}
