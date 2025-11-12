# Workspace Ritmo — Guida rapida

Questo repository è organizzato come workspace Rust. Lo scopo di questo documento è spiegare come lavorare con il workspace.

## Membri del workspace
I crate principali sono elencati nel Cargo.toml di root come `members`:
- ritmo_core: logica principale e gestione ebook
- ritmo_cli: interfaccia a linea di comando
- ritmo_db / ritmo_db_core: gestione DB e metadati
- ritmo_mapping: mapping metadati
- ritmo_errors: crate per tipi di errore condivisi
Altri crate (gui, search, ml, ebook_parser) possono essere presenti ma commentati se non pronti.

## Comandi utili
- Build di tutto il workspace:
  cargo build --workspace

- Eseguire i test per tutto il workspace:
  cargo test --workspace

- Formattare il codice:
  cargo fmt --all

- Eseguire clippy (lint):
  cargo clippy --all -- -D warnings

- Aggiungere un nuovo crate:
  1. Creare la cartella del crate con `cargo new --lib nome_crate` o `cargo new --bin nome_crate`.
  2. Aggiungere il path nella lista `members` del `Cargo.toml` di root.
  3. Documentare il crate in questo file.

## Database
- Per sviluppo locale si usa SQLite. La stringa di connessione d'esempio è in `.env.example` (`DATABASE_URL`).
- Le migrazioni (se usate) devono essere versionate e documentate nel crate `ritmo_db_core`.

## CI e quality gates
- Il repo include un workflow CI che esegue: formattazione (check), clippy, build e test per tutto il workspace.
- Assicurarsi che i job CI passino prima di aprire una PR significativa.

## Note
- Non committare file `.env`. Usa `.env.example`.
- Se aggiungi dipendenze native o binarie (es. motori di conversione), documenta come installarle nella sezione `docs/build.md` (non ancora presente).