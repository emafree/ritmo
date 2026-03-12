use std::path::PathBuf;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let schema_dir = PathBuf::from(&manifest_dir)
        .parent()
        .expect("CARGO_MANIFEST_DIR must have a parent directory")
        .join("ritmo_db")
        .join("schema");

    let schema_path = schema_dir.join("schema.sql");
    let seed_path = schema_dir.join("seed_page_fields.sql");

    println!("cargo:rerun-if-changed={}", schema_path.display());
    println!("cargo:rerun-if-changed={}", seed_path.display());
    println!("cargo:rerun-if-changed=build.rs");

    let schema_sql = std::fs::read_to_string(&schema_path)
        .unwrap_or_else(|e| panic!("Failed to read schema.sql from {}: {}", schema_path.display(), e));

    let seed_sql = std::fs::read_to_string(&seed_path)
        .unwrap_or_else(|e| panic!("Failed to read seed_page_fields.sql from {}: {}", seed_path.display(), e));

    // Write the regenerated database to OUT_DIR
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let output_path = PathBuf::from(&out_dir).join("template.db");

    // Create a new in-memory SQLite database and execute the schema
    let mem_conn = rusqlite::Connection::open_in_memory()
        .unwrap_or_else(|e| panic!("Failed to open in-memory SQLite database: {}", e));

    mem_conn
        .execute_batch(&schema_sql)
        .unwrap_or_else(|e| panic!("Failed to execute schema.sql: {}", e));

    // Seed static page_fields data at build-time
    mem_conn
        .execute_batch(&seed_sql)
        .unwrap_or_else(|e| panic!("Failed to execute seed_page_fields.sql: {}", e));

    // Serialize the in-memory database to bytes and write to the output file
    let db_bytes = mem_conn
        .serialize(rusqlite::DatabaseName::Main)
        .unwrap_or_else(|e| panic!("Failed to serialize in-memory database: {}", e));

    std::fs::write(&output_path, &*db_bytes)
        .unwrap_or_else(|e| panic!("Failed to write template.db to {}: {}", output_path.display(), e));

    println!("cargo:rustc-env=RITMO_TEMPLATE_DB_PATH={}", output_path.display());
    println!("cargo:warning=ritmo_db_core: generated template.db at {}", output_path.display());
}
