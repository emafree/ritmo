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

## ADR-005: Filter System Refactoring - Composable Filters with Smart Execution (2026-01-25)

### Status
🔄 **PROPOSED**

### Context

The current filter system in `ritmo_db_core/src/filters/` has filter logic tightly coupled to SQL query generation. This makes it difficult to:
- Compose filters together (nest filters on already-filtered results)
- Test filter logic without SQL strings
- Apply filters that can't be easily expressed in SQL
- Optimize execution based on dataset size
- Maintain and evolve the filter system independently

**Current Architecture:**
```
filters/
├── mod.rs        - Public API and module organization
├── types.rs      - Filter data structures (BookFilters, ContentFilters)
├── builder.rs    - SQL query construction (COUPLED TO LOGIC)
├── executor.rs   - Query execution against SQLite database
└── validator.rs  - Input validation
```

**Current Usage Pattern (CLI):**
```rust
let filters = BookFilters::new()
    .with_author(authors)
    .with_format(formats);
let books = execute_books_query(&pool, &filters).await?;
// ❌ Cannot apply additional filters on 'books'
// ❌ Cannot nest/compose filters
// ❌ Always uses SQL (no in-memory option)
```

**Problems Identified:**

1. **No Composability**: Cannot chain filters together
   - Filter returns `Vec<BookResult>` from SQL query
   - No way to apply another filter on results
   - Cannot build complex filter pipelines

2. **Tight Coupling to SQL**:
   ```rust
   pub fn build_books_query(filters: &BookFilters) -> (String, Vec<String>) {
       // All logic embedded in SQL string building
       // Cannot use different execution strategies
   }
   ```

3. **No Execution Strategy**: Always uses SQL queries
   - Inefficient for small datasets already in memory
   - Cannot leverage in-memory filtering when beneficial
   - No optimization based on context

4. **Limited Filter Expressiveness**: Some filters hard to express in SQL
   - Complex computed properties
   - ML-based similarity filters
   - Custom user-defined predicates

### Decision

**Implement Composable Filter Pattern with Smart Execution Strategy**

Filters operate on uniform interface: `Vec<BookResult>` → `Vec<BookResult>`

**Core Strategy:**
- **Uniform Interface**: All filters take and return `Vec<BookResult>`
- **Composability**: Filters can be chained/nested infinitely
- **Smart Execution**: Automatically choose SQL vs in-memory based on dataset size
- **Backward Compatibility**: Keep old API working with deprecation warnings

**Key Insight**: This pattern separates "what to filter" from "how to execute the filter", enabling optimization and composition.

### Proposed Architecture

**Core Trait: Uniform Interface**
```rust
/// All filters implement this trait
#[async_trait]
pub trait BookFilter: Send + Sync {
    /// Apply filter to a list of books, returning filtered list
    async fn apply(&self, books: Vec<BookResult>) -> RitmoResult<Vec<BookResult>>;

    /// Optional: Can this filter be executed via SQL?
    fn supports_sql(&self) -> bool { false }

    /// Optional: Convert to SQL conditions (if supports_sql returns true)
    fn to_sql_conditions(&self) -> Option<Vec<FilterCondition>> { None }
}
```

**Execution Strategies**

