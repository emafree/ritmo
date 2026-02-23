# Ritmo - Functional Test Checklist

## 1. EPUB Management

### 1.1 Import EPUB
- [ ] Import singolo EPUB valido
- [ ] Import EPUB con metadati completi
- [ ] Import EPUB con metadati parziali/mancanti
- [ ] Import EPUB corrotto/malformato (deve fallire gracefully)
- [ ] Import EPUB senza copertina
- [ ] Import EPUB con copertina
- [ ] Import EPUB molto grande (>50MB)
- [ ] Import EPUB molto piccolo (<100KB)
- [ ] Import duplicato (stesso libro già presente)
- [ ] Import batch (multipli EPUB in una volta)

### 1.2 Export EPUB
- [ ] Export singolo libro
- [ ] Export con metadati preservati
- [ ] Export mantiene integrità file

### 1.3 Delete
- [ ] Elimina libro dal DB
- [ ] File EPUB rimane/viene eliminato (a seconda della config)
- [ ] Eliminazione rimuove anche contenuti indicizzati

---

## 2. Metadata Management

### 2.1 Extraction
- [ ] Estrae titolo correttamente
- [ ] Estrae autore/i correttamente
- [ ] Estrae anno pubblicazione
- [ ] Estrae genere/tag
- [ ] Estrae lingua
- [ ] Estrae publisher
- [ ] Gestisce metadati multilingua

### 2.2 Normalization (con ML)
- [ ] "Asimov, Isaac" e "Isaac Asimov" → stesso autore
- [ ] "I. Asimov" → normalizzato correttamente
- [ ] Autori con nomi non-occidentali
- [ ] Autori con particelle ("von", "de", "van")

### 2.3 Update
- [ ] Modifica titolo esistente
- [ ] Modifica autore
- [ ] Aggiunge/rimuove tag
- [ ] Update batch su multipli libri

---

## 3. Content Indexing

### 3.1 Extraction
- [ ] Estrae capitoli/sezioni correttamente
- [ ] Estrae testo da EPUB con HTML complesso
- [ ] Gestisce EPUB con immagini inline
- [ ] Gestisce EPUB con footnote/endnotes
- [ ] Preserva struttura gerarchica (parti > capitoli)

### 3.2 Indexing
- [ ] Full-text search index creato
- [ ] Index aggiornato dopo modifica
- [ ] Index performante su 1k libri
- [ ] Index performante su 10k libri

---

## 4. Search Functionality

### 4.1 Basic Search
- [ ] Ricerca per titolo esatto
- [ ] Ricerca per titolo parziale
- [ ] Ricerca per autore
- [ ] Ricerca case-insensitive
- [ ] Ricerca con caratteri speciali/accenti

### 4.2 Full-Text Search
- [ ] Ricerca parola singola nel contenuto
- [ ] Ricerca frase esatta ("two words")
- [ ] Ricerca booleana (AND, OR, NOT)
- [ ] Ricerca con wildcard (robo*)
- [ ] Ricerca proximity (parole vicine)
- [ ] Ranking risultati per relevance

### 4.3 Filters
- [ ] Filtra per genere
- [ ] Filtra per anno (range)
- [ ] Filtra per autore
- [ ] Filtra per lingua
- [ ] Filtra per tag
- [ ] Combinazione multipli filtri

### 4.4 Performance
- [ ] Query su 100 libri < 100ms
- [ ] Query su 1k libri < 500ms
- [ ] Query su 10k libri < 1s
- [ ] Full-text search su 100k contenuti < 2s

---

## 5. Anthology Handling

### 5.1 Detection
- [ ] Rileva antologia con autori multipli
- [ ] Rileva antologia con "edited by"
- [ ] Rileva fix-up novel (singolo autore)
- [ ] NON rileva romanzo normale come antologia

### 5.2 Content Extraction
- [ ] Estrae contenuti da antologia
- [ ] Assegna autore corretto a ogni contenuto
- [ ] Preserva ordine originale
- [ ] Gestisce contenuti senza titolo chiaro

### 5.3 Search in Anthologies
- [ ] Ricerca trova contenuto specifico in antologia
- [ ] Filtra per autore di contenuto (non solo libro)
- [ ] Risultati mostrano: contenuto + libro contenitore

---

## 6. Library Management

### 6.1 Single Library
- [x] Crea nuova libreria
- [x] Apre libreria esistente
- [x] Libreria con path relativo
- [x] Libreria con path assoluto
- [x] Libreria su drive esterno

