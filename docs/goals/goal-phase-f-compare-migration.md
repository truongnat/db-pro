# DB Pro — Phase F Goal: Schema Compare & Migration

- Doc ID: `GOAL-PHASE-F`
- Phase: F — Schema Compare / Migration
- Release target: v0.4 (F01–F04)
- Priority: P2
- Authority: `docs/goals/goal-full-product.md` (master goal)
- Depends on: `A01` for the apply path and builders, `F02` diff depth before any trustworthy
  generation, `components/diff.rs` for rendering.
- Status: PLANNING. No implementation has started.
- Companions: `docs/goals/goal-phase-a-object-crud.md`, `docs/goals/goal-phase-h-ai.md`,
  `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` §12.

---

## 1. Problem

DB Pro has a schema-diff function that almost nobody can use, and no migration capability at
all.

Observed facts (source-verified):

1. **A narrow diff exists in the backend and is unreachable.** `SchemaService::diff_schemas`
   (`crates/core/src/application/schema_diff.rs:11`) introspects two connections and compares
   them via `compare_introspect_results` (`:62`). It covers **only** tables present/absent,
   per-table column presence and type mismatches, and indexes present/absent. It does not
   compare primary keys, foreign keys, unique constraints, check constraints, nullability,
   defaults, collation, views, materialized views, triggers, routines, sequences, or types.
   The capability matrix marks it `BACKEND_ONLY` and notes the coverage is overstated by the
   `schema_diff: true` flag (which is `true` for **both** providers).
2. **The native UI never calls it.** `crates/ui` contains no compare surface; the Transfers
   activity is a placeholder (`crates/ui/src/navigation_view.rs:610-615`); the documented
   Transfer sidebar entries (`Schema Compare`, `Data Compare`, `Migrations`) do not exist.
3. **Data diff is count-only.** `DataDiffService::diff_table_data`
   (`crates/core/src/application/data_diff.rs:20`) runs `SELECT COUNT(*)` on both sides and
   returns `row_count_diff`. The `data_diff: true` capability flag overstates this.
4. **There is no DDL diff, no migration plan, no apply path.** No code converts a diff into
   `ALTER`/`CREATE`/`DROP` statements, no ordered operation list exists, no migration record
   exists, and the only DDL execution path is the generic
   `SchemaService::execute_ddl_batch` (`schema_service.rs:361`).
5. **The name `migration` is already taken by something else.**
   `crates/infrastructure/src/meta/migration.rs` (`LATEST_VERSION = 2`) migrates the app's own
   metadata store. It has nothing to do with database schema migrations. Any new code must not
   reuse the word ambiguously (`PRODUCT_CAPABILITY_MATRIX.md` §12 explicitly warns about this).
6. **Determinism already matters here.** The existing diff sorts results deterministically and
   has tests for qualified-name handling and ordering (`schema_diff.rs:158`). That property
   must be preserved and extended — an unstable diff makes migration plans unreviewable.

Why this matters: comparing a development schema to staging before a release, and applying the
resulting changes, is the workflow that makes a database IDE valuable to a team rather than to
an individual. It is also the hardest phase in this roadmap to make *safe*, because a wrong
plan can destroy data.

## 2. Scope

### 2.1 In scope

| Capability | Milestone |
|---|---|
| Compare sources: connection, database, schema, and **saved snapshot** (offline) | F01 |
| Diff rendering: filterable tree with Added / Removed / Changed, per-object property diff, raw diff export | F01 |
| Diff depth: PK, FK, unique, check, views, materialized views, triggers, routines, sequences, types, and column attributes (nullability, default, collation, identity/generated) | F02 |
| Rename detection as a **labeled suggestion** (never silently applied) | F02 |
| Deterministic ordering of every diff and plan | F01/F02 |
| `MigrationPlanner`: diff → ordered operations with per-dialect SQL, operation class, and optional inverse | F03 |
| Migration plan review UI: generated SQL, warnings (data loss, locks, unsupported), copy/export | F03 |
| Apply: transactional where supported, per-operation progress, stop-on-error, post-apply verification | F04 |
| Rollback plan generated from operation inverses when available; explicit "no rollback" warning otherwise | F04 |
| Cross-provider comparison (PG↔SQLite) as **structural only**, with explicit refusal for operations the target cannot express | F01/F03 |

### 2.2 Explicitly out of scope

- **Data compare beyond a later, limited form.** PK-matched row diff with sampling/limits is a
  documented Later item (master goal §6.6); this phase ships schema compare only. The
  `data_diff` flag must be corrected to describe the count-only reality.
- **Auto-apply.** There is no path from a plan to execution without explicit user review and
  confirmation.
- **Versioned migration history as a product** (a changelog of applied migrations across time)
  — the plan/apply record exists for audit, but this is not an ORM migration framework.
