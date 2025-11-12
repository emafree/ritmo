# ritmo
Un programma inspirato a **Calibre**, ma con alcune differenze significative. **ritmo** non ha intenzione di gestire i libri, ma solo la biblioteca.
Per questo motivo non contiene facilities di editing, di reading, di conversione di formato, etc. . 
Lo scopo del programma è catalogare libri ed i loro contenuti, ed eventualmente le persone che hanno contribuito ai libri stessi, quindi autori, traduttori, illustratori, curatori, etc.

Il programma è scritto in rust, ed è un learning-while-coding example (in altri termini significa che per me è una scusa per studiare il rust).
Il db usato è SQLite, per il semplice motivo che non ha bisogno di un server esterno.

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
     
    Interfaccia a riga di comando
  
  5. **ritmo_gui**

    Interfaccia grafica
  
  6. **ritmo_ml**
  
  7. **ritmo_search**
  
  8. **ritmo_errors**
     
    Questa è la utility degli errori del progetto.
  
  9. **ebook_parser**

    Il ebook_parser è la utility che serve a leggere i files epub ed a estrarne i dati necessari a memorizzre l'epub
    stesso: autore/i, titolo/i, titolo/i originale/i, editore, date, etc. Inizialmente questa utility era stata
    scritta come un codice indipendente, ma poi ho pensato che fosse meglio integrarla nel software. In realtà questa è la parte più importante del codice.
    La mia libreria attualmente comprende circa 12000 libri, e non è pensabile doverli scandire a mano uno per uno ed introdurre i relativi dati nel database.
    Questa utility deve essere affinata fino ad essere in grado di estrarre tutti i dati necessari da almeno il 95% dei libri.