### 6.2 Statistics
- [x] Conta totale libri
- [ ] Conta per genere
- [ ] Conta per autore
- [ ] Conta contenuti indicizzati

---

## 7. CLI Interface

### 7.1 Commands
- [ ] `ritmo add <file>` funziona
- [ ] `ritmo search "query"` funziona
- [ ] `ritmo list` mostra tutti i libri
- [ ] `ritmo list --filter` applica filtri
- [ ] `ritmo info <id>` mostra dettagli
- [ ] `ritmo delete <id>` rimuove libro
- [ ] `ritmo --help` mostra aiuto
- [ ] `ritmo --version` mostra versione

### 7.2 Error Handling
- [ ] Comando invalido → messaggio chiaro
- [ ] File non esistente → errore descrittivo
- [ ] Libreria non trovata → suggerisce creazione
- [ ] Permessi insufficienti → errore chiaro

### 7.3 Output Format
- [ ] Output human-readable di default
- [ ] `--json` output machine-readable
- [ ] `--quiet` sopprime output verbose
- [ ] Colori nel terminale (se supportato)

---

## 8. GUI Interface (egui)

### 8.1 Main Window
- [ ] Finestra si apre senza crash
- [ ] Risponde a resize
- [ ] Chiusura pulita (salva state)

### 8.2 Book List
- [ ] Mostra lista libri
- [ ] Scrolling fluido con 100 libri
- [ ] Scrolling fluido con 1k libri
- [ ] Click su libro → mostra dettagli
- [ ] Ordinamento per titolo/autore/anno

### 8.3 Search Interface
- [ ] Input search funziona
- [ ] Risultati aggiornati in tempo reale
- [ ] Highlight query nei risultati
- [ ] Clear search ripristina lista

### 8.4 Detail View
- [ ] Mostra metadati completi
- [ ] Mostra copertina (se presente)
- [ ] Mostra contenuti/capitoli
- [ ] Pulsante per aprire EPUB

---

## 9. Database

### 9.1 Integrity
- [ ] DB creato se non esiste
- [ ] Schema migrazione da versione precedente
- [ ] Foreign key constraints rispettate
- [ ] Indici creati correttamente

### 9.2 Performance
- [ ] Insert 1k libri < 10s
- [ ] Query su 10k libri < 1s
- [ ] DB size ragionevole (< 10% EPUB size)

### 9.3 Corruption Recovery
- [ ] DB corrotto → rebuild possibile
- [ ] Transazioni rollback su errore

---

## 10. Cross-Platform

### 10.1 Linux
- [ ] Compila senza errori
- [ ] Binario funziona su x86_64
- [ ] Binario funziona su ARM
- [ ] Path unix gestiti correttamente

### 10.2 Windows
- [ ] Compila senza errori
- [ ] GUI rendering corretto
- [ ] Path Windows (C:\) gestiti
- [ ] .exe eseguibile

### 10.3 macOS
- [ ] Compila senza errori
- [ ] GUI rendering corretto
- [ ] Binario funziona su Intel
- [ ] Binario funziona su Apple Silicon

---

## 11. Edge Cases

- [ ] Libreria vuota (0 libri)
- [ ] Libreria enorme (100k libri)
- [ ] EPUB con encoding non-UTF8
- [ ] EPUB con DRM (dovrebbe fallire gracefully)
- [ ] Path con caratteri unicode
- [ ] Path molto lunghi (>256 char)
- [ ] Disco pieno durante import
- [ ] Interruzione durante indicizzazione
- [ ] Concurrent access (2 istanze ritmo)

---

## 12. Regression Tests

- [ ] Bug fix precedenti non regrediscono
- [ ] Feature esistenti continuano a funzionare dopo refactor

---

## Test Priority Levels

### P0 - Blocker (must pass per prototipo)
- Import/export EPUB base
- Search metadata + full-text base
- CLI comandi essenziali
- GUI main window + lista funzionante

### P1 - Important (should pass prima di release)
- Anthology handling
- Performance con 10k libri
- Cross-platform (almeno Linux + Windows)
- Error handling robusto

### P2 - Nice-to-have
- Edge cases estremi
- Performance con 100k libri
- Tutte le feature avanzate

---

## Test Results

**Date:** _______
**Version:** _______
**Tester:** _______

**Summary:**
- Total tests: ___
- Passed: ___
- Failed: ___
- Skipped: ___

**Notes:**
