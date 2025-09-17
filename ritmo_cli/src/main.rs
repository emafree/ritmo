use ritmo_db_core::LibraryConfig;
use tokio; // assicurati che tokio sia tra le dependency nel tuo Cargo.toml

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Crea una nuova configurazione con la directory radice desiderata
    let config = LibraryConfig::new("/home/ema/mylib_async");

    // 2. Inizializza tutte le directory necessarie
    config.initialize()?;

    // 3. Inizializza il database (se non esiste, copia dal template)
    match config.initialize_database().await {
        Ok(_) => println!("Database inizializzato correttamente."),
        Err(e) => println!("Errore nell'inizializzazione del database: {:?}", e),
    }

    dbg!(&config);

    // 4. Verifica che tutte le directory siano state create
    if config.validate()? {
        println!("Tutte le directory sono pronte!");
    } else {
        println!("Ci sono problemi con le directory.");
    }

    // 5. Esegui un health check completo (controlla anche database e permessi)
    let issues = config.health_check();
    if issues.is_empty() {
        println!("Configurazione sana!");
    } else {
        println!("Problemi rilevati:");
        for issue in issues {
            println!("- {}", issue);
        }
    }

    // 6. Salva la configurazione su file TOML
    let mut save_str = config.canonical_config_path();
    save_str.push("ritmo_toml");
    dbg!(&save_str);
    config.save(save_str)?;
    println!("Configurazione salvata!");

    // 7. Crea il pool di connessioni asincrono
    match config.create_pool().await {
        Ok(pool) => {
            println!("Pool di connessioni creato con successo!");
            // Qui puoi usare 'pool' per accedere al database con sqlx
        }
        Err(e) => println!("Errore nella creazione del pool: {:?}", e),
    }

    Ok(())
}
