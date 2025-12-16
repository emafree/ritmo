# Architecture Decision Records (ADR)

Questo documento traccia le decisioni architetturali importanti per il progetto ritmo.

---

## ADR-001: Single-User Architecture (2024-12-16)

### Status
✅ **ACCEPTED**

### Context
Durante la discussione sull'implementazione di un sistema di analytics, è emersa la necessità di decidere se ritmo dovesse supportare multi-utente o rimanere single-user.

### Decision
**ritmo rimane un'applicazione single-user per la versione 1.0**

### Rationale

**Pro Single-User:**
- ✅ Semplicità architetturale (SQLite locale, no authentication layer)
- ✅ Velocità di sviluppo (no complessità multi-tenant)
- ✅ Privacy by design (tutto locale, zero dipendenze cloud)
- ✅ Portabilità perfetta (USB library funziona out-of-the-box)
- ✅ Focus sulle feature core (filtri, ebook_parser, catalogazione)

**Contro Multi-User (se implementato ora):**
- ❌ Complessità: +40-60% codebase (auth, user management, permissions)
- ❌ Testing: +300% scenari (multi-user edge cases)
- ❌ Deployment: richiede server setup, monitoring, security updates
- ❌ Database: necessita PostgreSQL o SQLite multi-tenant con locking issues
- ❌ Privacy: GDPR compliance, user data management

**Use Case Primario:**
- Gestione libreria personale (12,000+ libri)
- Uso desktop/laptop personale
- Portabilità su USB per accesso multi-dispositivo
- No necessità di condivisione con altri utenti al momento

### Consequences

**Positive:**
- Architettura rimane semplice e manutenibile
- SQLite locale sufficiente (no PostgreSQL)
- No authentication/authorization layer necessario
- Analytics (se implementato) sarà molto più semplice
- Portable library funziona perfettamente

**Negative:**
- Se in futuro serve multi-utente, richiederà refactoring significativo
- Condivisione libreria tra persone richiede workaround (es: libreria shared su network drive)

**Neutral:**
- Multi-utente rimane possibile in v2.0 come breaking change
- Database schema attuale non include user_id (corretto per single-user)

### Future Considerations
- **v2.0**: Se emerge necessità reale di multi-utente, rivalutare
- Possibile scenario: famiglia/team vuole condividere libreria
- A quel punto: complete database redesign con user_id, auth layer, permissions

### Related Decisions
- Vedi ADR-002 per analytics (correlato)

---

## ADR-002: Filter System as Core Identity (2024-12-16)

### Status
✅ **ACCEPTED**

### Context
Durante il refactoring del sistema filtri, è emerso che l'utente identifica il programma principalmente attraverso l'interfaccia di ricerca e filtri, non attraverso altre feature.

### Decision
**Il sistema di filtri è considerato il core del programma e deve essere architetturalmente isolato e facilmente evolvibile**

### Rationale

**Osservazione Chiave:**
- L'utente non pensa "uso ritmo", pensa "cerco i miei libri"
- Il sistema di filtri è l'interfaccia principale, non una feature accessoria
- La qualità dei filtri determina l'usabilità dell'intero programma

**Implicazioni:**
```
      ┌─────────────┐
      │   FILTRI    │  ← Core del sistema
      └──────┬──────┘
             │
    ┌────────┼────────┐
    │        │        │
┌───▼───┐ ┌──▼──┐ ┌──▼──┐
│  CLI  │ │ GUI │ │ API │  ← Interfacce che espongono filtri
└───────┘ └─────┘ └─────┘
```

### Architecture Implementation

**Modulo Isolato:**
```
ritmo_db_core/src/filters/
├── mod.rs          # Public API stabile
├── types.rs        # BookFilters, ContentFilters, enums
├── builder.rs      # Query SQL construction
├── executor.rs     # Query execution
├── validator.rs    # Input validation
└── tests.rs        # Comprehensive test suite
```

**Design Principles:**
1. **Isolamento**: Modifiche ai filtri non impattano il resto del sistema
2. **API Stabile**: FilterEngine con interfaccia pubblica che non cambia
3. **Testabilità**: Test indipendenti senza setup database complesso
4. **Evolvibilità**: Facile aggiungere nuovi tipi di filtri

### Consequences

**Positive:**
- Modifiche ai filtri sono locali e sicure
- Testing isolato più semplice
- Facile aggiungere feature (range filters, negation, fuzzy search, complex logic)
- Performance tuning non impatta CLI/GUI
- Backward compatibility mantenuta facilmente

**Technical Debt Avoided:**
- Filtri sparsi in tutto il codebase
- Logica SQL duplicata tra CLI e GUI
- Testing difficile e accoppiato al resto del sistema

### Implementation Status
🔄 **IN PROGRESS** - Refactoring in corso (Zed)

### Future Evolution Path
1. **Phase 1** (current): OR logic per parametri multipli
2. **Phase 2**: Range filters, negation
3. **Phase 3**: Full-text search integration
4. **Phase 4**: Query DSL language (opzionale)
5. **Phase 5**: AI-powered search (futuro lontano)

---

## ADR-003: OR Logic for Repeated Parameters (2024-12-16)

### Status
🔄 **IN PROGRESS** (Refactoring in corso)

### Context
Il sistema di filtri attuale usa solo logica AND. Per ricerche più flessibili, serve supporto OR su parametri ripetuti.

### Decision
**Parametri ripetuti vengono combinati con OR logic, parametri diversi con AND logic**

### Examples

