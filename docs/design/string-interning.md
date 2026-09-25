# String Interning for the Schema Representation

Status: **resolved — symbol-interned `Name` ships in 2.0**
Branch: `tninesling/interned-names` (rebased onto `v2`; see "End state"
below for what actually landed).

## Outcome

After building and measuring every design in this document, the decision
landed on classic symbol interning with a freezable global table:

- `Name` interns its string into a process-global symbol table; equality
  is an integer comparison and `Hash` emits the 4-byte symbol. This is a
  breaking change (`Borrow<str>` lookups become `NameKey` probes; `name!()`
  in `const` items becomes `static`; `Name` grows 24→32 bytes).
- **Measured on production supergraphs and real operation corpora: −15%
  on the per-request parse+validate path** (up to −29 to −41% on
  abstract-type-heavy validation microbenches), reproduced after rebasing
  onto the `Node`/`Component` consolidation. `freeze_interning()` after
  schema build makes the table lookup-only and the post-freeze read path
  lock-free: hostile documents cannot grow it (0 bytes across 50k
  unique-alias requests), and reloads degrade gracefully.
- The complementary **shared-seed foldhash hasher swap** (−7 to −13% on
  composition, −3% on the request path) is deliberately *not* part of this
  branch, to keep it scoped to interning. It lives standalone on
  `tninesling/collections-hasher` (off `main`, shippable on 1.x), and
  composition can capture most of that win federation-side by using its
  own foldhash-backed collection aliases instead of apollo-compiler's.
- Earlier candidates — the arc-interner with pointer-equality
  (`tninesling/string-interning-archive`) and the `HashedName` federation
  migration — were built, measured, and rejected; their post-mortems are
  in the findings below.

The rest of this document is the investigation record.

## Motivation

Consumers of `apollo-compiler` — most importantly the router (query planning,
response validation, introspection) and composition — spend a measurable amount
of time comparing and hashing GraphQL names:

- Every `Name` equality check is a full string comparison (`as_str() == as_str()`).
- Every map lookup keyed by `Name` (fields of a type, types of a schema,
  arguments of a directive, …) hashes the full string with `ahash`.
- Every parsed name allocates a fresh `Arc<str>` in
  `cst::Name::convert` (`ast/from_cst.rs`), even though real documents repeat
  the same handful of names constantly (`id`, `__typename`, type names shared
  between a schema and the operations validated against it).

Interning gives us three wins:

1. **O(1) equality**: interned names that are equal share a pointer, so `Eq`
   becomes a pointer comparison with a string-compare fallback.
2. **Cached hashing where opted in**: the hash can be computed once at intern
   time and cached. The shared schema maps keep byte-based hashing to preserve
   by-`&str` lookups (see the `Borrow<str>` decision below), so this win
   applies to `Name`-keyed side tables that opt into a pass-through hasher.
3. **Memory**: one allocation per distinct string per process instead of one
   per occurrence, and cheaper `Clone` pressure on the allocator.

## Current representation

`Name` (`crates/apollo-compiler/src/name.rs`) is a 24-byte packed struct:

```
ptr: NonNull<u8>          // Arc<str> data pointer OR &'static str pointer
len: u32
start_offset: u32         // source location (0 = none)
tagged_file_id: TaggedFileId  // 1-bit tag (ARC vs STATIC) + FileId
```

Key properties we must preserve:

- `Deref<Target = str>`, `AsRef<str>`, `Borrow<str>` — `Name` is usable as a
  string everywhere, with no interner handle needed to resolve it. This rules
  out classic `u32`-symbol interners: the public API would break completely.
- `Hash for Name` hashes the string bytes, and `Borrow<str>` lets callers look
  up `IndexMap<Name, V>` / `HashMap<Name, V>` entries by `&str`. **Any change
  to `Hash` must keep `hash(name) == hash(name.as_str())` under the map's
  hasher, or every by-`&str` lookup silently breaks.** This is the central
  constraint on hash caching (see the `Borrow<str>` decision below).
- `name!()` produces `const` static-tagged names with zero refcounting.
- Locations are carried inline and excluded from `Eq`/`Hash`.