- **Three-way merge** (base + ours + theirs) and team merge conflict resolution.
- **Data migration/transformation expressions** (e.g. "populate new column from a function").
  Column additions may carry a default; arbitrary backfill SQL is out of scope for the planner
  and belongs in a migration script the user writes.
- **Cross-engine type translation matrices.** Unsupported translations are refused with a
  reason, not approximated.

## 3. Non-goals

- **No silent destructive migration.** Any operation that drops or narrows is marked
  `Destructive` and cannot be applied without the dedicated confirmation.
- **No plan generation from a partial diff.** If F02 has not covered an object kind, the
  planner must report "not compared" for that kind rather than implying there is no difference.
  *Absence of evidence is not evidence of absence* — this rule is the difference between a
  useful tool and a dangerous one.
- **No cross-provider DDL generation.** PG↔SQLite produces a structural comparison and an
  explicit list of non-migratable differences.
- **No re-use of the term "migration" for the app meta store.** New code uses
  `SchemaMigrationPlan`/`MigrationOperation`; the existing `meta/migration.rs` stays as-is.
- **No hidden regeneration between preview and apply.** The applied SQL is the reviewed SQL
  (fingerprint rule from Phase A).
- **No new diff engine per provider.** Comparison operates on `IntrospectResult` values, which
  are already provider-neutral; provider-specific SQL generation happens only in the planner.

## 4. Current implementation (code-verified baseline)

| Element | Reality | Evidence |
|---|---|---|
| `diff_schemas` | Exists, driver-agnostic, deterministic | `core/src/application/schema_diff.rs:11-156` |
| Diff coverage | Tables (present/absent), columns (present/absent + type mismatch), indexes (present/absent) | `schema_diff.rs:62-156` |
| Diff types | `SchemaDiff`, `TableColumnDiff`, `ColumnTypeMismatch` | `core/src/domain/cross_connection.rs:4-27` |
| Data diff | Count-only | `core/src/application/data_diff.rs:20-77` (`extract_count:80`, `row_count_difference:103`) |
| Capability flags | `schema_diff: true`, `data_diff: true` for both drivers — overstates both | `core/src/domain/capabilities.rs:173-174, 228-229` |
| Runtime exposure | `SchemaApi::diff_schemas` (`runtime/src/api.rs:507`), `DataDiffApi::diff_table_data` (`:873`) | both reachable only from code, not the native UI |
| Native UI | **No compare surface**; Transfers placeholder | `ui/src/navigation_view.rs:610-615` |
| DDL generation for objects | Builders exist for tables/views/indexes/triggers but are unused; no constraint builders | `ddl_builder.rs` (see Phase A §4) |
| DDL execution | Generic batch with policy check | `schema_service.rs:340-380` |
| Diff rendering component | `components/diff.rs::{DiffViewer, DiffLine, DiffLineType}` exists, **gallery-only** | `crates/ui/src/components/diff.rs` |
| Snapshot persistence | **Absent** — no diff snapshot table in the meta store | `infrastructure/meta/schema.rs` (9 tables, none for diffs) |
| Plan/apply | **Absent** | no planner, no operation model, no apply flow, no rollback |

## 5. UX

### 5.1 Compare workspace tab

```text
COMPARE — Schema                                    [environment: SOURCE → TARGET]

Source  [ local-pg ▾ ]  [ database: appdb ▾ ]  [ schema: public ▾ ]   ( ) Live  (•) Snapshot 2026-09-10 14:22
Target  [ staging-pg ▾ ] [ database: appdb ▾ ]  [ schema: public ▾ ]  (•) Live  ( ) Snapshot …

[ Compare ]     Last compared: 14:31:02 · 412 objects · 37 differences · 6 destructive

┌─ Differences ────────────────────────────┐ ┌─ Details ─────────────────────────────────────┐
│ ▾ Tables (12)                            │ │ public.orders                                 │
│    + public.invoices           added     │ │ Status: Changed (columns)                     │
│    − public.legacy_orders      removed   │ │                                               │
│    ~ public.orders             changed   │ │ Property    Source            Target          │
│        ~ columns (2 changed)             │ │ ---------   ----------------  ----------------│
│          ~ total_amount: numeric         │ │ total_amount numeric(12,2)    numeric(14,2)   │
│            (10,2) → (14,2)               │ │ notes        text NOT NULL    text NULL       │
│          ~ notes: nullability            │ │                                               │
│ ▾ Indexes (4)                            │ │ [ Generate migration for this object ]        │
│ ▾ Foreign keys (3)    (not compared — F02)│ │                                               │
│ …                                        │ └───────────────────────────────────────────────┘
└──────────────────────────────────────────┘
[ Generate migration plan ]   [ Export diff (JSON) ]   [ Save snapshot ]
```

