# Ritmo GUI

Interfaccia grafica moderna e minimalista per Ritmo, costruita con [Slint](https://slint.dev/).

## Caratteristiche

- **Design Minimalista**: Interfaccia pulita e moderna con focus sulla semplicità
- **Leggera**: Build nativo senza dipendenze pesanti, funziona completamente offline
- **Cross-platform**: Funziona su Linux, Windows e macOS
- **Async**: Integrazione completa con Tokio per operazioni database non bloccanti

## Struttura

```
ritmo_gui/
├── src/
│   └── main.rs          # Entry point e logica applicazione
├── ui/
│   └── main_window.slint # Definizione UI in linguaggio Slint
├── build.rs             # Build script per compilare file .slint
└── Cargo.toml           # Dipendenze
```

## Compilazione

```bash
# Build in modalità debug
cargo build -p ritmo_gui

# Build ottimizzata (release)
cargo build -p ritmo_gui --release
```

## Esecuzione

```bash
# Esegui direttamente
cargo run -p ritmo_gui

# Oppure esegui il binario compilato
./target/release/ritmo_gui
```

## Interfaccia

### Sidebar
- 📖 **Libri**: Vista principale con lista di tutti i libri
- ✍️ **Autori**: Gestione autori (in sviluppo)
- 🏢 **Editori**: Gestione editori (in sviluppo)
- 📚 **Serie**: Gestione serie di libri (in sviluppo)
- ⚙️ **Impostazioni**: Configurazione applicazione (in sviluppo)

### Area Principale
- **Barra di ricerca**: Cerca libri, autori, editori in tempo reale
- **Lista libri**: Vista cards dei libri con titolo, autore, editore, anno
- **Pulsante Aggiungi**: Per aggiungere nuovi libri (in sviluppo)
- **Messaggi di stato**: Feedback visivo per operazioni

## Inizializzazione

All'avvio, l'applicazione:
1. Crea automaticamente la directory della libreria in `~/RitmoLibrary`
2. Inizializza il database SQLite se non esiste
3. Crea la struttura di directory necessaria (database, storage, config, bootstrap)
4. Carica i libri dalla libreria (attualmente dati di esempio)

## Tecnologie

- **Slint 1.7.2**: Framework UI nativo e performante
- **Tokio**: Runtime asincrono per operazioni I/O
- **SQLx**: Database access layer asincrono
- **Rust**: Linguaggio sicuro e performante

## Stato Attuale

✅ Interfaccia base implementata
✅ Navigazione sidebar funzionante
✅ Ricerca libri con filtro in tempo reale
✅ Integrazione con LibraryConfig
✅ Inizializzazione automatica libreria

🚧 In sviluppo:
- Integrazione vera con database (query SQL)
- Dialog per aggiungere/modificare libri
- Vista dettaglio libro
- Gestione autori, editori, serie
- Import di file EPUB
- Gestione copertine

## Dipendenze

Le principali dipendenze sono:
- `slint = "1.7.2"` - Framework UI
- `slint-build = "1.7.2"` - Build script per file .slint
- `tokio` - Async runtime
- `dirs` - Per trovare directory home utente
- `ritmo_db_core` - Gestione database e config
- `ritmo_db` - Modelli database
- `ritmo_core` - Logica business

## Note di Sviluppo

### File .slint
I file `.slint` definiscono l'interfaccia grafica usando un linguaggio dichiarativo simile a QML. Durante la build, `slint-build` compila questi file in codice Rust.

### Async/Sync Bridge
L'applicazione usa `tokio::runtime::Runtime` per gestire operazioni async dal thread UI sincrono. Le operazioni database vengono eseguite tramite `runtime.block_on()`.

### Callbacks
I callback UI sono definiti in Slint e implementati in Rust:
- `initialize-library`: Inizializza la libreria
- `refresh-books`: Ricarica la lista libri
- `search-books`: Filtra libri in base al testo di ricerca
- `add-new-book`: Apre dialog per aggiungere libro (TODO)