```rust
/// Strategy 1: SQL Execution (for large datasets or initial queries)
pub struct SqlFilter {
    conditions: Vec<FilterCondition>,
    pool: SqlitePool,
}

#[async_trait]
impl BookFilter for SqlFilter {
    async fn apply(&self, _books: Vec<BookResult>) -> RitmoResult<Vec<BookResult>> {
        // Ignores input, queries database directly
        let (query, params) = build_sql_query(&self.conditions)?;
        execute_query(&self.pool, query, params).await
    }

    fn supports_sql(&self) -> bool { true }
    fn to_sql_conditions(&self) -> Option<Vec<FilterCondition>> {
        Some(self.conditions.clone())
    }
}

/// Strategy 2: Memory Execution (for small datasets or complex filters)
pub struct MemoryFilter {
    predicate: Box<dyn Fn(&BookResult) -> bool + Send + Sync>,
}

#[async_trait]
impl BookFilter for MemoryFilter {
    async fn apply(&self, books: Vec<BookResult>) -> RitmoResult<Vec<BookResult>> {
        // Filters in memory
        Ok(books.into_iter().filter(|b| (self.predicate)(b)).collect())
    }
}

/// Strategy 3: Smart Execution (automatic optimization)
pub struct SmartFilter {
    threshold: usize,  // e.g., 1000 books
    sql_filter: SqlFilter,
    memory_filter: MemoryFilter,
}

#[async_trait]
impl BookFilter for SmartFilter {
    async fn apply(&self, books: Vec<BookResult>) -> RitmoResult<Vec<BookResult>> {
        if books.is_empty() {
            // First filter in chain: use SQL for efficiency
            self.sql_filter.apply(books).await
        } else if books.len() > self.threshold {
            // Large dataset: use SQL with WHERE id IN (...)
            self.sql_filter.apply_on_subset(books).await
        } else {
            // Small dataset: filter in memory (faster)
            self.memory_filter.apply(books).await
        }
    }

    fn supports_sql(&self) -> bool { true }
}
```

**Filter Composition**
```rust
/// Combinator: Chain multiple filters
pub struct FilterChain {
    filters: Vec<Box<dyn BookFilter>>,
}

#[async_trait]
impl BookFilter for FilterChain {
    async fn apply(&self, mut books: Vec<BookResult>) -> RitmoResult<Vec<BookResult>> {
        for filter in &self.filters {
            books = filter.apply(books).await?;
        }
        Ok(books)
    }
}

/// Helper for fluent API
pub struct FilterPipeline {
    pool: SqlitePool,
    filters: Vec<Box<dyn BookFilter>>,
}

impl FilterPipeline {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool, filters: Vec::new() }
    }

    /// Add filter to pipeline
    pub fn then(mut self, filter: impl BookFilter + 'static) -> Self {
        self.filters.push(Box::new(filter));
        self
    }

    /// Execute pipeline
    pub async fn execute(self) -> RitmoResult<Vec<BookResult>> {
        let chain = FilterChain { filters: self.filters };
        chain.apply(Vec::new()).await  // Empty vec = query all from DB
    }
}
```

**New Module Structure:**
```
filters/
├── mod.rs              - Public API
├── types.rs            - BookFilters, ContentFilters (for backward compat)
├── validator.rs        - Input validation (unchanged)
├── trait.rs            - NEW: BookFilter trait definition
├── conditions.rs       - NEW: FilterCondition enum (for SQL translation)
├── strategies/
│   ├── mod.rs          - Execution strategy exports
│   ├── sql.rs          - SqlFilter implementation
│   ├── memory.rs       - MemoryFilter implementation
│   └── smart.rs        - SmartFilter with automatic optimization
├── combinators/
│   ├── mod.rs          - Filter combinators
│   ├── chain.rs        - FilterChain for composition
│   ├── and.rs          - AND combinator
│   └── or.rs           - OR combinator
├── pipeline.rs         - NEW: FilterPipeline fluent API
├── executor.rs         - Updated to use new filter trait
└── compat/             - NEW: Backward compatibility layer
    ├── builder.rs      - Deprecated: build_books_query
    └── mod.rs          - Re-export old API with deprecation warnings
```