Rules:

- Object kinds that the current diff depth does not cover are shown as **"not compared"** with
  a link to the milestone, not omitted.
- `+` added, `−` removed, `~` changed are the only three states — no fourth "maybe" state.
- Filters: state, object kind, name search, "destructive only".
- Selecting a node shows a property-level diff using the shared `DiffViewer`.
- Snapshots make offline comparison possible: compare two snapshots, or a snapshot against a
  live connection (for example, "compare production-as-of-last-week with staging-now").

### 5.2 Migration plan review

```text
MIGRATION PLAN                                    37 operations · 6 destructive · target: staging-pg

  #   Class           Object                    Statement (summary)                       Rollback
  1   Mutation        type order_status         CREATE TYPE order_status AS ENUM (...)    DROP TYPE
  2   Mutation        table public.invoices     CREATE TABLE public.invoices (...)        DROP TABLE
  3   Mutation        column public.orders      ALTER TABLE ... ADD COLUMN ...            DROP COLUMN
  4   Mutation        column public.orders      ALTER TABLE ... ALTER COLUMN TYPE ...     ALTER COLUMN TYPE (may fail)
  5   Destructive     column public.legacy…     ALTER TABLE ... DROP COLUMN ...           NO ROLLBACK
  6   Mutation        constraint …              ALTER TABLE ... ADD CONSTRAINT ...        DROP CONSTRAINT
  …
  37  Mutation        view public.v_orders      CREATE OR REPLACE VIEW ...                CREATE OR REPLACE VIEW (previous)

  Warnings
   ⚠  #5 drops a column containing data (1,204,551 rows estimated). No rollback available.
   ⚠  #4 narrows numeric(14,2) → numeric(12,2): values outside the range will fail.
   ⚠  #9 adds a NOT NULL column without a default to a non-empty table: the operation will fail
      unless a default is supplied.
   ⚠  #22 adds a foreign key; validation scans the table and takes a lock.

  [ Copy SQL ]  [ Export .sql ]  [ Export rollback .sql ]   [ Review complete → Apply… ]
```

Rules:

- The plan is **ordered by dependency**: types → tables → columns → constraints → indexes →
  foreign keys → views → materialized views → routines → triggers → sequences ownership.
  The planner owns this order and documents it.
- Each operation lists its class, its target, a one-line summary, the full statement on
  expansion, and its rollback disposition.
- Warnings are generated from the operation set, not from heuristics on the raw SQL:
  data-loss detection comes from the operation type (drop column, narrow type, drop object with
  rows).
- `Export rollback .sql` is offered only when every operation has an inverse; otherwise the
  file contains the inverses that exist plus explicit comments for the ones that do not, and
  the UI states that the rollback is partial.
- Apply is a separate, deliberate action (§5.3) — the plan screen is a review surface.

### 5.3 Apply

```text
APPLY MIGRATION — staging-pg (Staging)                     37 operations · 6 destructive

This will modify the target database. Review the plan before continuing.
[ ] I have reviewed all 37 operations and understand 6 of them cannot be rolled back.
Type the target connection name to confirm: [ staging-pg          ]

Transaction: single transaction (rollback on failure)     [ Change to per-operation ]

[ Cancel ]                                    [ Apply migration ]
```

During apply:

```text
  ✓ 1/37  CREATE TYPE order_status               12 ms
  ✓ 2/37  CREATE TABLE public.invoices           48 ms
  ▶ 3/37  ALTER TABLE public.orders ADD COLUMN … running
  ⋯ 4–37  pending
                                             [ Stop after current operation ]
```

After apply:

```text
  Applied 37/37 in 4.2 s.        [ Re-compare to verify ]   [ Open audit entry ]   [ Export report ]
```

Rules:

- **Destructive plans require the typed confirmation even outside Production.** Production adds
  the environment banner and requires the typed name for *any* plan (not only destructive).
- Transaction mode: `Single transaction` is the default when the provider supports
  transactional DDL for all operations in the plan (both providers do, but some operations
  cannot run inside a transaction, e.g. PostgreSQL `VACUUM`-class operations are not in a
  migration plan; `CREATE INDEX CONCURRENTLY` is explicitly excluded from plans because it
  cannot run in a transaction — if the planner would need it, the operation is flagged and a
  non-transactional apply is offered with a warning).
- Stop-on-error: either stop (leaving the transaction rolled back) or stop-after-current
  (per-operation mode, leaving a partial state that is reported exactly).
- Post-apply verification: re-introspect and re-diff against the source; the result is shown as
  "0 differences" or the residual difference list with an explanation.
- Apply always writes an audit record and a migration report (artifact) the user can export.

## 6. Architecture