The dual `ARC`/`STATIC` tag is the hook that makes interning almost invisible:
`Name` already supports two backing representations behind one pointer, so an
interned name is just "an `Arc<str>` that happens to be shared".

## Design

### Phase 1 — the interner, pointer-equality fast path

Add a process-global interner:

```rust
// name/interner.rs (sketch)
static INTERNER: OnceLock<Interner> = OnceLock::new();

struct Interner {
    shards: [Mutex<HashSet<Entry>>; N],  // shard by hash to limit contention
}
```

- **Entries are the packed `(ptr, len, tag)` repr**, not `Arc<str>` directly,
  so the interner can hold both heap entries and the pre-seeded static
  entries with one table.
- **Pre-seed with well-known names**: built-in scalars, introspection names
  (`__typename`, `__Schema`, …), `Query`/`Mutation`/`Subscription`, directive
  names, and everything else `name!()`-declared inside the crate. Parsed
  occurrences of `"String"` then get the static-tagged entry, and
  `parsed_name == name!("String")` hits the pointer fast path.
- **Intern at the choke points**: `cst::Name::convert` (all parsed names flow
  through here), `Name::new`, `Name::try_from(...)`. `with_location` already
  layers location onto the shared pointer without touching the string, so
  interned names still carry their own source spans.
- **Equality fast path**:

  ```rust
  fn eq(&self, other: &Self) -> bool {
      self.ptr == other.ptr || self.as_str() == other.as_str()
  }
  ```

  Correct even with a mix of interned and non-interned names; strictly faster
  whenever both sides are interned (the common case after this change).

`Hash` stays byte-based in Phase 1 so the `Borrow<str>` contract is untouched.
Phase 1 is semver-compatible: no public API changes, only behavior.

#### DoS containment: lookup-only interning for untrusted input

The router parses **adversarial input**. A naive intern-everything table turns
unique field aliases in hostile queries into unbounded memory growth — a DoS
vector. Rather than solving that with eviction machinery, split interning into
two modes:

- **Insert mode** — trusted, bounded paths only: `name!()` statics, built-in
  names, and schema construction (`Schema::parse` / builder). The table grows
  only with names that actually exist in some schema, and schemas are
  operator-controlled and small.
- **Lookup-only mode** — request-path parsing (executable documents): probe
  the table; on a hit, return the shared pointer; on a miss, return a plain
  un-interned `Arc<str>` exactly as today. Hostile input can never insert.

This eliminates the unbounded-growth problem *structurally*: no eviction, no
drop-path locking, no refcount races. The misses are names that appear in a
query but in no schema — aliases, variable names, fragment names — which are
exactly the names that repeat too rarely to benefit from interning. Every name
that participates in hot schema-map lookups (type names, field names,
directive names) is by definition in the schema, so it hits.

Schemas themselves can be dropped and re-created (router hot-reload,
composition runs), so insert-mode entries still want cleanup. Options, in
order of preference:

1. **Weak entries + amortized sweep**: store weak references; sweep dead
   entries when a shard's size doubles. Insert mode is rare (schema loads), so
   sweep cost is irrelevant, and floating garbage is bounded by dropped
   schemas, not request traffic.
2. **Refcount-based removal** (`internment::ArcIntern` style): deterministic,
   but puts a check on every `Name` drop — only if 1 proves insufficient.

#### Threading: global static, not thread- or task-local

The interner is a `static` (e.g. `OnceLock<Interner>` with sharded locks) and
is fully shared across threads; `Name` remains `Send + Sync`.

Thread-local or tokio-task-local interners look attractive for DoS isolation
(per-task tables die with the task) but don't work here:

- **Interned names escape their task.** Parsed documents are cached (router's
  query-plan and parsed-operation caches) and shared across requests via
  `Arc`; tokio work-stealing moves them between threads. A table scoped to
  the parsing task can't own allocations that outlive the task, so entries
  could never be reclaimed on task exit anyway — reintroducing the same
  lifetime problem with extra steps.
- **The payoff requires one table.** The hot comparisons are *cross-document*:
  query names vs. schema names during validation and planning. The schema is
  interned once at startup on a different task than every request; pointer
  equality only happens if both sides resolved through the same table.