**New API Examples:**
```rust
// Example 1: Simple filter chain
let books = FilterPipeline::new(pool)
    .then(AuthorFilter::new(vec!["King".to_string()]))
    .then(FormatFilter::new(vec!["epub".to_string()]))
    .then(YearFilter::after(2020))
    .execute().await?;

// Example 2: Nested filtering
let scifi_books = FilterPipeline::new(pool)
    .then(TagFilter::new(vec!["sci-fi".to_string()]))
    .execute().await?;

// Now filter sci-fi books by author (operates on subset!)
let king_scifi = FilterPipeline::new(pool)
    .then(ConstantFilter::new(scifi_books))  // Start with subset
    .then(AuthorFilter::new(vec!["King".to_string()]))
    .execute().await?;

// Example 3: Complex filter not expressible in SQL
let custom_filter = MemoryFilter::new(Box::new(|book| {
    // Custom logic: e.g., ML-based similarity
    compute_similarity(&book, reference_book) > 0.8
}));

let similar_books = FilterPipeline::new(pool)
    .then(TagFilter::new(vec!["fantasy".to_string()]))  // SQL first
    .then(custom_filter)  // Then apply custom logic in memory
    .execute().await?;

// Old API (deprecated but still works)
let books = execute_books_query(&pool, &filters).await?;
```

### Rationale

**Why Composable Filter Pattern?**
- ✅ **Uniform Interface**: `Vec<BookResult>` → `Vec<BookResult>` enables infinite composition
- ✅ **Flexibility**: Can apply filters on already-filtered results (nested filtering)
- ✅ **Smart Execution**: Automatically optimizes (SQL for large sets, memory for small)
- ✅ **Expressive Power**: Support filters that can't be expressed in SQL (ML, custom logic)
- ✅ **Testability**: Easy to test with in-memory data, no database needed
- ✅ **Performance**: Best of both worlds (SQL efficiency + in-memory flexibility)
- ✅ **Foundation for Advanced Features**: DSL, AI search, query optimization

**Why Smart Execution Strategy?**
- ✅ **Performance Optimization**: Uses SQL when efficient, memory when beneficial
- ✅ **Automatic**: No user intervention needed, system decides best strategy
- ✅ **Transparent**: Same API regardless of execution strategy
- ✅ **Configurable**: Threshold can be tuned based on performance profiling

**Why Backward Compatibility?**
- ✅ Zero breaking changes initially
- ✅ Gradual migration at our own pace
- ✅ Easy rollback if issues are found
- ✅ Existing CLI code continues to work

**Concrete Benefits:**

1. **Nested Filtering** (impossible with current architecture):
   ```rust
   // Get all sci-fi books
   let scifi = tag_filter("sci-fi").execute().await?;

   // Now filter sci-fi by author (operates on subset!)
   let king_scifi = author_filter("King").apply(scifi).await?;
   ```

2. **Custom Complex Filters**:
   ```rust
   // ML-based similarity (can't be expressed in SQL!)
   let similar = MemoryFilter::new(|book| ml_similarity(book) > 0.8);
   let results = sql_filter.apply().await?.then(similar.apply).await?;
   ```

3. **Performance Optimization**:
   ```rust
   // System automatically decides:
   // - 100,000 books → use SQL
   // - 50 books already in memory → filter in memory (faster!)
   ```

**Alternatives Considered:**

1. **Query Builder Pattern** (initial proposal):
   - ❌ Still SQL-centric, no in-memory strategy
   - ❌ Cannot nest/compose filters easily
   - ❌ Cannot express non-SQL filters
   - ✅ Easier to implement initially
   - **Decision**: Rejected in favor of more flexible composable pattern

2. **ORM (diesel/sea-orm)**:
   - ❌ Heavy dependency
   - ❌ Learning curve
   - ❌ Still SQL-only, no composition
   - ❌ Less control over queries

3. **Pure In-Memory Filtering**:
   - ❌ Inefficient for large databases
   - ❌ Must load all books into memory
   - ✅ Very flexible
   - **Decision**: Use as one strategy in smart filter

4. **Pure SQL Approach** (current):
   - ❌ Cannot compose/nest filters
   - ❌ Cannot express complex logic
   - ✅ Efficient for large datasets
   - **Decision**: Use as one strategy in smart filter

### Implementation Plan

**Phase 1: Core Infrastructure (No Breaking Changes)**

Goal: Introduce composable filter architecture without breaking existing code.