### 6.1 Flow

```text
CompareService.compare(source, target)         → SchemaDifference (typed, deterministic)
MigrationPlanner.plan(difference, target_dialect) → SchemaMigrationPlan { operations, warnings, rollback }
MigrationApplier.apply(plan, options)          → MigrationReport { per-operation results }
MigrationApplier.verify(plan, source)          → SchemaDifference (residual)
```

### 6.2 Component boundaries

| Component | Responsibility | Must not |
|---|---|---|
| `SchemaComparator` | Compare two `IntrospectResult`s (or snapshots) in a provider-neutral way | Generate SQL |
| `SnapshotStore` | Persist/load `IntrospectResult` snapshots (meta store) | Contain credentials |
| `MigrationPlanner` | Turn a difference into ordered `MigrationOperation`s with per-dialect SQL and inverses | Execute anything |
| `MigrationApplier` | Execute operations through `SchemaService`/`ObjectMutationService` with the policy gate, progress, and rollback handling | Build SQL itself (it consumes operation statements) |
| UI | Render difference and plan; collect confirmation | Build SQL, reorder operations, filter operations out of a plan silently |

The planner consumes operation statements **from Phase A builders**. If an operation has no
builder, the planner reports it as `Unsupported` rather than inlining SQL — this keeps one
statement-construction path for the whole product.

### 6.3 Determinism

- Comparison sorts by `(object_kind, schema, name, sub_key)`.
- Operations are emitted in the documented dependency order; within a stage, by
  `(schema, name)`.
- Identical inputs must produce byte-identical plans. This is a test, not an aspiration.
- Snapshot metadata (server version, capture time, connection identity **without credentials**)
  is recorded so a later comparison can explain version-dependent differences.

### 6.4 Cross-provider handling

| Pair | Comparison | Plan generation |
|---|---|---|
| PG ↔ PG | Full (per F02 depth) | Full |
| SQLite ↔ SQLite | Full (per F02 depth, limited to SQLite object kinds) | Full |
| PG ↔ SQLite | Structural: names, columns, types (as declared strings) | **Refused** for operations the target cannot express; each non-migratable difference is listed with a reason |

Cross-provider comparison must never claim "no differences" for an object kind the target
provider cannot have (for example, PG has no equivalent of nothing; SQLite has no types) — those
rows are reported as "not comparable" with an explanation.

## 7. Domain model

New module `crates/core/src/domain/migration.rs` (named to avoid the meta-store confusion;
`SchemaMigration*` prefixes in prose):

```rust
pub enum DiffState { Added, Removed, Changed, Unchanged, NotCompared }

pub struct ObjectDifference {
    pub kind: ObjectKind,
    pub schema: Option<String>,
    pub name: String,
    pub state: DiffState,
    pub property_diffs: Vec<PropertyDiff>,      // field name, source value, target value
    pub children: Vec<ObjectDifference>,        // e.g. columns inside a table
    pub not_compared_reason: Option<String>,    // set when state == NotCompared
}

pub struct PropertyDiff { pub property: String, pub source: Option<String>, pub target: Option<String> }

pub struct SchemaDifference {
    pub source: SnapshotRef,
    pub target: SnapshotRef,
    pub objects: Vec<ObjectDifference>,
    pub summary: DiffSummary,                   // counts per kind and state
    pub scope_note: String,                     // e.g. "cross-provider: structural comparison only"
}

pub enum OperationClass { Mutation, Destructive, Administrative }

pub struct MigrationOperation {
    pub ordinal: u32,
    pub stage: MigrationStage,
    pub class: OperationClass,
    pub target: ObjectRef,
    pub summary: String,
    pub statements: Vec<String>,                // exact SQL for the target dialect
    pub inverse: Option<Vec<String>>,           // None = no rollback available
    pub warnings: Vec<String>,
    pub long_running: bool,
}

pub enum MigrationStage {
    Types, Tables, Columns, Constraints, Indexes, ForeignKeys,
    Views, MaterializedViews, Sequences, Routines, Triggers, Ownership,
}

pub struct SchemaMigrationPlan {
    pub target_connection: ConnectionId,
    pub target_dialect: DriverType,
    pub operations: Vec<MigrationOperation>,
    pub warnings: Vec<String>,
    pub destructive_count: u32,
    pub rollback: RollbackPlan,
    pub fingerprint: String,                    // binds review to apply
}

pub enum RollbackPlan { Full(Vec<String>), Partial { statements: Vec<String>, gaps: Vec<String> }, None { reason: String } }

pub struct MigrationReport {
    pub plan_fingerprint: String,
    pub results: Vec<OperationResult>,
    pub status: MigrationStatus,                // AppliedAll | StoppedOnError | Cancelled | PartiallyApplied
    pub committed_count: u32,
    pub duration_ms: u64,
    pub residual: Option<SchemaDifference>,     // filled by verification
}
```

