# Architecture Decision Records (ADR)

This document tracks important architectural decisions for the ritmo project.

---

## ADR-001: Single-User Architecture (2024-12-16)

### Status
✅ **ACCEPTED**

### Context
During the discussion on implementing an analytics system, the need arose to decide whether ritmo should support multi-user or remain single-user.

### Decision
**ritmo remains a single-user application for version 1.0**

### Rationale

**Pro Single-User:**
- ✅ Architectural simplicity (local SQLite, no authentication layer)
- ✅ Development speed (no multi-tenant complexity)
- ✅ Privacy by design (everything local, zero cloud dependencies)
- ✅ Perfect portability (USB library works out-of-the-box)
- ✅ Focus on core features (filters, ebook_parser, cataloging)

**Cons Multi-User (if implemented now):**
- ❌ Complexity: +40-60% codebase (auth, user management, permissions)
- ❌ Testing: +300% scenarios (multi-user edge cases)
- ❌ Deployment: requires server setup, monitoring, security updates
- ❌ Database: requires PostgreSQL or SQLite multi-tenant with locking issues
- ❌ Privacy: GDPR compliance, user data management

**Primary Use Case:**
- Personal library management (12,000+ books)
- Personal desktop/laptop use
- USB portability for multi-device access
- No need for sharing with other users at the moment

### Consequences

**Positive:**
- Architecture remains simple and maintainable
- Local SQLite sufficient (no PostgreSQL)
- No authentication/authorization layer needed
- Analytics (if implemented) will be much simpler
- Portable library works perfectly

**Negative:**
- If multi-user is needed in the future, will require significant refactoring
- Library sharing between people requires workarounds (e.g., shared library on network drive)

**Neutral:**
- Multi-user remains possible in v2.0 as breaking change
- Current database schema doesn't include user_id (correct for single-user)

### Future Considerations
- **v2.0**: If real multi-user need emerges, re-evaluate
- Possible scenario: family/team wants to share library
- At that point: complete database redesign with user_id, auth layer, permissions

### Related Decisions
- See ADR-002 for analytics (related)

---

## ADR-002: Filter System as Core Identity (2024-12-16)

### Status
✅ **ACCEPTED**

### Context
During the filter system refactoring, it emerged that the user identifies the program primarily through the search and filter interface, not through other features.

### Decision
**The filter system is considered the program's core and must be architecturally isolated and easily evolvable**

### Rationale

**Key Observation:**
- User doesn't think "I use ritmo", thinks "I search my books"
- Filter system is the primary interface, not an accessory feature
- Filter quality determines entire program usability

**Implications:**
```
      ┌─────────────┐
      │   FILTERS   │  ← System core
      └──────┬──────┘
             │
    ┌────────┼────────┐
    │        │        │
┌───▼───┐ ┌──▼──┐ ┌──▼──┐
│  CLI  │ │ GUI │ │ API │  ← Interfaces exposing filters
└───────┘ └─────┘ └─────┘
```

### Architecture Implementation

**Isolated Module:**
```
ritmo_db_core/src/filters/
├── mod.rs          # Stable public API
├── types.rs        # BookFilters, ContentFilters, enums
├── builder.rs      # SQL query construction
├── executor.rs     # Query execution
├── validator.rs    # Input validation
└── tests.rs        # Comprehensive test suite
```

**Design Principles:**
1. **Isolation**: Filter changes don't impact rest of system
2. **Stable API**: FilterEngine with public interface that doesn't change
3. **Testability**: Independent tests without complex database setup
4. **Evolvability**: Easy to add new filter types

### Consequences

**Positive:**
- Filter changes are local and safe
- Isolated testing simpler
- Easy to add features (range filters, negation, fuzzy search, complex logic)
- Performance tuning doesn't impact CLI/GUI
- Backward compatibility easily maintained

**Technical Debt Avoided:**
- Filters scattered throughout codebase
- SQL logic duplicated between CLI and GUI
- Difficult testing coupled to rest of system

### Implementation Status
🔄 **IN PROGRESS** - Refactoring in progress (Zed)

### Future Evolution Path
1. **Phase 1** (current): OR logic for multiple parameters
2. **Phase 2**: Range filters, negation
3. **Phase 3**: Full-text search integration
4. **Phase 4**: Query DSL language (optional)
5. **Phase 5**: AI-powered search (distant future)

