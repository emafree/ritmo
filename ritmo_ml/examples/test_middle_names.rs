use ritmo_ml::people::parse_names::ParsedName;
use ritmo_ml::utils::MLStringUtils;

fn main() {
    let normalizer = MLStringUtils::default();
    
    println!("\n=== TESTING MIDDLE NAME PARSING ===\n");
    
    let test_cases = vec![
        "Mauro P. Corona",
        "Stephen E. King",
        "Stephen Edwin King",
        "J.K. Rowling",
        "J. K. Rowling",
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
}