## 8. APIs, services, ports

### 8.1 Services

```rust
impl CompareService {
    pub async fn compare(&self, source: CompareSource, target: CompareSource) -> Result<SchemaDifference, DbError>;
    pub async fn save_snapshot(&self, connection_id: ConnectionId, scope: SnapshotScope) -> Result<SnapshotId, DbError>;
    pub async fn list_snapshots(&self, connection_id: ConnectionId) -> Result<Vec<SnapshotMeta>, DbError>;
    pub async fn delete_snapshot(&self, id: SnapshotId) -> Result<(), DbError>;
}

impl MigrationPlanner {
    pub fn plan(&self, diff: &SchemaDifference, target: DriverType) -> Result<SchemaMigrationPlan, DbError>;
    // pure: no I/O; all SQL comes from Phase A builders via the target dialect
}

impl MigrationApplier {
    pub async fn plan_apply(&self, plan: &SchemaMigrationPlan) -> Result<ApplyPreview, DbError>;   // modes + warnings
    pub async fn apply(&self, plan: &SchemaMigrationPlan, options: ApplyOptions, progress: ProgressSink)
        -> Result<MigrationReport, DbError>;
    pub async fn verify(&self, plan: &SchemaMigrationPlan, source: CompareSource) -> Result<SchemaDifference, DbError>;
}
```

`MigrationPlanner::plan` being pure is what makes ordered, dialect-correct SQL testable without
a database — the same discipline as Phase A's `plan_change`.

### 8.2 Ports

| Port | Methods | Notes |
|---|---|---|
| `SchemaSnapshotRepository` (new) | `save`, `list`, `get`, `delete`, `prune` | Meta store; stores redacted `IntrospectResult` JSON plus metadata (server version, timestamp, connection label, no credentials) |
| Reuse `IntrospectCache` shapes | — | Snapshots are a persisted, named form of introspection; reuse the serialization approach rather than inventing a second format |
| Reuse `DbConnector::introspect` | — | Snapshot capture is an introspection call plus persistence |

No new connector methods are required for comparison. Apply reuses
`SchemaService::execute_ddl_batch` / `ObjectMutationService` from Phase A.

### 8.3 Runtime/UI wiring

```text
UiCommand::CompareSchemas        { request_id, source, target }
UiCommand::SaveSchemaSnapshot    { request_id, connection_id, scope }
UiCommand::ListSchemaSnapshots   { request_id, connection_id }
UiCommand::PlanMigration         { request_id, diff, target_connection_id }
UiCommand::ApplyMigration        { request_id, plan, options, confirmation }
UiCommand::VerifyMigration       { request_id, plan, source }
UiEvent::SchemasCompared         { request_id, diff }
UiEvent::SchemaSnapshotSaved     { request_id, meta }
UiEvent::SchemaSnapshotsLoaded   { request_id, snapshots }
UiEvent::MigrationPlanned        { request_id, plan }
UiEvent::MigrationProgress       { request_id, ordinal, total, operation_summary, status }
UiEvent::MigrationApplied        { request_id, report }
UiEvent::MigrationVerified       { request_id, residual }
```

`MigrationProgress` reuses the Phase C progress plumbing (rate-limited; same shape) rather than
adding a fourth progress mechanism.

### 8.4 Agent tools (documented here; implemented in Phase H)

| Tool | Risk | Backend | Constraint |
|---|---|---|---|
| `compare_schemas` | ReadOnly | `CompareService::compare` | Returns a bounded summary + the diff handle |
| `generate_migration` | ReadOnly (produces text) | `MigrationPlanner::plan` | Never applies; output is a plan the user reviews in the F03/F04 UI |

## 9. Provider behavior

### 9.1 PostgreSQL ↔ PostgreSQL

- Full depth after F02. Column attributes compared: type, nullability, default expression
  (normalized text), identity, generated expression, collation.
- Constraint comparison by (kind, name, definition): PG auto-generates constraint names, so a
  name mismatch with an identical definition is reported as a difference in name only and the
  planner **does not** rename automatically — it offers a rename operation as optional with a
  warning about dependent references.
- Index comparison by (name, method, columns, unique, predicate, include columns).
- View/matview comparison by normalized definition text; a whitespace/format-only difference is
  reported as "changed (definition)" with the raw diff shown, because normalizing SQL text
  reliably is not attempted. The planner offers `CREATE OR REPLACE VIEW` only, not a
  reformat.
- Routine comparison by signature + source text; a change produces `CREATE OR REPLACE`.
- Type comparison: enum labels compared in order (order is semantically meaningful); a label
  reorder is Destructive-ish and is reported explicitly.