Tasks:
1. **Create trait and types** (~200 lines):
   - `filters/trait.rs` - Define `BookFilter` trait with `async_trait`
   - `filters/conditions.rs` - Define `FilterCondition` enum for SQL translation

2. **Implement execution strategies** (~400 lines):
   - `filters/strategies/sql.rs` - `SqlFilter` (wraps current SQL logic)
   - `filters/strategies/memory.rs` - `MemoryFilter` (predicate-based)
   - `filters/strategies/smart.rs` - `SmartFilter` (auto-optimization)

3. **Implement combinators** (~200 lines):
   - `filters/combinators/chain.rs` - `FilterChain` for composition
   - `filters/combinators/and.rs` - AND combinator
   - `filters/combinators/or.rs` - OR combinator

4. **Create fluent API** (~150 lines):
   - `filters/pipeline.rs` - `FilterPipeline` builder

5. **Implement concrete filters** (~300 lines):
   - `AuthorFilter`, `FormatFilter`, `YearFilter`, `TagFilter`, etc.
   - Each implements `BookFilter` trait
   - Each can execute via SQL or memory

6. **Backward compatibility layer** (~100 lines):
   - Reimplement `build_books_query()` using new `SqlFilter`
   - Add deprecation warnings
   - Verify all existing tests still pass

**Phase 2: CLI Migration & Testing (Gradual)**

Goal: Migrate CLI commands to use new composable API.

Tasks:
1. **Update CLI commands**:
   - Migrate `ritmo_cli/src/commands/books.rs` to `FilterPipeline`
   - Migrate `ritmo_cli/src/commands/contents.rs`
   - Keep both old and new API available

2. **Add comprehensive tests**:
   - Test filter composition (nested filters)
   - Test smart execution strategy switching
   - Test in-memory vs SQL execution
   - Test custom filters (MemoryFilter with predicates)
   - Benchmark SQL vs memory execution at various thresholds

3. **Documentation**:
   - Migration guide with examples
   - API documentation with composition examples
   - Performance tuning guide (threshold configuration)

**Phase 3: Advanced Features**

Goal: Leverage composable architecture for powerful new features.

Possible enhancements:
1. **ML-based filters**:
   ```rust
   let similar = SimilarityFilter::new(reference_book, threshold: 0.8);
   let results = tag_filter.then(similar).execute().await?;
   ```

2. **Query optimization**:
   - Analyze filter chain
   - Reorder filters for optimal execution
   - Combine multiple SQL filters into single query

3. **Filter caching**:
   - Cache frequent filter results
   - Invalidate on database changes

4. **SQL DSL** (from roadmap):
   ```rust
   let results = parse_filter_dsl(r#"
       author IN ["King", "Rowling"] AND format = "epub"
   "#)?.execute(&pool).await?;
   ```

5. **Interactive filter builder** (GUI):
   - Visual filter composition
   - Drag-and-drop filter pipeline
   - Real-time result preview

6. **Filter presets v2**:
   - Save entire filter pipelines (not just SQL filters)
   - Include custom memory filters
   - Share presets between users

### Consequences

**Positive:**
- ✅ **Infinite Composability**: Can nest filters arbitrarily deep
- ✅ **Performance Optimization**: Automatic strategy selection (SQL vs memory)
- ✅ **Expressive Power**: Support filters beyond SQL capabilities (ML, custom logic)
- ✅ **Testability**: Test filters with in-memory data, no database needed
- ✅ **Database Agnostic**: Filter logic independent of SQL
- ✅ **Easy to Extend**: Add new filters by implementing simple trait
- ✅ **No Breaking Changes**: Old API continues to work
- ✅ **Foundation for Advanced Features**: DSL, AI search, query optimization all possible
- ✅ **Clear Separation of Concerns**: Filter logic, execution strategy, SQL generation all separate

**Negative:**
- ⚠️ **Increased Complexity**: More abstraction layers (trait, strategies, combinators)
- ⚠️ **Learning Curve**: Contributors need to understand trait, async, composition patterns
- ⚠️ **Potential Performance Issues**:
  - Switching between SQL and memory has overhead
  - Threshold tuning required for optimal performance
  - Multiple small SQL queries might be slower than one large query