Lookup-only mode (above) is what addresses the DoS concern task-locality was
aiming at: the request path gets read-mostly access to a table it cannot
grow. Shards can use `RwLock` so concurrent request parsing takes only read
locks, making contention a non-issue on the hot path.

### The `Borrow<str>` decision: keep it

Cached hashes and by-`&str` map lookups are in direct tension: `Borrow<str>`
requires `hash(name) == hash(name.as_str())` under the map's hasher, and
`str`'s `Hash` impl feeds bytes to the hasher, so `Name` cannot emit a cached
`u64` in the shared maps without breaking every `map.get("Query")`-style
lookup.

We considered breaking it and surveyed the router workspace (2026-08,
non-test code) to size the blast radius:

- *apollo-federation is already `&Name`-native.* Its position types carry
  `type_name: Name` / `field_name: Name`; ~95 of ~110 direct map lookups pass
  `&Name`. The `&str` residue is cold, one-time composition code.
- *apollo-router uses by-string lookup as a first-class pattern on hot
  paths*, because the string comes out of JSON and has no `Name`:
  subgraph-response `__typename` values driving `schema.types.get(...)` /
  `is_subtype(...)` per response object (`spec/query.rs`, `json_ext.rs`,
  `response_cache/plugin.rs`), and request-variable object keys driving input
  validation (`spec/field_type.rs`). Also ~110 call sites of apollo-compiler's
  own `&str`-signature helpers (`is_subtype`, `get_object`, `type_field`, …).

**Decision: keep `Borrow<str>` and the `&str` helper signatures.** The
by-string pattern is legitimate and widespread; if our own flagship consumer
leans on it this heavily, external consumers do too, and breaking it would be
gratuitously disruptive. Consequence: `Hash for Name` stays byte-based and the
shared schema maps keep ahash — **no cached-hash win inside apollo-compiler's
public maps**. What interning still buys, untouched:

- pointer-equality fast path on `Eq` (the field-merging / `is_subtype` /
  validation comparison hot paths),
- one allocation per distinct name instead of per occurrence,
- cheap `Name` clones and lower allocator pressure,
- and the downstream opt-in paths below.

### Phase 2 — downstream adoption (opt-in, non-breaking)

The remaining wins move to additive API, adopted where profiles justify it:

1. **Convert JSON strings to `Name` at the boundary.** This is the biggest
   remaining router win. Expose:

   ```rust
   /// Returns the interned Name for this string, if one exists.
   /// Hit ⇒ shared pointer, no allocation — and validity is free,
   /// because the table only contains valid schema names.
   pub fn Name::interned(value: &str) -> Option<Name>
   ```

   The router converts each response object's `__typename` (and entity
   representations, enum values, variable keys) **once**, at the JSON
   boundary: one byte-hash to probe the lookup-only table, then every
   downstream comparison and map probe on that object rides pointer equality
   instead of re-hashing/re-comparing the string at each of the N places the
   typename is consulted. A miss means "not a name in any schema", which
   those paths already treat as an error/skip case — today they discover it
   later via a failed map lookup. Combined with a small per-response memo
   (`ByteString` ptr → `Name`), repeated typenames in a response cost one
   probe total.
2. **Cached hashes for `Name`-keyed side tables.** Store the hash in the
   interned allocation header (we already use `triomphe::HeaderSlice` for
   `Node`) and expose `Name::cached_hash() -> u64` plus a pass-through
   `BuildHasher`. Downstream maps that are keyed by `Name` and never queried
   by `&str` — federation's position-keyed structures, router's own caches —
   opt in and get O(1) hashing. apollo-compiler's public maps are untouched,
   so nothing breaks; if profiles later show big wins we can revisit
   converting specific *internal* maps.
3. **Pre-interned constants for hot literals.** The router has ~97
   `name == "literal"` comparisons (dominated by `== "__typename"` in
   response-selection filtering and cost calculation). `PartialEq<str>` can't
   use pointer equality; migrating those sites to exported `name!()`
   constants (`apollo_compiler::names::TYPENAME`, …) turns them into pointer
   compares.