- Sequence comparison: data type, increment, min, max, cache, cycle, owned-by. **Current value
  is not compared** by default (it is data, not schema) — offer it as an explicit opt-in with a
  clear label.

### 9.2 SQLite ↔ SQLite

- Depth limited to SQLite's object kinds: tables, columns (`PRAGMA table_info` data), indexes,
  views, triggers, foreign keys, check constraints (as parsed today — note the fidelity
  limitations in `PRODUCT_CAPABILITY_MATRIX.md` §2), no schemas, no sequences/types/routines.
- Because SQLite cannot alter a column type, a type change produces a **`Destructive`,
  non-transactional-in-place** rebuild operation. The planner may emit the documented
  create-copy-swap sequence **only** when the user explicitly opts into "rebuild table"
  operations; otherwise the operation is reported as "cannot migrate automatically" with a
  generated rebuild script for manual review. Either way it is Destructive and requires the
  typed confirmation.
- Data loss risk on rebuild is stated per table with an estimated row count.

### 9.3 PostgreSQL ↔ SQLite (structural)

- Comparison is limited to schema/table/column names and declared type strings.
- The plan is **refused**; the UI lists each difference with one of: "not expressible in SQLite"
  (types, schemas, sequences, routines, matviews, roles), "expressible but not automatic"
  (type changes requiring rebuild), or "expressible" (table/index/view creation) with the
  statement shown for manual use.
- The scope note in `SchemaDifference.scope_note` must state the comparison is structural.

### 9.4 Capability flags

`SchemaCapabilities`/`FeatureCapabilities` after this phase:

| Flag | PostgreSQL | SQLite |
|---|---|---|
| `schema_diff` | true (depth per F02) | true (depth per F02, SQLite kinds) |
| `schema_diff_generate_migration` | true (F03) | true (F03, with rebuild caveats) |
| `schema_diff_apply` | true (F04) | true (F04) |
| `data_diff` | **false** until the later row-level work lands; the flag must be corrected or renamed to `data_diff_row_counts` to stop overstating | same |

The `data_diff` correction is part of F01 (flags must match reality — master goal §4.3).

## 10. UI structure

### 10.1 Components

| Need | Component | Status |
|---|---|---|
| Diff tree | `ObjectTree`/`ObjectList` + state icons | reuse |
| Property diff | `DiffViewer` (`components/diff.rs`) | **wire from gallery** |
| SQL plan view | `DDLViewer` + `CodeBlock` | reuse |
| Operation list with classes | `DataGrid`/`ObjectList` + `Badge` | reuse |
| Warnings | `Alert` list with severity | reuse |
| Confirmation (typed name) | `ConfirmationDialog` with a typed-name variant | extend |
| Progress | `ProgressView` | reuse |
| Snapshot picker/manager | `Select` + `ObjectList` | reuse |
| Report export | file picker + `CodeBlock` preview | reuse |

### 10.2 Screens

1. **Compare tab** (§5.1) — sources, live/snapshot selection, difference tree, property diff,
   filters, diff export.
2. **Snapshot manager** — list, save, delete, rename, and compare two snapshots.
3. **Migration plan tab** (§5.2) — operation list, warnings, SQL view, copy/export, rollback
   export.
4. **Apply dialog** (§5.3) — review acknowledgement, typed confirmation, transaction mode,
   progress with stop, report.
5. **Migration report** — per-operation results, duration, residual verification, export.
6. **Transfer activity entries** — Schema Compare and Migration become real entries (the
   placeholder is removed in Phase C for the activity; these two entries are enabled in F01/F03).

### 10.3 Interaction rules

- The compare result is a snapshot in time; a "Re-compare" action is explicit and the tab shows
  the comparison timestamp. Nothing auto-refreshes a plan.
