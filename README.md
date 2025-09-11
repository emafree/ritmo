# full_ritmo
Un programma inspirato a **Calibre**, ma con alcune differenze significative:

  1. I contenuti di un libro sono molto importanti, e centrali per l'elaborazione dei dati
  2. Le persone che hanno contribuito o lavorato ad un libro hanno una importanza molto maggiore che non per Calibre 
  3. Non è prevista al momento la possibilità di usare dati user-defined
  4. Altre 100 differenze...
  
Il programma è scritto in rust, ed è un learning-while-coding example.
Il db usato è SQLite.

I vari blocchi sono:
  1. **ritmo_db**

    Questa parte è l'immagine nel codice del database.

    - La cartella **src/models** contiene le strutture che supportano l'intero db.
    - La cartella **schema** contiene lo script sql per generare un database
    - La cartella **assets** contiene un database template: template.db

    Quando si crea un nuovo database, questo viene copiato dal template. Se il template per qualche motivo è corrotto
    o non valido, viene ricreato usando lo script.
    
  2. **ritmo_db_core**

    Questo è il basso livello del database.
  
  3. **ritmo_core**
  4. **ritmo_cli**
  5. **ritmo_gui**
  6. **ritmo_ml**
  7. **ritmo_search**
  8. **ritmo_errors**
     
    Questa è la utility degli errori del progetto.
  
  9. **ebook_parser**

    Il ebook_parser è la utility che serve a leggere i files epub ed a estrarne i dati necessari a memorizzre l'epub
    stesso: autore/i, titolo/i, titolo/i originale/i, editore, date, etc. Inizialmente questa utility era stata
    scritta come un codice indipendente, ma poi ho pensato che fosse meglio integrarla nel software. 