4. **`Name::ptr_eq`** for callers that want to assert the fast path.

Additive `&Name` overloads of the `&str` helpers (e.g.
`Schema::get_object(&Name)` via an `impl AsRef<str>`-style or dedicated
method) can come later if profiles show the `&str` signatures forcing
`.as_str()` round-trips to matter; keeping the `&str` versions means this is
never forced on anyone.

## Implementation order

1. Extract an `interner` module; land it unused with unit + concurrency tests.
2. Pre-seed statics; wire schema construction through insert mode and
   `Name::new` / `from_cst` / `TryFrom` through lookup-only mode.
3. Switch `PartialEq for Name` to the pointer fast path.
4. Benchmark (see below). Phase 1 ships if compiler benches are neutral-or-
   better and router benches improve.
5. Phase 2 additive API: `Name::interned`, `Name::cached_hash` + pass-through
   hasher, `names::` constants, `Name::ptr_eq`. All non-breaking.
6. Router/composition adoption PRs, driven by profiles: JSON-boundary `Name`
   conversion first (`spec/query.rs`, `json_ext.rs`, `response_cache`), then
   literal-comparison constants, then cached-hash side tables.

Each step is a separate PR well under the 500-line guideline.

## Validation

### Correctness

- Full workspace suite (`cargo nextest run`) — interning must not change any
  test result or snapshot (locations, diagnostics,
  serialization are all location-carrying and must be unaffected).
- **Fuzzing**: existing `fuzz/` targets (`parser`, `one_of`, …) exercise
  parse → validate → serialize round-trips; run extended sessions and add an
  interner-focused target that parses documents on multiple threads and
  asserts `a == b ⇔ a.as_str() == b.as_str()` for observed name pairs.
- **Concurrency**: stress test parsing the same schema from N threads
  (pointer-dedup under contention), plus `cargo miri` on the interner module's
  unit tests — the `Name` repr is already `unsafe`-heavy and interning adds
  lifecycle subtlety at the drop/remove race.
- **DoS check**: test that parses 10k executable documents full of unique
  aliases/names and asserts the interner never grows beyond the pre-seeded +
  schema baseline (lookup-only mode cannot insert); plus a schema-churn test
  that builds and drops many schemas and asserts sweeps reclaim their entries.
- **`Borrow<str>` contract preserved**: property test asserting
  `m.get(s) == m.get(&Name::new(s).unwrap())` across random maps — by-string
  lookups must behave identically before and after interning.
- **`Name::interned` correctness** (Phase 2): property test asserting
  `Name::interned(s)` returns `Some(n)` with `n == s` iff `s` was interned in
  insert mode, and `None` never occurs for schema-declared names.

### Performance

All measurements on P-cores via `taskpolicy -c utility`, comparing `main` against each phase.

1. **Compiler microbenches**: existing `benches/` (`fields_validation`,
   `directives_validation`, `fragments_validation`, `multi_source`) cover the
   validation paths where name hashing dominates. Add two benches:
   - parse+build a large real supergraph schema (allocation-heavy path — this
     is where interner lock cost would show up),
   - repeated executable-document validation against a fixed schema (the
     router-shaped workload where cross-document pointer equality pays off).
2. **Profiles**: `samply` on the new benches before/after; confirm
   `Name::eq` / hashing frames shrink and no new lock frames appear hot.
3. **Allocation counts**: run the parse bench under `dhat` (dev-dependency,
   feature-gated) and record total allocations + peak bytes. Interning should
   cut name allocations roughly by the repetition factor of the document.
4. **Downstream validation** (the actual success criterion): patch router and
   composition to this branch via `[patch.crates-io]` and run their existing
   benchmark suites (router query-planning benches, composition benches on
   large supergraphs). Target: measurable wins on planning/validation
   latency; hard requirement: no regression > noise on any bench.
5. **Contention check**: router-side load test with concurrent query parsing
   (many worker threads) to confirm the frozen snapshot keeps parsing
   lock-free.

### Findings so far (2026-08-14, Phase 1 draft)

