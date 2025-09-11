use ritmo_db::models::Book;
use chrono::Utc;

#[test]
fn test_set_book_persistence() {
    // Creiamo un libro di test con valori noti per verificare la generazione dell'hash
    let mut book = Book {
        id: None,
        name: "Il Nome della Rosa".to_string(),
        original_title: Some("The Name of the Rose".to_string()),
        publisher_id: Some(42),
        format_id: Some(1),
        series_id: Some(7),
        series_index: Some(1),
        publication_date: Some(315532800), // 1980-01-01
        last_modified_date: Utc::now().timestamp(),
        isbn: Some("9788845292866".to_string()),
        pages: Some(512),
        notes: Some("Note di test".to_string()),
        has_cover: 1,
        has_paper: 1,
        file_link: None,
        file_size: None,
        file_hash: None,
        created_at: 1691747760, // Fissiamo un timestamp noto per avere un hash deterministico
    };
    
    // Chiamiamo la funzione che vogliamo testare
    book.set_book_persistence();
    
    // Verifichiamo che l'hash sia stato generato
    assert!(book.file_hash.is_some(), "L'hash del file non è stato generato");
    
    // Verifichiamo che il file link sia stato impostato
    assert!(book.file_link.is_some(), "Il link al file non è stato impostato");
    
    let hash = book.file_hash.clone().unwrap();
    let file_link = book.file_link.clone().unwrap();
    
    // Verifichiamo che l'hash abbia la lunghezza corretta (32 caratteri per 16 byte in hex)
    assert_eq!(hash.len(), 32, "La lunghezza dell'hash non è corretta");
    
    // Verifichiamo che il file link abbia il formato corretto
    let expected_prefix = format!("books/{}/{}/", &hash[0..2], &hash[2..4]);
    assert!(file_link.starts_with(&expected_prefix), 
            "Il file link non ha la struttura di directory corretta");
    
    // Verifichiamo che il file link contenga l'hash completo
    assert!(file_link.contains(&hash), 
            "Il file link non contiene l'hash completo");
    
    // Verifichiamo che l'estensione sia corretta
    assert!(file_link.ends_with(".epub"), 
            "Il file link non ha l'estensione corretta");
    
    // Verifichiamo che il file size sia stato inizializzato a 0
    assert_eq!(book.file_size, Some(0), 
               "La dimensione del file non è stata inizializzata a 0");
    
    // Test di idempotenza: eseguiamo nuovamente la funzione e verifichiamo che l'hash non cambi
    let hash_before = book.file_hash.clone();
    let link_before = book.file_link.clone();
    
    book.set_book_persistence();
    
    assert_eq!(book.file_hash, hash_before, 
               "L'hash è cambiato dopo la seconda chiamata della funzione");
    assert_eq!(book.file_link, link_before, 
               "Il file link è cambiato dopo la seconda chiamata della funzione");
}

#[test]
fn test_set_book_persistence_with_different_books() {
    // Creiamo due libri con metadati differenti
    let mut book1 = Book {
        id: None,
        name: "Il Nome della Rosa".to_string(),
        original_title: Some("The Name of the Rose".to_string()),
        isbn: Some("9788845292866".to_string()),
        created_at: 1691747760,
        last_modified_date: Utc::now().timestamp(),
        has_cover: 0,
        has_paper: 0,
        ..Default::default()
    };
    
    let mut book2 = Book {
        id: None,
        name: "1984".to_string(),
        original_title: Some("1984".to_string()),
        isbn: Some("9788804144663".to_string()),
        created_at: 1691747760,
        last_modified_date: Utc::now().timestamp(),
        has_cover: 0,
        has_paper: 0,
        ..Default::default()
    };
    
    // Generiamo l'hash e il file link per entrambi i libri
    book1.set_book_persistence();
    book2.set_book_persistence();
    
    // Verifichiamo che gli hash siano diversi
    assert_ne!(book1.file_hash, book2.file_hash, 
               "Libri diversi hanno generato lo stesso hash");
    
    // Verifichiamo che i file link siano diversi
    assert_ne!(book1.file_link, book2.file_link, 
               "Libri diversi hanno generato lo stesso file link");
}

#[test]
fn test_set_book_persistence_stability() {
    // Creiamo un libro con valori noti
    let mut book = Book {
        id: None,
        name: "Il Pendolo di Foucault".to_string(),
        original_title: Some("Foucault's Pendulum".to_string()),
        isbn: Some("9788845248252".to_string()),
        created_at: 1691747760,
        last_modified_date: Utc::now().timestamp(),
        has_cover: 0,
        has_paper: 0,
        ..Default::default()
    };
    
    // Generiamo l'hash
    book.set_book_persistence();
    
    // Salviamo l'hash e il file link
    let first_hash = book.file_hash.clone();
    let first_link = book.file_link.clone();
    
    // Cambiamo un campo non utilizzato per la generazione dell'hash
    book.has_cover = 1;
    book.has_paper = 1;
    book.last_modified_date = Utc::now().timestamp();
    
    // Rigeneriamo l'hash
    book.set_book_persistence();
    
    // Verifichiamo che l'hash e il file link non siano cambiati
    assert_eq!(book.file_hash, first_hash, 
               "L'hash è cambiato nonostante non siano cambiati i metadati rilevanti");
    assert_eq!(book.file_link, first_link, 
               "Il file link è cambiato nonostante non siano cambiati i metadati rilevanti");
    
    // Ora cambiamo un campo rilevante
    book.name = "Il Nome della Rosa".to_string();
    
    // Rigeneriamo l'hash
    book.set_book_persistence();
    
    // Verifichiamo che l'hash e il file link siano cambiati
    assert_ne!(book.file_hash, first_hash, 
               "L'hash non è cambiato nonostante sia cambiato un metadato rilevante");
    assert_ne!(book.file_link, first_link, 
               "Il file link non è cambiato nonostante sia cambiato un metadato rilevante");
}