- ⚠️ **More Code to Maintain**: ~1000+ new lines across multiple modules
- ⚠️ **Memory Usage**: In-memory filtering loads data into RAM
- ⚠️ **Async Complexity**: All filters must be async (trait objects with async are tricky)

**Neutral:**
- 🔄 Two APIs coexist during migration period
- 🔄 Old API kept for backward compatibility (via wrapper)
- 🔄 Migration can be done incrementally, one command at a time
- 🔄 Performance characteristics change (better for some use cases, worse for others)

**Risks and Mitigations:**

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Performance regression** | High | - Extensive benchmarking (SQL vs memory at various sizes)<br>- Tune threshold based on real data<br>- Allow manual override of strategy<br>- Monitor query performance in production |
| **Complexity overwhelms contributors** | Medium | - Comprehensive documentation with examples<br>- Start with simple filters (author, format)<br>- Gradual migration guide<br>- Keep old API as reference |
| **Memory exhaustion** | Medium | - Set max memory limit for in-memory filtering<br>- Fall back to SQL if dataset too large<br>- Monitor memory usage<br>- Add config option to disable memory strategy |
| **Async trait object issues** | Low | - Use `async_trait` crate (stable, widely used)<br>- Test extensively with different filter combinations<br>- Document async requirements clearly |
| **Bugs in strategy switching** | Medium | - Unit tests for threshold logic<br>- Integration tests with various dataset sizes<br>- Keep old implementation as reference/fallback |
| **SQL optimization lost** | Low | - Smart filter can still combine multiple SQL conditions<br>- Add query optimizer in Phase 3<br>- Profile and optimize hot paths |

**Performance Analysis:**

| Scenario | Current (SQL only) | New (Composable) | Winner |
|----------|-------------------|------------------|--------|
| Initial query (all books) | Fast (SQL index) | Fast (SQL strategy) | 🟰 Tie |
| Filter 100,000 books | Fast (SQL) | Fast (SQL strategy) | 🟰 Tie |
| Filter 50 books subset | Slow (SQL overhead) | **Fast (memory)** | ✅ New |
| Complex non-SQL filter | Impossible | **Possible (memory)** | ✅ New |
| Nested filters | Impossible | **Possible** | ✅ New |
| Multiple independent filters | 1 query (optimal) | N queries (suboptimal) | ⚠️ Current |

**Overall**: New architecture has better expressiveness and flexibility, with comparable or better performance in most scenarios. The risk of performance regression exists for some edge cases and must be monitored.

### Implementation Details

**BookFilter Trait (Complete Design)**
```rust
// filters/trait.rs
use async_trait::async_trait;
use ritmo_db::BookResult;
use ritmo_errors::RitmoResult;

#[async_trait]
pub trait BookFilter: Send + Sync {
    /// Apply filter to a list of books
    /// If input is empty, should query all books from database
    async fn apply(&self, books: Vec<BookResult>) -> RitmoResult<Vec<BookResult>>;

    /// Check if filter supports SQL execution
    fn supports_sql(&self) -> bool { false }

    /// Convert filter to SQL conditions (if supports_sql() == true)
    fn to_sql_conditions(&self) -> Option<Vec<FilterCondition>> { None }

    /// Estimate result size (for optimization)
    fn estimate_selectivity(&self) -> f64 { 0.5 }  // Default: 50% selectivity
}

/// Helper trait for composing filters
pub trait BookFilterExt: BookFilter {
    /// Chain another filter after this one
    fn then(self, next: impl BookFilter + 'static) -> FilterChain
    where
        Self: Sized + 'static,
    {
        FilterChain::new(vec![Box::new(self), Box::new(next)])
    }

    /// Combine with another filter using AND logic
    fn and(self, other: impl BookFilter + 'static) -> AndFilter
    where
        Self: Sized + 'static,
    {
        AndFilter::new(Box::new(self), Box::new(other))
    }

    /// Combine with another filter using OR logic
    fn or(self, other: impl BookFilter + 'static) -> OrFilter
    where
        Self: Sized + 'static,
    {
        OrFilter::new(Box::new(self), Box::new(other))
    }
}

// Blanket implementation
impl<T: BookFilter> BookFilterExt for T {}
```