- Editing the plan is limited to: excluding an operation (with a warning that the plan is now
  partial and the fingerprint changes), and reordering is **not** offered (dependency order is
  the planner's contract).
- Excluding a destructive operation is allowed and produces a clearly-labeled partial plan; the
  excluded operations are listed in the report.
- Applying a plan from a snapshot source works, but the UI warns that the source snapshot may be
  stale relative to live state.

## 11. Safety model

### 11.1 Classification

| Operation | Class |
|---|---|
| Compare, snapshot save/list/load, diff export, plan generation, plan export | `ReadOnly` |
| Non-destructive migration operations (create, add column with default, create index, add constraint, create-or-replace view) | `Mutation` |
| Destructive operations (drop column/table/type, narrow type, reorder enum labels, SQLite table rebuild, drop with data) | `Destructive` |
| Applying a plan containing any destructive operation | The whole apply is gated as `Destructive` |

### 11.2 Gates

1. **Coverage gate**: the plan states which object kinds were compared; a plan built from a
   diff with `NotCompared` kinds shows that in the review screen and the report.
2. **Destructive gate**: any destructive operation forces the dedicated confirmation (typed
   connection name, and in Production, typed name regardless of destructiveness).
3. **Transaction gate**: single-transaction apply is the default; per-operation apply is an
   explicit choice with a stated partial-state risk; operations that cannot run in a transaction
   are isolated and labeled.
4. **Fingerprint gate**: the reviewed plan's fingerprint must match at apply time.
5. **Policy gate**: applying uses the standard policy — read-only connections cannot apply, and
   `allow_ddl`/`allow_destructive` (I01) block accordingly.
6. **No-auto-apply gate**: no code path exists from `MigrationPlanner::plan` to
   `MigrationApplier::apply` without a user-supplied fingerprint from a reviewed plan. This is
   asserted by test.

### 11.3 Rollback honesty

- A rollback plan is generated only from real inverses. Where an inverse does not exist
  (dropped column data, narrowed type with data loss, dropped table), the rollback plan says
  so explicitly and the apply screen lists those operations before confirmation.
- The rollback script is **generated, reviewed, and executed by the user** — this phase does not
  offer a one-click rollback button, because "roll back a partially applied destructive
  migration" is exactly the situation where an automated action is most dangerous. (A future
  milestone may add a gated rollback execution; it is not in F04.)

### 11.4 Additional rules

- Snapshot files/rows contain schema metadata only: no row data, no credentials, no connection
  strings with passwords.
- The migration report contains statements and outcomes, never row values.
- Cross-provider plans cannot exist (refused), so there is no risk of generating a plan whose
  dialect does not match the target.

## 12. Testing strategy

### 12.1 Unit

- Comparator per object kind: added/removed/changed/unchanged/not-compared; property-level
  diffs; nested children (columns inside tables); deterministic ordering under shuffled input
  (the existing determinism test is extended).
- Snapshot round-trip: serialize → persist → load → compare produces an identical diff.
- Planner: dependency ordering (types before tables before constraints before indexes before
  FKs before views before routines); stable ordinals; stage assignment.
- Statement generation: dialect correctness per provider; identifier quoting; refusal for
  cross-provider; `Unsupported` for operations without a Phase A builder.
- Inverses: every operation that can have one declares one; every operation that cannot declares
  why; assert that no destructive operation silently has `None` without a warning.
- Classification: each operation class assignment is asserted.
- Fingerprint stability: the same plan produces the same fingerprint; any change to the
  operation set changes it.

### 12.2 Service / integration

- Fixture pairs (new `fixtures/schema-diff/`): identical, additive-only, destructive-only,
  rename-lookalike (drop+create vs rename suggestion), type change, index-only,
  constraint-only, view body change, routine body change, enum label change, sequence attribute
  change.
- SQLite (always-run): compare two fixture databases; plan; apply non-destructive operations to
  a scratch database; verify residual diff is empty; apply a destructive plan only after the
  typed confirmation and verify the state; assert the no-auto-apply gate.
- PostgreSQL (live, ignored suite): the same matrix against two databases/schemas in one
  cluster; apply and verify; a partial-failure test (operation 3 of 5 fails) asserting the
  deterministic state and the report.
- Cross-provider: PG vs SQLite produces a structural diff with `NotCompared`/non-migratable
  entries and `plan` returns the documented refusal.
- Rollback: apply a plan whose inverses exist, then execute the generated rollback script and
  verify the target returns to the original schema (compare against the pre-apply snapshot).

### 12.3 UI

- Compare tab: tree states, filters, "not compared" rendering, property diff.
- Plan tab: ordering display, warning rendering, exclude-an-operation flow (fingerprint change +
  partial-plan warning), copy/export.
- Apply dialog: typed confirmation enforcement (destructive and Production), transaction mode
  explanation, progress rendering, stop behavior, report.
- Verification: re-compare shows zero differences; residual differences render with explanations.
- Disabled states: read-only connection, cross-provider plan refusal, snapshot from a dropped
  connection.

## 13. Runtime verification

**PostgreSQL**

1. Capture a comparison between two live schemas (dev → staging) with a mix of added/removed/
   changed objects across tables, columns, indexes, constraints, views, and routines.
2. Save a snapshot of the source; drop the source connection's changes; compare snapshot vs live
   target to prove offline comparison works; capture the snapshot manager.
3. Generate a migration plan; capture the ordered operation list and all warnings.
4. Attempt to apply without the typed confirmation; capture the block.
5. Apply a non-destructive subset; capture progress and the report.
6. Re-compare; capture the residual difference list showing the expected remaining differences.
7. Export the rollback script; execute it manually; re-compare to show the schema returned to the
   pre-apply state.