```bash
# OR implicito per parametri ripetuti
ritmo list-books --author "King" --author "Tolkien"
# SQL: WHERE (author LIKE '%King%' OR author LIKE '%Tolkien%')

ritmo list-books --format epub --format pdf
# SQL: WHERE (format = 'epub' OR format = 'pdf')

# AND tra parametri diversi
ritmo list-books --author "King" --format epub
# SQL: WHERE author LIKE '%King%' AND format = 'epub'

# Combinato (OR dentro gruppi, AND tra gruppi)
ritmo list-books --author "King" --author "Tolkien" --format epub --year 2024
# SQL: WHERE (author LIKE '%King%' OR author LIKE '%Tolkien%')
#       AND format = 'epub'
#       AND year = 2024
```

### Rationale

**Alternativa Considerata #1: Flag --or esplicito**
```bash
ritmo list-books --or --author "King" --author "Tolkien"
```
❌ Rifiutata: Sintassi più complessa, meno intuitiva

**Alternativa Considerata #2: Sintassi con virgole**
```bash
ritmo list-books --author "King,Tolkien"
```
❌ Rifiutata: Problemi con nomi che contengono virgole, meno chiara

**Soluzione Scelta: OR implicito**
✅ Più intuitiva: ripetere parametro = "questo O quello"
✅ Clap già supporta parametri multipli
✅ Backward compatible
✅ Facile da implementare

### Implementation

```rust
// In BookFilters
pub struct BookFilters {
    pub authors: Vec<String>,      // King, Tolkien → OR
    pub formats: Vec<String>,      // epub, pdf → OR
    pub publishers: Vec<String>,   // OR logic
    pub series: Vec<String>,       // OR logic
    // Single values remain as Option<T>
    pub year: Option<i32>,
}

// In query builder
if !filters.authors.is_empty() {
    let conditions: Vec<String> = filters.authors
        .iter()
        .map(|_| "p.name LIKE ?")
        .collect();
    where_clauses.push(format!("({})", conditions.join(" OR ")));
}
```

### Consequences

**Positive:**
- Ricerche più flessibili e potenti
- Sintassi naturale e intuitiva
- Supporta casi d'uso comuni (multiple authors, formats)

**Considerations:**
- Possibile confusion su quando si usa OR vs AND (documentazione importante)
- Performance: OR su molti valori può essere più lento (accettabile)

### Testing
- Test per OR logic su singolo parametro
- Test per combinazione OR + AND
- Test per injection prevention con parametri multipli

---

## ADR-004: Analytics System - Deferred Decision (2024-12-16)

### Status
⏸️ **DEFERRED**

### Context
Discussa l'implementazione di un sistema di analytics per tracciare query e generare insights. La decisione è stata rimandata per approfondimento.

### Considerations Raised

**Pro Analytics:**
- Comprensione pattern di utilizzo
- Suggerimenti preset automatici
- Miglioramento continuo basato su dati reali
- Performance monitoring

**Concerns:**
- Complessità aggiunta
- Privacy considerations (anche se tutto locale)
- Rischio over-engineering in fase iniziale
- Multi-user analytics richiederebbe auth (vedi ADR-001)

### Key Insight
**Analytics nel contesto multi-user** richiederebbe:
- User authentication/authorization
- Per-user analytics + global analytics
- Permission model (chi vede cosa)
- Privacy controls più sofisticati

**Decisione**: Posticipare analytics fino a:
1. Sistema filtri completato e testato in produzione
2. ebook_parser integrato (catalogazione automatica)
3. Uso reale del programma con libreria completa
4. Valutazione empirica: gli analytics servono davvero?

### Next Steps
- Completare feature core (filtri, ebook_parser, GUI)
- Usare il programma per gestire libreria reale
- Rivalutare analytics tra 3-6 mesi basandosi su necessità concrete

### If Analytics is Implemented Later

**Simplified Approach (Single-User):**
```
~/.config/ritmo/analytics/
├── queries.jsonl        # Query log (append-only)
├── stats.toml           # Aggregated statistics
└── insights.toml        # Generated suggestions

Commands:
- ritmo analytics stats
- ritmo analytics insights
- ritmo analytics clear
```

**Design Principles:**
- Opt-in (ask on first run)
- Tutto locale (privacy by design)
- Cancellazione facile
- No complessità multi-user

---

## Template per Future ADR

```markdown
## ADR-XXX: [Title] (YYYY-MM-DD)

### Status
[PROPOSED | ACCEPTED | DEPRECATED | SUPERSEDED]

### Context
[Problema o situazione che richiede una decisione]

### Decision
[Decisione presa in modo chiaro e conciso]

### Rationale
[Perché questa decisione? Alternative considerate?]

### Consequences
**Positive:**
- [Pro]

**Negative:**
- [Contro]

**Neutral:**
- [Trade-offs]

### Related Decisions
- [Link ad altri ADR correlati]
```

---

## Metadata

**Last Updated:** 2024-12-16
**Contributors:** Emanuele (maintainer)
**Repository:** ritmo - Rust Library Management System

## Notes

Questo documento segue il pattern Architecture Decision Records (ADR) per tracciare decisioni importanti nel tempo. Ogni decisione ha:
- **Context**: Perché serve una decisione
- **Decision**: Cosa è stato deciso
- **Rationale**: Perché questa scelta
- **Consequences**: Impatto della decisione

Le decisioni non sono immutabili - possono essere rivalutate (status SUPERSEDED) quando il contesto cambia.