Measured on the Phase 1 draft (global interner + pointer-eq fast path,
`Hash` unchanged), P-cores via `taskpolicy -c utility`.

**Compiler microbenches** (criterion, vs `main` baseline):

- Improved: `many_same_directive_query` −6.8%, `many_same_nested_field`
  −2.7% — executable documents validated against a fixed schema, the
  cross-document pointer-eq case.
- Regressed: benches that re-parse and rebuild *the schema* every iteration
  — `simple_query parse_and_validate` +14% (tiny doc, fixed overhead most
  visible), `supergraph parse_and_validate` +6.7%, `many_same_directive`
  +7.3%. An early-exit read-lock fast path in `insert` already removed a
  +5.4% regression in `many_extensions`.
- Everything else within the ±3% run-to-run noise band.

**Composition** (apollo-federation via `supergraph-smith` `scale_stress`,
30 subgraphs / 500 object types, medians of 5 runs): no wins anywhere;
+9.6% on a ~200 ms compose (seed 44), noise on a ~500 ms and an ~11 s
compose. Profiling and a same-binary kill-switch A/B
(`APOLLO_DISABLE_INTERNER`) decomposed the 9.6%:

- Interner runtime cost is ~5 ms per composition — a fixed cost that is ~3%
  of a short compose and invisible in a long one. Zero
  `interner`/rwlock/weak-upgrade frames in >10k profile samples.
- The remainder is binary-level difference (eq fast-path code, layout luck),
  not interner runtime.
- Compose hot frames are federation query-graph construction, position
  lookups, and allocator traffic — hash-map-dominated paths that byte-based
  hashing keeps unchanged. **Composition cannot benefit from pointer-eq
  interning; it needs the Phase 2 cached-hash adoption.**