8. Destructive case: attempt to apply a plan containing a dropped column; capture the dedicated
   confirmation and the "no rollback" warning; after apply, capture the report listing the
   non-rollbackable operation.

**SQLite**

9. Compare two SQLite fixture databases; capture the diff (SQLite kinds only).
10. Plan and apply a non-destructive migration; verify the residual diff is empty.
11. Attempt a plan containing a column type change; capture the refusal or the explicit
    rebuild opt-in with its destructive warning.

**Cross-provider**

12. Compare a PostgreSQL schema with a SQLite schema; capture the structural comparison, the
    non-migratable list, and the plan refusal with reasons.

## 14. Milestone order

| Order | ID | Title | Release | Pri | Prerequisites | Rationale |
|---:|---|---|---|---|---|---|
| 1 | F01 | Schema Compare UI (+ snapshot store, flag correction) | v0.4 | P2 | existing diff backend; `components/diff.rs` | Makes the existing backend visible, introduces snapshots and the diff UI, and corrects the overstated `data_diff`/`schema_diff` flags — all before deepening the diff |
| 2 | F02 | Diff Depth Expansion | v0.4 | P2 | F01 | A migration planner built on a shallow diff would silently ignore differences — the most dangerous possible failure mode for this phase |
| 3 | F03 | DDL Diff & Migration Generation | v0.4 | P2 | F02, A01 builders | Requires complete diff coverage and Phase A's builders (statements must not be inlined) |
| 4 | F04 | Migration Apply + Rollback Plan | v0.4 | P2 | F03, A01 apply path | Apply carries the heaviest safety requirements; must follow generation and the A01 policy/audit path |

Ordering rules: F01 and F02 may merge if the reviewer accepts the scope, but F03 must not begin
before every object kind in the target matrix is either compared or explicitly `NotCompared`
(F02's exit criterion). F04 must not begin before Phase A's `ObjectMutationService` and policy
producer exist.

## 15. Definition of done

1. **Plan folder** per milestone; `docs/plans/STATUS.md` matches.
2. **P0 = 0, P1 = 0.**
3. **The diff is honest**: every object kind in the target matrix is either compared at the
   declared depth or reported `NotCompared` with a reason; the summary states the covered
   kinds; a test asserts that an uncompared kind cannot silently appear as "no difference".
4. **Determinism proven**: byte-identical plans from identical inputs, verified by a
   shuffled-input test.
5. **Snapshots work**: save, list, load, compare (snapshot↔snapshot and snapshot↔live), with no
   credentials or row data stored.
6. **Planner correctness**: dependency ordering, dialect-correct SQL from Phase A builders,
   `Unsupported` for missing builders, inverses declared for every operation that has one.
7. **Apply is never automatic**: no code path from plan to apply without a reviewed fingerprint;
   asserted by test and by review evidence (grep for `apply(` call sites).
8. **Destructive gating is real**: destructive plans cannot be applied without the typed
   confirmation, in any environment; Production requires the typed name for every plan.
9. **Transaction semantics are explicit** and tested: single-transaction default, per-operation
   opt-in with partial-state reporting, non-transactional operations isolated and labeled.
10. **Rollback honesty**: every non-rollbackable operation is listed before apply and in the
    report; the exported rollback script length matches the available inverses.
11. **Post-apply verification** exists and produces a residual diff; the runtime evidence shows
    zero differences after a successful apply.
12. **Cross-provider refusal** is implemented and tested: no plan is generated, and each
    non-migratable difference is listed with a reason.
13. **Flags corrected**: `data_diff` no longer overstates its coverage (renamed or set to
    reflect row-count-only), and `schema_diff` reflects the F02 depth per provider.
14. **Quality gates executed and recorded** (fmt, clippy with stated tauri scope,
    `cargo test --workspace`, SQLite suite, PostgreSQL live suite including a full
    compare → plan → apply → verify → rollback cycle).
15. **Docs updated**: capability matrix §12 rows, master goal §9 (target matrix), and the
    `PRODUCT_ROADMAP.md` note on migration depth.
16. **Non-goals respected**: no data compare beyond counts, no auto-apply, no inlined SQL in the
    planner, no cross-provider DDL, no one-click rollback execution, no reuse of the
    meta-store "migration" name.

Phase F is complete when a user can compare a development schema against staging (live or from
a snapshot), see every difference at the object and property level with the coverage of the
comparison stated explicitly, generate a dependency-ordered migration plan with warnings and
rollback dispositions, review and export it, apply it behind a typed confirmation inside a
transaction with per-operation progress, verify the result with a re-compare, and export a
rollback script that actually restores the previous schema where inverses exist — with
PostgreSQL and SQLite evidence recorded independently and cross-provider comparison refusing to
pretend.