**Concrete Filter Example: AuthorFilter**
```rust
// filters/strategies/sql.rs (or filters/implementations/author.rs)
pub struct AuthorFilter {
    authors: Vec<String>,
    pool: SqlitePool,
    threshold: usize,  // When to switch from SQL to memory
}

impl AuthorFilter {
    pub fn new(pool: SqlitePool, authors: Vec<String>) -> Self {
        Self {
            authors,
            pool,
            threshold: 1000,  // Default threshold
        }
    }

    /// SQL execution: query database
    async fn apply_sql(&self, book_ids: Option<Vec<i64>>) -> RitmoResult<Vec<BookResult>> {
        let conditions = vec![FilterCondition::AuthorNameIn(self.authors.clone())];
        let (query, params) = build_sql_query(&conditions, book_ids)?;
        execute_query(&self.pool, query, params).await
    }

    /// Memory execution: filter in-memory
    async fn apply_memory(&self, books: Vec<BookResult>) -> RitmoResult<Vec<BookResult>> {
        Ok(books
            .into_iter()
            .filter(|book| {
                // Check if any book author matches filter
                book.authors.iter().any(|author| {
                    self.authors.iter().any(|filter_author| {
                        author.to_lowercase().contains(&filter_author.to_lowercase())
                    })
                })
            })
            .collect())
    }
}

#[async_trait]
impl BookFilter for AuthorFilter {
    async fn apply(&self, books: Vec<BookResult>) -> RitmoResult<Vec<BookResult>> {
        if books.is_empty() {
            // First filter in chain: use SQL
            self.apply_sql(None).await
        } else if books.len() > self.threshold {
            // Large dataset: use SQL with WHERE id IN (...)
            let book_ids: Vec<i64> = books.iter().map(|b| b.id).collect();
            self.apply_sql(Some(book_ids)).await
        } else {
            // Small dataset: filter in memory (faster!)
            self.apply_memory(books).await
        }
    }

    fn supports_sql(&self) -> bool {
        true
    }

    fn to_sql_conditions(&self) -> Option<Vec<FilterCondition>> {
        Some(vec![FilterCondition::AuthorNameIn(self.authors.clone())])
    }

    fn estimate_selectivity(&self) -> f64 {
        // Estimate: author filter typically selects 1-5% of books
        0.03
    }
}
```

**SmartFilter with Automatic Optimization**
```rust
// filters/strategies/smart.rs
pub struct SmartFilter {
    pool: SqlitePool,
    conditions: Vec<FilterCondition>,
    threshold: usize,
}

impl SmartFilter {
    /// Create smart filter with automatic strategy selection
    pub fn new(pool: SqlitePool, conditions: Vec<FilterCondition>) -> Self {
        Self {
            pool,
            conditions,
            threshold: 1000,  // Configurable
        }
    }

    /// Decide which strategy to use
    fn choose_strategy(&self, input_size: usize) -> ExecutionStrategy {
        if input_size == 0 {
            // No input: use SQL to query entire database
            ExecutionStrategy::Sql
        } else if input_size > self.threshold {
            // Large input: use SQL with WHERE id IN (...)
            ExecutionStrategy::SqlSubset
        } else {
            // Small input: filter in memory
            ExecutionStrategy::Memory
        }
    }
}

#[async_trait]
impl BookFilter for SmartFilter {
    async fn apply(&self, books: Vec<BookResult>) -> RitmoResult<Vec<BookResult>> {
        match self.choose_strategy(books.len()) {
            ExecutionStrategy::Sql => {
                let (query, params) = build_sql_query(&self.conditions, None)?;
                execute_query(&self.pool, query, params).await
            }
            ExecutionStrategy::SqlSubset => {
                let book_ids: Vec<i64> = books.iter().map(|b| b.id).collect();
                let (query, params) = build_sql_query(&self.conditions, Some(book_ids))?;
                execute_query(&self.pool, query, params).await
            }
            ExecutionStrategy::Memory => {
                // Convert SQL conditions to memory predicates
                let predicate = self.conditions_to_predicate(&self.conditions)?;
                Ok(books.into_iter().filter(predicate).collect())
            }
        }
    }
}

enum ExecutionStrategy {
    Sql,        // Query entire database
    SqlSubset,  // Query with WHERE id IN (...)
    Memory,     // Filter in memory
}
```