Implications: Phase 1's costs are small, fixed, and confined to schema
builds; its wins are small and confined to request-validation eq paths. The
performance case rests on Phase 2 (cached hashes in `Name`-keyed side
tables, `Name::interned` at the router's JSON boundary) and on router
request-path benchmarks that these workloads don't cover.

**Follow-up (same day): the transparent hasher lever.** Profiling compose
showed two hasher-level costs orthogonal to interning: per-map
`ahash::RandomState` construction (compose builds thousands of short-lived
maps) and byte hashing of short names on every map op. Switching the
`collections::` aliases to a process-shared-seed `foldhash` state fixed
both with zero consumer code changes (federation's 135 importing files
compile unmodified):

- compose 30 subgraphs/500 types: **−7.6%** (~190 → ~175 ms); pathological
  11 s compose: **−13%**; planner construction −5–8%.
- compiler microbenches: field-merging suites −5–6%, others neutral
  (`many_types` wobbled +4% in one run — re-check).

**Follow-up 2: the full `HashedName` migration, measured and rejected.** We
ran the experiment the size question demanded: converted every federation
position type's name fields and the `Referencers` maps to `HashedName`
(branch `tninesling/hashed-name-experiment` in the federation repo —
50 files, ±1150 lines, largely auto-generated wrapping). Result, on top of
the transparent levers:

- compose seed 44: **+2%** (median 175.3 vs 171.5 ms) — slower;
- pathological seed 43: +2% (9.77 vs 9.54 s) — no win;
- planner construction: flat to slightly worse.

Why it loses: positions are constructed *transiently* all over composition
and planning, so `HashedName::new` re-hashes far more often than the cached
hash gets reused in map ops — and once the shared-seed foldhash made byte
hashing of short names cheap, the per-op saving left to capture was too
small to pay for the extra hashing at construction plus the
lookup-boundary conversions.

Conclusion: ship the transparent levers (interning + shared-seed
foldhash); keep `HashedName` as opt-in API for long-lived key sets where a
stored key really is reused across many map ops, and do not pursue a
blanket federation migration.

### Follow-up 3: classic symbol interning (drop `Borrow<str>`), measured

Branch `tninesling/symbol-interning` (+ `tninesling/symbol-name-experiment`
in the federation repo): `Name` keeps its string pointer (`Deref`,
`as_str`, serialization unchanged) and adds a lazily-interned `NonZeroU32`
symbol from a global table; `Eq` is an integer compare, `Hash` emits the
4-byte symbol, `Borrow<str>` is removed and replaced by a read-only
`NameKey` probe key (`indexmap::Equivalent`).

Change surface: apollo-compiler 18 files / ±350 lines; federation 21
files / ±70 lines — far smaller than the `HashedName` migration because
`Eq`/`Hash` change transparently under every existing `Name`-keyed map;
only by-string lookups need edits.

Results (same interleaved methodology):

- **Executable validation against a many-abstract-types schema
  (`many_types`): −41%** vs main (foldhash alone was flat on this bench) —
  the router-request-shaped workload; other validation benches −2 to −4%.
- Composition: ~0 to −2% over the foldhash hasher swap (compose is
  allocator/graph-bound, confirming earlier profiles).
- Planner construction: flat.

**Reclamation: solved by freezing.** `freeze_interning()` makes the table
lookup-only. The invariant that makes this sound: with a frozen table,
hit-or-miss is a pure function of the string, so equal strings always
agree on which branch they take. Post-freeze misses fall back to
string-based `Eq`/`Hash` (a `PROBED_MISS` sentinel distinguishes
"probed and absent" from "not yet probed" for lazy `name!()` statics),
and a mixed comparison is unequal by construction. Consequences:

- hostile executable documents cannot insert: 0 bytes of live growth
  across 50k unique-alias requests in the memory harness; the table's
  size is a function of the schemas alone, no reclamation needed;
- schema hot-reload degrades gracefully: genuinely-new names use string
  semantics (today's cost), unchanged names keep their symbols;
- the freeze must happen-before untrusted parsing (a data race at the
  freeze instant could violate purity; fine at server startup).

Remaining before this could ship:

- the breaking change itself: every consumer's by-string map lookup
  breaks (compile-time, fixable with `NameKey`), and `name!()` in `const`
  items must become `static` (interior mutability);
- `Name` grows 24→32 bytes;
- ~~re-verify bench numbers on a quiet machine~~ done: with freeze
  sentinels, `many_types` is **−29%** vs main (−36% pre-freeze, so the
  sentinel branches cost ~7 points of the win); other validation benches
  are flat-to-+4% (within the ±3–5% machine noise band).

**Router pipeline (apollo-router `basic_composition`, full request →
plan → mock subgraphs → response), measured:** adapting apollo-router
took 12 files / ±40 lines (mostly `NameKey`, plus `freeze_interning()`
after schema build). Result: **tie** — best-vs-best 166.7 vs 168.2 µs,
and the bench's noise floor (±30% swings from async machinery) swamps
any single-digit effect. The tiny fixture graph spends its time in tokio,
serde and mock services, not name operations; the −29% validation win is
real at the compiler layer but invisible in this end-to-end fixture.

**Production supergraphs + real operations, measured** (freeze after
schema build; interleaved runs; min-of-passes as the robust statistic;
identical valid/invalid counts across variants):

| Corpus | main | foldhash branch | symbol branch |
| --- | --- | --- | --- |
| 4,226 types, 1,927 ops | 352 ms | 341 ms (−2%) | **299 ms (−15%)** |
| 11,243 types, 26,672 ops | 5.72 s | 5.22 s (−9%) | **4.99 s (−13%)** |

Every symbol-branch run beat every main/foldhash run, with visibly
tighter variance. Per-operation validation cost on the first corpus:
~183 µs → ~155 µs.

**Attribution between the two stacked changes** (three-way on the
`spec-plus-nodes` base: base → +foldhash → +interning, min-of-passes):

| Corpus | foldhash alone | interning on top | total |
| --- | --- | --- | --- |
| 4,226 types / 1,927 ops | −3% | −15% | **−18%** |
| 11,243 types / 26,672 ops | −2% | −6% | **−7%** |

On the request path, interning is the dominant contributor (~4/5 of the
win); foldhash's main value remains composition-shaped workloads that
churn many short-lived maps (−7 to −13% there), which the request path
doesn't exercise.

Rebased onto the `Node`/`Component` consolidation
(`tninesling/interned-names` off `tninesling/spec-plus-nodes`) and
productionized — single-`RwLock` table with table-owned storage,
documented freeze contract, lock-free snapshot reads after freeze — the
win reproduces: −13 to −14% on the production corpus in every run.

Side-finding while benchmarking: the September 2025 validation rules
reject at least one real production supergraph (`@deprecated` on
implementing fields whose interface fields are not deprecated), which is
direct evidence for the compatibility-flag discussion on the
spec-validation-rules PR.

Verdict: on real production workloads, symbol interning is worth
**−13 to −15%** on the per-request parse+validate path (the −29 to −41%
microbench shape, diluted by parse time), on top of the foldhash win.
Compose, planning, and tiny-graph end-to-end latency are unmoved.
Freezing removes the DoS/reclamation objection cleanly, and the
migration cost measured small (compiler ±350 lines, federation ±70,
router ±40, all compile-time-guided). The breaking change now has a
quantified case.

### End state (2026-09-25)

What actually landed on `tninesling/interned-names`, rebased onto `v2`:

- **`src/symbol.rs`, ~120 lines**: one
  `RwLock<collections::HashMap<Box<str>, NonZeroU32>>` (ahash, same as the
  public collections), an `AtomicU32` symbol counter, the freeze flag, and
  an immutable `SNAPSHOT` captured at freeze time. Pre-freeze reads and
  inserts take the lock; post-freeze probes read the snapshot with no lock
  at all.
- **Simplifications from the productionized draft**: the 16-shard table
  and the internal foldhash hasher were removed before merge. The snapshot
  already makes the post-freeze request path lock-free, so sharding only
  served pre-freeze interning (schema builds, low contention); and the
  table hasher is only touched when a `Name` is constructed or first
  resolved, never on the eq/hash hot path, so foldhash there was a
  speculative micro-optimization that conflated this branch with the
  collections-hasher work. The branch is interning-only; foldhash is
  perf-tested separately on `tninesling/collections-hasher`.
- **`Name` (`src/name.rs`)**: the 24-byte struct grows to 32 with a
  lazily-resolved `AtomicU32` symbol cache (0 = unresolved, for `const`
  names; `u32::MAX` = probed a frozen table and missed). `Eq` is a symbol
  compare with a string fallback for frozen-table misses; `Hash` emits the
  symbol likewise. `Borrow<str>` is removed; `NameKey` is the no-alloc
  probe key replacing it.
- **Kept from the productionized draft**: the freeze contract (the DoS
  answer), table-owned entries (no weak references or sweeps — without a
  freeze, growth is bounded by the distinct names in schemas and documents
  actually processed), and `NameKey`.

The −13 to −14% reproduction above was measured with the sharded,
foldhash-backed table; re-run the production corpus on the simplified
table before merge to confirm the number holds.

### Exit criteria

- Phase 1: zero test/snapshot changes, no compiler bench regression, ≥ one
  downstream bench shows a statistically significant improvement, DoS test
  passes, miri clean.
- Phase 2/adoption: by-string lookup property tests pass; each router
  adoption PR justified by its own before/after bench or profile.

## Risks

| Risk | Mitigation |
| --- | --- |
| Interner lock contention on hot parse paths | Post-freeze reads use an immutable snapshot with no lock; pre-freeze interning happens during schema builds, where contention is low |
| Unbounded interner growth from hostile input | Lookup-only mode on request paths — untrusted input structurally cannot insert; explicit DoS test |
| Schema churn leaving dead entries | No reclamation: entries are table-owned and live for the process. Growth is bounded by the distinct names in schemas and documents actually processed (frozen after schema build on servers), so churn cannot grow the table the way request traffic could |
| Breaking by-`&str` map lookups | Accepted and quantified (Follow-up 3): `NameKey` is the compile-time-guided migration; compiler ±350, federation ±70, router ±40 lines |
| `unsafe` lifecycle bugs (drop/remove race) | Miri, concurrency stress tests, keep interner logic in one small module |
| Semver | Breaking by design (`Borrow<str>` removed, `name!()` const → `static`, `Name` 24→32 bytes); lands in 2.0 |
