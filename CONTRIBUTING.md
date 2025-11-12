# Contributing

Grazie per il contributo a Ritmo!

Linee guida rapide:
- Fork & branch: crea un branch con nome `feature/<descrizione>` o `fix/<descrizione>`.
- Formato codice: usa `cargo fmt` prima di aprire PR.
- Lint: esegui `cargo clippy --all -- -D warnings` dove possibile.
- Tests: aggiungi test unitari / di integrazione per nuove funzionalità.
- Commit: scrivi messaggi chiari e referenzia issue se appropriato.
- Aggiunta di nuovi crate: aggiornare `members` in `Cargo.toml` (root) e documentare il crate in `docs/workspace.md`.

Stile e processo:
- PR minimo: descrizione del cambiamento, motivazione, come testare.
- Il maintainer potrebbe richiedere revisioni di stile o architettura.
- Non committare segreti: usa `.env.example` per valori di esempio.