### Migration Timeline

**Week 1-2: Core Infrastructure**
- Implement `BookFilter` trait (~100 lines)
- Implement `FilterCondition` enum (~200 lines)
- Implement `SqlFilter`, `MemoryFilter`, `SmartFilter` strategies (~400 lines)
- Implement `FilterChain`, `AndFilter`, `OrFilter` combinators (~200 lines)
- Add comprehensive unit tests (~300 lines)

**Week 3: Concrete Filters**
- Implement `AuthorFilter`, `PublisherFilter`, `SeriesFilter` (~600 lines)
- Implement `FormatFilter`, `YearFilter`, `DateFilter` (~300 lines)
- Implement `TagFilter` (~100 lines)
- Test each filter independently

**Week 4: Backward Compatibility**
- Reimplement `build_books_query()` using new filters
- Add deprecation warnings to old API
- Verify all existing tests still pass
- Create migration guide

**Week 5-6: CLI Migration**
- Update `list-books` command to use `FilterPipeline`
- Update `list-contents` command
- Add examples to documentation
- Performance benchmarking (SQL vs memory at various sizes)

**Week 7+: Advanced Features**
- Query optimizer (reorder filters, combine SQL filters)
- Filter caching
- ML-based filters
- DSL parser (optional)

### Success Criteria

**Phase 1 (Core Infrastructure):**
1. ✅ `BookFilter` trait compiles and is async-safe
2. ✅ All three execution strategies (SQL, Memory, Smart) implemented
3. ✅ Filter combinators (`FilterChain`, `AndFilter`, `OrFilter`) work correctly
4. ✅ Unit tests for each strategy pass independently
5. ✅ All existing tests pass without modification

**Phase 2 (Concrete Filters):**
1. ✅ At least 6 concrete filters implemented (Author, Publisher, Series, Format, Year, Tag)
2. ✅ Each filter supports both SQL and memory execution
3. ✅ Filters are composable (can be chained together)
4. ✅ Old API (`build_books_query`) still works via wrapper

**Phase 3 (Migration & Performance):**
1. ✅ CLI commands migrated to use `FilterPipeline`
2. ✅ Performance benchmarks show:
   - SQL strategy within 5% of current implementation for large datasets
   - Memory strategy 2-10x faster for small datasets (<1000 books)
   - Smart strategy chooses correct execution path >95% of time
3. ✅ Documentation includes:
   - Migration guide with before/after examples
   - API reference for all filters
   - Performance tuning guide (threshold configuration)
4. ✅ At least one advanced feature implemented:
   - Nested filtering works
   - Custom `MemoryFilter` with user-defined predicates
   - OR ML-based similarity filter

**Phase 4 (Advanced Features - Optional):**
1. ⭕ Query optimizer can combine multiple SQL filters into single query
2. ⭕ Filter caching reduces redundant queries
3. ⭕ DSL parser for text-based filter expressions
4. ⭕ ML-powered filters (similarity, recommendations)

### Example Usage Scenarios