---

## ADR-003: OR Logic for Repeated Parameters (2024-12-16)

### Status
🔄 **IN PROGRESS** (Refactoring in progress)

### Context
Current filter system only uses AND logic. For more flexible searches, OR support for repeated parameters is needed.

### Decision
**Repeated parameters are combined with OR logic, different parameters with AND logic**

### Examples
```bash
# Implicit OR for repeated parameters
ritmo list-books --author "King" --author "Tolkien"
# SQL: WHERE (author LIKE '%King%' OR author LIKE '%Tolkien%')

ritmo list-books --format epub --format pdf
# SQL: WHERE (format = 'epub' OR format = 'pdf')

# AND between different parameters
ritmo list-books --author "King" --format epub
# SQL: WHERE author LIKE '%King%' AND format = 'epub'

# Combined (OR within groups, AND between groups)
ritmo list-books --author "King" --author "Tolkien" --format epub --year 2024
# SQL: WHERE (author LIKE '%King%' OR author LIKE '%Tolkien%')
#       AND format = 'epub'
#       AND year = 2024
```

### Rationale

**Alternative Considered #1: Explicit --or flag**
```bash
ritmo list-books --or --author "King" --author "Tolkien"
```
❌ Rejected: More complex syntax, less intuitive

**Alternative Considered #2: Comma syntax**
```bash
ritmo list-books --author "King,Tolkien"
```
❌ Rejected: Problems with names containing commas, less clear

**Chosen Solution: Implicit OR**
✅ More intuitive: repeating parameter = "this OR that"
✅ Clap already supports multiple parameters
✅ Backward compatible
✅ Easy to implement

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
- More flexible and powerful searches
- Natural and intuitive syntax
- Supports common use cases (multiple authors, formats)

**Considerations:**
- Possible confusion about when OR vs AND is used (documentation important)
- Performance: OR on many values can be slower (acceptable)

### Testing
- Tests for OR logic on single parameter
- Tests for OR + AND combination
- Tests for injection prevention with multiple parameters

---

## ADR-004: Analytics System - Deferred Decision (2024-12-16)

### Status
⏸️ **DEFERRED**

### Context
Discussed implementing an analytics system to track queries and generate insights. Decision was postponed for further consideration.

### Considerations Raised

**Pro Analytics:**
- Understanding usage patterns
- Automatic preset suggestions
- Continuous improvement based on real data
- Performance monitoring

**Concerns:**
- Added complexity
- Privacy considerations (even if all local)
- Risk of over-engineering in initial phase
- Multi-user analytics would require auth (see ADR-001)

### Key Insight
**Analytics in multi-user context** would require:
- User authentication/authorization
- Per-user analytics + global analytics
- Permission model (who sees what)
- More sophisticated privacy controls

**Decision**: Postpone analytics until:
1. Filter system completed and tested in production
2. ebook_parser integrated (automatic cataloging)
3. Real usage of program with complete library
4. Empirical evaluation: are analytics really needed?

### Next Steps
- Complete core features (filters, ebook_parser, GUI)
- Use program to manage real library
- Re-evaluate analytics in 3-6 months based on concrete needs

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
- Everything local (privacy by design)
- Easy deletion
- No multi-user complexity

---

## Template for Future ADR
```markdown
## ADR-XXX: [Title] (YYYY-MM-DD)

### Status
[PROPOSED | ACCEPTED | DEPRECATED | SUPERSEDED]

### Context
[Problem or situation requiring a decision]

### Decision
[Decision made clearly and concisely]

### Rationale
[Why this decision? Alternatives considered?]

### Consequences
**Positive:**
- [Pros]

**Negative:**
- [Cons]

**Neutral:**
- [Trade-offs]

### Related Decisions
- [Links to other related ADRs]
```

---

## Metadata

**Last Updated:** 2024-12-16
**Contributors:** Emanuele (maintainer)
**Repository:** ritmo - Rust Library Management System

## Notes

This document follows the Architecture Decision Records (ADR) pattern to track important decisions over time. Each decision has:
- **Context**: Why a decision is needed
- **Decision**: What was decided
- **Rationale**: Why this choice
- **Consequences**: Impact of the decision

Decisions are not immutable - they can be re-evaluated (status SUPERSEDED) when context changes.