**Scenario 1: Simple Filter (same as current)**
```rust
// Old API (deprecated)
let filters = BookFilters::new().with_author(vec!["King".to_string()]);
let books = execute_books_query(&pool, &filters).await?;

// New API (recommended)
let books = AuthorFilter::new(pool.clone(), vec!["King".to_string()])
    .apply(Vec::new())
    .await?;

// Or using pipeline
let books = FilterPipeline::new(pool)
    .author(vec!["King".to_string()])
    .execute()
    .await?;
```

**Scenario 2: Nested Filtering (NEW - impossible before)**
```rust
// Get all sci-fi books (SQL - efficient)
let scifi_books = FilterPipeline::new(pool.clone())
    .tag(vec!["sci-fi".to_string()])
    .execute()
    .await?;

// Filter sci-fi books by author (memory - fast because small dataset)
let king_scifi = AuthorFilter::new(pool.clone(), vec!["King".to_string()])
    .apply(scifi_books)
    .await?;

// Or as pipeline
let king_scifi = FilterPipeline::new(pool)
    .tag(vec!["sci-fi".to_string()])
    .author(vec!["King".to_string()])
    .execute()
    .await?;
```

**Scenario 3: Custom Complex Filter (NEW)**
```rust
// ML-based similarity filter (can't be expressed in SQL!)
let reference_book = get_book(42).await?;
let similarity_filter = MemoryFilter::new(Box::new(move |book| {
    compute_ml_similarity(book, &reference_book) > 0.8
}));

// Combine SQL and custom filter
let similar_books = FilterPipeline::new(pool)
    .tag(vec!["fantasy".to_string()])  // SQL: narrow down to fantasy
    .custom(similarity_filter)          // Memory: ML similarity
    .execute()
    .await?;
```

**Scenario 4: Performance Optimization (automatic)**
```rust
// System automatically chooses best strategy:

// Case A: Empty input → SQL query (efficient)
let all_king = AuthorFilter::new(pool, vec!["King".to_string()])
    .apply(Vec::new())  // Empty = query all
    .await?;

// Case B: Large dataset (>1000) → SQL with WHERE id IN (...)
let large_subset = get_books(0..10000).await?;  // 10,000 books
let filtered = AuthorFilter::new(pool, vec!["King".to_string()])
    .apply(large_subset)  // Uses SQL with id filter
    .await?;

// Case C: Small dataset (<1000) → in-memory filtering (fastest!)
let small_subset = get_books(0..50).await?;  // 50 books
let filtered = AuthorFilter::new(pool, vec!["King".to_string()])
    .apply(small_subset)  // Uses memory filtering
    .await?;
```

**Scenario 5: Complex Composition**
```rust
// Chain multiple filters with automatic optimization
let books = FilterPipeline::new(pool)
    .tag(vec!["sci-fi".to_string()])           // SQL
    .author(vec!["Asimov", "Clarke"])          // SQL or memory (depends on first result)
    .year_between(1950, 1980)                  // SQL or memory
    .format(vec!["epub", "pdf"])               // Memory (likely small dataset by now)
    .execute()
    .await?;

// System automatically:
// - Combines SQL filters if all results not yet loaded
// - Switches to memory filtering for later stages
// - Optimizes query order based on selectivity estimates
```

### Related Decisions

- See ADR-002: Filter System as Core Identity
- See ADR-003: OR Logic for Repeated Parameters

### References

- Current implementation: `ritmo_db_core/src/filters/`
- Usage examples: `ritmo_cli/src/commands/books.rs`, `ritmo_cli/src/commands/contents.rs`
- Related documentation: `docs/filters.md`
- Session history: `docs/sessions/2026-01-sessions.md`

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

**Last Updated:** 2026-01-25
**Contributors:** Emanuele (maintainer)
**Repository:** ritmo - Rust Library Management System

## Notes

This document follows the Architecture Decision Records (ADR) pattern to track important decisions over time. Each decision has:
- **Context**: Why a decision is needed
- **Decision**: What was decided
- **Rationale**: Why this choice
- **Consequences**: Impact of the decision

Decisions are not immutable - they can be re-evaluated (status SUPERSEDED) when context changes.