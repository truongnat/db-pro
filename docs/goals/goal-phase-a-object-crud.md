# DB Pro — Phase A Goal: Database Object CRUD (Typed Object Workbench)

- Doc ID: `GOAL-PHASE-A`
- Phase: A — Database Object CRUD
- Release targets: v0.2 (A01–A04), v0.3 (A05–A08)
- Priority: P1 (A01–A04), P2 (A05–A08)
- Authority: `docs/goals/goal-full-product.md` (master goal). This document must not
  contradict it; where it needs more detail it adds it.
- Depends on: nothing new for A01 (uses existing introspection + `execute_batch`); §9/§10 of
  the master goal for capability enforcement and the policy producer.
- Status: PLANNING. No implementation has started.
- Companions: `docs/goals/goal-phase-b-routines.md`, `docs/notes/PRODUCT_CAPABILITY_MATRIX.md`
  §2/§3, `docs/plans/FEATURE_LIFECYCLE.md`.

---

## 1. Problem

DB Pro can **read** a schema extremely well and **change** it almost not at all.

- Introspection covers tables, columns, PK, FK, indexes, checks, triggers, and functions on
  PostgreSQL, and a narrower set on SQLite (`crates/infrastructure/src/postgres/introspect.rs`,
  `crates/infrastructure/src/sqlite/introspect.rs`).
- Reconstructed, dialect-aware DDL already exists and is trustworthy enough to display
  (`crates/core/src/application/schema_service.rs:264-620`).
- But the only mutation path is **free-form SQL text**: `SchemaService::execute_ddl`
  (`schema_service.rs:340`) and `execute_ddl_batch` (`:361`) accept a string and execute it
  after a readonly/multi-statement check. No object is typed, nothing is previewed, and
  **capability flags are never consulted** — a SQLite connection can be asked to emit
  `ALTER COLUMN TYPE` and nothing in the backend stops it.
- Typed builders exist but are **dead code**: `crates/core/src/application/ddl_builder.rs`
  (create/drop table, add/drop column, rename table, create/drop view, create/drop index,
  create/drop trigger) has exactly one reference in the workspace —
  `crates/core/src/application/mod.rs:4`. No UI, runtime, or Tauri code calls them.
- The native UI confirms this: grepping `crates/ui` for `build_create_view`,
  `build_create_index`, `build_create_trigger`, or `create_trigger` call sites returns zero
  real usages (only demo strings in `component_gallery_view.rs` and one test assertion). The
  only "Create Table" affordance opens a query tab with a commented template
  (`crates/ui/src/explorer_view.rs:115-127`).

Consequences for users: adding an index, adding a foreign key, creating a view, editing a
trigger, or managing an enum all require hand-writing DDL in the query editor and knowing the
provider dialect. For SQLite they must additionally know what SQLite cannot do. There is no
preview, no confirmation, and no post-change verification that the schema actually changed.

**This phase exists to make the schema as editable as the data grid already is.**

## 2. Scope

### 2.1 Object kinds in scope

| Object | PostgreSQL | SQLite | Milestone |
|---|---|---|---|
| Table (create/alter/drop) | yes | yes | A01 (framework + table path) |
| Column (add/drop/rename; alter type on PG) | yes | partial (no type change) | A01 |
| View | yes | yes | A02 |
| Materialized view | yes | n/a | A02 |
| Index | yes | yes (btree only) | A03 |
| Primary key / Foreign key / Unique / Check | yes | partial (FK edits may need table rebuild) | A04 |
| Trigger (+ enable/disable on PG) | yes | create/drop only | A05 |
| Sequence | yes | n/a | A06 |
| Enum / Domain / Composite type | yes | n/a | A07 |
| Schema (create/rename/drop) | yes | n/a | A08 |
| Database (list; guarded create/drop) | guarded | n/a | A08 |

### 2.2 Capability matrix per object (the required contract)

Every in-scope object must support, where the provider allows:

- **inspect** — metadata already introspected (extend where noted)
- **create** — typed request → generated DDL → preview → apply
- **alter** — typed diff against current metadata → generated DDL → preview → apply
- **drop** — typed request → dependency warning → confirmation → apply
- **view DDL** — reconstructed DDL (exists for tables/views; add for the rest)
- **generate DDL** — produce SQL without executing (opens the query editor)
- **refresh metadata** — targeted invalidation + re-introspection of the affected object
- **dependencies** — what references this object / what it references
- **references** — reverse lookup where the provider exposes it

### 2.3 Cross-cutting deliverables

- `ObjectAction`, `ObjectMutationRequest`, `ObjectMutationPreview`, `ObjectMutationResult`
  canonical types (master goal §4.4).
- Backend capability enforcement in the mutation path (master goal §4.3).
- One shared UI pattern for every object: `ObjectHeader` / `ObjectTabs` / `Properties` /
  `DDL` / `Dependencies` / `Edit` / `Preview SQL` / `Apply` / `Cancel`.
- Audit records for every applied mutation (master goal §10.5).

## 3. Non-goals

- **No ER design/edit mode.** Diagram stays a visualization (`PRODUCT_CAPABILITY_MATRIX.md` §6
  marks design mode `DEFERRED`).
- **No table partitioning management**, no tablespace management, no extension management.
- **No data migration between objects** (that is Phase F) and no data movement (Phase C).
- **No partial-object editors** that write SQL fragments without validation — every request is
  typed and validated.
- **No new object kinds beyond the table above** in this phase; when a new kind is needed
  later it uses the A01 framework rather than a new mechanism.
- **No UI-side SQL construction.** Generating SQL in a view is a review-blocking defect.
- **No "raw DDL" fallback hidden behind a form.** If a field cannot be typed, the field is
  omitted and the user is directed to the query editor via "Generate SQL".

## 4. Current implementation (code-verified baseline)

Verified by source inspection on `main@b2cc33e` (see master goal §2.1 for method). Nothing in
this section is runtime-verified.

| Capability | Reality | Evidence |
|---|---|---|
| Table introspect / DDL reconstruct | Works | `schema_service.rs:264-620` (`build_create_table_ddl`, `format_column_definition`, index replay, PG FK `ALTER TABLE … ADD CONSTRAINT`, trigger reconstruction) |
| View create/drop builders | Exist, unused | `ddl_builder.rs:114-128` |
| Index create/drop builders | Exist, unused | `ddl_builder.rs:130-158` |
| Trigger create/drop builders | Exist, unused | `ddl_builder.rs:159-183` |
| Column add/drop builders | Exist, unused | `ddl_builder.rs:64-96` |
| Constraint builders (PK/FK/Unique/Check) | **Absent** | no such functions in `ddl_builder.rs` |
| Alter column type / rename column builder | **Absent** | `SchemaCapabilities.alter_column_type`/`rename_column` are true for PG (`capabilities.rs:40-45`) but nothing builds the statement |
| ALTER VIEW / `CREATE OR REPLACE VIEW` | **Absent** | only `build_create_view`/`build_drop_view` |
| Materialized view (any) | **Absent** | no `pg_matviews` query, no domain struct |
| Sequence (any) | **Absent** | no `pg_sequences`/`information_schema.sequences`; `sequences: true` flag with no code (`capabilities.rs:150`) |
| Enum / domain / type (any) | **Absent** | no `pg_type`/`pg_enum`; `enum_types: true` flag with no code (`capabilities.rs:151`) |
| Schema create/drop | **Absent** | only PG rename, as an inherent method: `postgres/connector.rs:648-666` + `postgres/cross_connection.rs:141-180`, reachable via `runtime/src/api.rs:954` with an inline readonly check at `:963-977` — **bypasses the application layer** |
| Database create/drop | **Absent** | introspection only knows the active database string |
| Unique constraints (multi-column) | Partial | derived from single-column unique indexes in `postgres/introspect.rs:34-51`; no `pg_constraint contype='u'` query |
| View columns | **Absent** | `View { name, schema, definition }` (`domain/schema.rs:98-103`) — no column list |
| Capability enforcement in DDL | **Absent** | `execute_ddl` checks readonly + multi-statement only (`schema_service.rs:340-380`); master goal §2.5 C2/C5 |
| Mutation preview + confirmation (pattern reference) | Exists for table DDL | `ui/src/table_editor_view.rs:1363-1431` (`draw_ddl_script_card`, `draw_ddl_confirmation_card`), reused from `components/agent_primitives.rs::{ExecutionApproval, RiskLevel}` |
| Object tabs pattern | Exists for tables | `ui/src/table_view.rs:243-251`, `app.rs:222-231` (Data/Structure/Indexes/Foreign Keys/Constraints/Dependencies/DDL) |
| Explorer object context menus | Partial | tables have menus (`explorer_details.rs:29-165`); views/functions have copy/select menus (`explorer_folders.rs:116-231`); **triggers have none** (`:252-291`) |

## 5. UX

### 5.1 Principles

1. **Preview before apply, always.** The generated SQL is visible and copyable before anything
   runs. "Generate SQL" is a first-class action that opens the query editor instead of
   executing.
2. **The object is the page.** Creating and editing happen in the object's workspace tab
   (ObjectTabs), not in a modal chain. Modals are reserved for confirmations and small
   property dialogs (e.g. rename).
3. **Never hide the target.** Header shows connection · environment · database · schema ·
   object, in that order, with the environment badge.
4. **Dialect-aware fields.** A field the provider cannot express is disabled with a reason
   (inline `Alert`), not silently ignored.
5. **Reversibility is stated.** The confirmation says whether the operation is transactional
   or has no rollback (master goal §10.4).
6. **Keyboard-first.** `⌘⏎` applies from a form, `Esc` cancels, `⌘⇧V` toggles the preview.

### 5.2 Object lifecycle flow

```text
Explorer context menu / ObjectHeader action
  → choose object kind + action (Create / Edit / Drop / Duplicate / Generate SQL)
  → typed form (FormEditor) prefilled from introspection (Edit) or templates (Create)
  → validation (client-side field rules + server-side capability/policy check)
  → Preview SQL (DDLViewer, exact statement + effect + rollback note)
  → Confirm (ConfirmationDialog; stronger for Destructive/Production)
  → Apply → progress (indeterminate for instant ops) → result
  → targeted metadata refresh → object tab + Explorer updated in place
  → audit entry in ActivityLog
```

### 5.3 Per-object UX requirements

| Object | Create | Edit | Drop |
|---|---|---|---|
| Table | Full form: columns (name/type/nullable/default/identity/generated/collation), PK, unique, checks, FKs (compose the same sub-forms), schema picker | Column-level operations first (add/rename/alter type/set default/set null), plus constraint tabs; each change is a discrete previewable operation | Typed name confirmation, dependency warning, `CASCADE` never default |
| View | Body editor (`CodeEditor`) + schema/name + options | Replace body (`CREATE OR REPLACE` where supported); rename | Confirmation with dependents list |
| Matview (PG) | Definition + storage options (tablespace, `WITH DATA`/`WITH NO DATA`) | Refresh (`REFRESH MATERIALIZED VIEW`, with `CONCURRENTLY` offered only if a unique index exists) | Confirmation with dependents |
| Index | Wizard: table → columns + order → method → unique → include columns → partial predicate (PG) | Drop + recreate with an explicit warning (no `ALTER INDEX` rewrite path) | Confirmation naming the index and table |
| Constraint | Type picker → column(s) → referenced table/columns (FK) → actions (on delete/update, match, deferrable) | Drop + add when the provider cannot alter in place | Confirmation; FK drop warns about orphan data risk |
| Trigger | Table → timing/event → function (PG picker) or body (SQLite) → when (PG `WHEN`) | Recreate; PG exposes enable/disable/replica/always | Confirmation; "disabled" state is visible in the tree |
| Sequence (PG) | name/schema/start/increment/min/max/cache/cycle/owner column | Alter any attribute; `setval` is a separate explicit data operation | Confirmation with dependency list |
| Type/Enum/Domain (PG) | Enum (labels), domain (base type + constraints + default + not null), composite (fields) | Add/rename enum value; domain constraints; composite fields | Confirmation listing dependent columns |
| Schema (PG) | name + owner | Rename only | Confirmation; refuse non-empty unless `CASCADE` is explicitly chosen with a warning |
| Database (PG) | name/owner/encoding (guarded; requires an administrative connection and is hidden otherwise) | n/a | Confirmation; requires no active sessions on the target |

### 5.4 Empty, error, and disabled states

- **Empty**: a folder with no objects shows `EmptyState` with the primary create action.
- **Error**: a failed apply shows the structured error (`DbError` code + message), the exact
  statement, and a "Copy diagnostics" affordance; the form stays open with entered values.
- **Disabled**: capability-gated actions render disabled with the provider reason
  (e.g. "SQLite cannot change a column type" / "Materialized views are PostgreSQL-only").
- **Read-only connection**: every mutating action is disabled with
  "Connection is read-only" and the reason is also enforced server-side.

## 6. Architecture

### 6.1 Flow (no exceptions)

```text
UI (ObjectTabs / FormEditor / DDLViewer / ConfirmationDialog)
  → UiCommand::MutateObject { request_id, connection_id, request: ObjectMutationRequest }
  → RuntimeCommand::MutateObject
  → ObjectMutationService::plan(...)   → ObjectMutationPreview   (no SQL executed)
  → ObjectMutationService::apply(...)  → ObjectMutationResult    (policy + capability gated)
  → DbConnector::execute_batch / execute (single transaction where supported)
  → SchemaService::invalidate_target(connection_id, object_ref)
  → RuntimeEvent::ObjectMutationApplied → UiEvent::ObjectMutationApplied
     (UI updates the object tab + triggers a targeted introspection refresh)
```

Two round trips are intentional: **plan** (preview, no side effects) and **apply** (execute).
The preview object carries a fingerprint of the request; apply rejects a mismatched
fingerprint, so the SQL shown is the SQL that runs (master goal §10.4).

### 6.2 Layering rules specific to this phase

- `ObjectMutationService` lives in `crates/core/src/application/` and depends only on
  `DbConnector`, `IntrospectionCache`, `ConnectionRepository`, `ConnectionRegistry`, and
  `DatabaseCapabilities`.
- DDL text is produced only by builders in `crates/core/src/application/ddl_builder.rs`
  (extended), parameterized by `SqlDialect` for identifier quoting.
- The UI sends a typed request; it never sends SQL. The only SQL the UI ever holds is the
  **preview string for display**, returned by `plan`.
- Capability and policy checks happen in the service, before any builder is invoked, so an
  unsupported operation never produces SQL at all.

### 6.3 Cache and refresh strategy

- Apply → `invalidate_target` for the affected object and its dependents (not the whole
  connection) → emit an event → UI refreshes the object tab.
- The Explorer refreshes on the same event, reusing the existing introspection cache
  machinery (`SchemaService::invalidate_cache`, `query_service.rs:117-121`).
- Bulk operations (a multi-statement batch) invalidate the whole schema once, after commit.

### 6.4 Concurrency and drift

- Edit forms load current metadata and bind it to a `metadata_version` (hash of the
  introspected object). Apply re-checks the version; on drift the user is told the object
  changed and offered "reload and re-apply" — never a silent overwrite.
- Long operations (table rewrite on SQLite, matview refresh) are `LongRunning`: cancellable
  and progress-reporting (master goal §10.1).

## 7. Domain model

New module `crates/core/src/domain/object_action.rs` (names are the contract):

```rust
pub enum ObjectKind {
    Table, Column, View, MaterializedView, Index, PrimaryKey, ForeignKey,
    UniqueConstraint, CheckConstraint, Trigger, Sequence, EnumType, DomainType,
    CompositeType, Schema, Database,
}

pub enum ObjectAction {
    Create, Alter, Drop, Rename, Refresh, Enable, Disable, GenerateDdl,
}

pub struct ObjectRef { pub kind: ObjectKind, pub schema: Option<String>, pub name: String, pub parent: Option<String> }

pub struct ObjectMutationRequest {
    pub connection_id: ConnectionId,
    pub action: ObjectAction,
    pub target: Option<ObjectRef>,       // None for Create
    pub definition: ObjectDefinition,    // typed payload, one variant per kind
    pub options: MutationOptions,        // cascade, if_exists, transactional, dry_run
    pub metadata_version: Option<String>,// drift guard for Alter/Drop
}

pub struct ObjectMutationPreview {
    pub statements: Vec<String>,         // exact SQL, in execution order
    pub safety: StatementSafety,         // highest class among statements
    pub long_running: bool,
    pub effects: Vec<MutationEffect>,    // rows/objects affected, data-loss, locks
    pub rollback: RollbackInfo,          // Transactional | NoRollback | UndoScript(String)
    pub fingerprint: String,             // binds preview to apply
}

pub struct ObjectMutationResult {
    pub applied: Vec<String>,            // statements actually executed
    pub refreshed: Vec<ObjectRef>,       // objects whose metadata is now stale
    pub duration_ms: u64,
    pub audit_id: String,
}

pub enum MutationError {
    Unsupported { capability: String, provider: DriverType, reason: String },
    PolicyViolation { class: StatementSafety, policy: String },
    Drift { expected: String, actual: String },
    Validation { field: String, message: String },
    Failed { statement_index: usize, error: DbError, rolled_back: bool },
}
```

`ObjectDefinition` carries one typed variant per object kind (e.g.
`ObjectDefinition::Table(TableDefinition)`, `::Index(IndexDefinition)`, …) with no raw SQL
field. `ddl_builder.rs` converts a definition to statements; `IntrospectResult` metadata
converts back into a definition so `Edit` is a diff, not a re-typing exercise.

Existing types to extend rather than duplicate:

- `domain/schema.rs` — add `MaterializedView`, `Sequence`, `EnumType`, `DomainType`,
  `CompositeType`; add `columns: Vec<Column>` to `View`.
- `domain/capabilities.rs` — add `SchemaCapabilities::{materialized_views, sequences,
  enum_types, domain_types, create_schema, drop_schema, create_database, alter_column_type}`
  consistency (every new flag lands with its implementation, per master goal §4.3).
- `domain/safety.rs` — add `Administrative` and the `LongRunning` modifier (master goal §10.1).
- `domain/cross_connection.rs` `ObjectDependency` — reuse for dependency display.

## 8. APIs, services, ports

### 8.1 Services

`crates/core/src/application/object_mutation_service.rs` (new):

```rust
impl ObjectMutationService {
    pub async fn plan(&self, request: ObjectMutationRequest) -> Result<ObjectMutationPreview, MutationError>;
    pub async fn apply(&self, request: ObjectMutationRequest, fingerprint: &str) -> Result<ObjectMutationResult, MutationError>;
    pub async fn generate_ddl(&self, request: ObjectMutationRequest) -> Result<Vec<String>, MutationError>;
}
```

Internals: `validate(request)` → `capability_gate(kind, action, capabilities)` →
`policy_gate(statements, policy)` → `build(kind, definition, dialect)` →
`ObjectMutationPreview`. `apply` re-runs the same pipeline and compares fingerprints before
executing.

`SchemaService` additions:

- `invalidate_target(connection_id, ObjectRef)` — targeted cache invalidation.
- `get_object_ddl(connection_id, ObjectRef)` — extend the existing table/view path to every
  in-scope object (used by the DDL tab and "Generate SQL").

`ddl_builder.rs` additions (each returns `Vec<String>`, each has per-dialect tests):

- `build_create_table`/`build_drop_table` (exist — extend for identity/generated/collation)
- `build_alter_table_add_column` / `drop_column` (exist) + `build_alter_table_rename_column`,
  `build_alter_column_type`, `build_alter_column_set_default`/`drop_default`,
  `build_alter_column_set_not_null`/`drop_not_null`
- `build_add_constraint` / `build_drop_constraint` for PK/FK/Unique/Check (new)
- `build_create_index` / `build_drop_index` (exist — extend for include columns/method/predicate)
- `build_create_view` / `build_drop_view` (exist) + `build_create_or_replace_view`
- `build_create_materialized_view` / `build_refresh_materialized_view` / `build_drop_materialized_view`
- `build_create_trigger` / `build_drop_trigger` (exist) + `build_alter_trigger_state` (PG)
- `build_create_sequence` / `build_alter_sequence` / `build_drop_sequence`
- `build_create_enum` / `build_alter_enum_add_value` / `build_alter_enum_rename_value` /
  `build_drop_type`; domain/composite equivalents
- `build_create_schema` / `build_drop_schema` / `build_rename_schema`
- `build_create_database` / `build_drop_database` (guarded)

Every builder rejects raw fragments that contain statement separators or destructive tokens —
the existing `validate_raw_fragment` (`ddl_builder.rs:201`) is the pattern to keep and
extend for expressions (defaults, predicates, check bodies).

### 8.2 Ports

Req 1: **do not add a new port for DDL execution.** Reuse `DbConnector::{execute,
execute_batch, execute_parameterized_transaction}`; both providers already implement
transactional DDL with rollback on failure/timeout.

Req 2: add one new port for objects the current introspection cannot describe:

```rust
#[async_trait]
pub trait ObjectCatalogPort: Send + Sync {
    async fn list_sequences(&self, handle: ConnectionHandle, schema: &str) -> Result<Vec<Sequence>, DbError>;
    async fn list_types(&self, handle: ConnectionHandle, schema: &str) -> Result<Vec<TypeInfo>, DbError>;
    async fn list_materialized_views(&self, handle: ConnectionHandle, schema: &str) -> Result<Vec<MaterializedView>, DbError>;
    async fn list_view_columns(&self, handle: ConnectionHandle, schema: &str, view: &str) -> Result<Vec<Column>, DbError>;
}
```

Implementation: PostgreSQL via `pg_catalog`; SQLite returns `DbError::Unsupported` for the
first three and implements `list_view_columns` (SQLite has view columns via `PRAGMA
table_info` on the view name).

Req 3: promote the PG inherent methods (`object_dependencies`, `rename_schema_object`,
`list_partitions`, `list_tablespaces`) behind either `ObjectCatalogPort` or a
`PostgresExtrasPort` so `runtime/src/api.rs:890-1010` stops calling the connector directly
(master goal §2.5, `backend-debt-register.md` P1-01/P1-02).

### 8.3 Runtime and UI wiring

New command/event pairs (all four layers per master goal §4.2):

```text
UiCommand::PlanObjectMutation { request_id, connection_id, request }
UiCommand::ApplyObjectMutation { request_id, connection_id, request, fingerprint }
UiCommand::GenerateObjectDdl  { request_id, connection_id, request }
UiEvent::ObjectMutationPlanned { request_id, preview }
UiEvent::ObjectMutationApplied { request_id, result }
UiEvent::ObjectDdlGenerated    { request_id, statements }
RuntimeCommand / RuntimeEvent mirror these.
```

Translation lives in `crates/native-app/src/translate.rs`. Progress for `LongRunning`
mutations reuses `RuntimeEvent::OperationProgress`.

## 9. Provider behavior

### 9.1 PostgreSQL

Supported: table/column/index/view/matview/constraint/trigger/sequence/type/schema
lifecycle; `ALTER COLUMN TYPE` (with `USING` required for non-trivial casts — the form must
demand it); trigger enable/disable; `CREATE OR REPLACE VIEW`.

Constraints and caveats the UI must respect:

- `DROP … CASCADE` is never implicit; when dependents exist, the confirmation lists them and
  requires an explicit choice.
- `ALTER TABLE … ADD CONSTRAINT` takes an `ACCESS EXCLUSIVE` lock on large tables — the
  confirmation shows a lock warning (master goal §10.4 "Effect").
- Enum value removal is **not supported** by PostgreSQL; the UI offers add/rename only and
  refuses removal with a reason.
- A column type change that narrows data must be flagged as `Destructive`.
- Databases/schemas have owner semantics; the create form offers the current user by default.

### 9.2 SQLite

Supported: table create/drop, column add/drop/rename, index create/drop, view create/drop,
trigger create/drop, constraint changes only via a table rebuild.

Must be gated with reasons (never faked):

| Operation | Behavior |
|---|---|
| `ALTER COLUMN TYPE` | Disabled: "SQLite cannot change a column type; recreate the table and copy data" |
| Drop/alter column with dependencies | SQLite ≥3.35 supports `DROP COLUMN` with restrictions; when the column is indexed/referenced, disable with reason |
| Add FK/CHECK/UNIQUE to an existing table | Disabled with an explicit "requires table rebuild" explanation and a "Generate rebuild script" affordance that produces a documented create-copy-swap script (never auto-applied) |
| Enable/disable trigger | Not offered |
| Materialized views, sequences, enum/domain types, schemas, roles | Hidden with reason |
| `CREATE INDEX` options (method/include/partial) | Only plain btree columns; other options disabled |

SQLite table rebuild (create new → copy → drop old → rename) is a **Destructive, documented
generated script**, not a hidden automatic behavior, until/unless it is implemented as a
first-class typed operation with its own tests. If it is implemented, it must run in a single
transaction and verify `PRAGMA foreign_keys` handling explicitly.

### 9.3 Cross-provider rules

- A mutation plan is always generated for one provider; there is no "portable DDL".
- Capability checks read `DatabaseCapabilities` for the connection's driver; unsupported →
  `MutationError::Unsupported { capability, provider, reason }`.
- SQLite-specific quoting (`"`) and PG quoting (`"` with doubling) both come from
  `SqlDialect::quote_identifier` — no view-side quoting logic.

## 10. UI structure

### 10.1 Shared components used (contract from master goal §4.5)

| Need | Component | Status |
|---|---|---|
| Object header (name/schema/env/badges) | `ObjectHeader` (extract from `table_view.rs:139-327`) | new/generalize |
| Sub-tabs | `ObjectTabs` (generalize `TableView`, `table_view.rs:243-251`) | generalize |
| Property display/edit | `PropertyGrid` | new |
| DDL / SQL preview | `DDLViewer` (extract `draw_ddl_script_card`, `table_editor_view.rs:1363-1431`) | extract |
| Forms | `FormEditor` (`components/form.rs`) | reuse |
| Confirmation | `ConfirmationDialog` (`Dialog` + `ExecutionApproval` + `RiskLevel`) | consolidate |
| Dependency tree | `ObjectTree` / `components/tree.rs` | reuse |
| Empty/error | `EmptyState`, `Alert` | reuse |
| Activity/audit | `ActivityLog` (`components/logs.rs`, promote from gallery) | promote |

### 10.2 Screens to build or extend

1. **Explorer context menus** per object kind: New <object>, Edit, Drop, Rename, Generate DDL,
   Refresh, Add to Favorites (Phase G), Open Dependencies. Triggers currently have no menu
   (`explorer_folders.rs:252-291`) — add one.
2. **Table workspace** (extend `table_metadata_view.rs`): each metadata tab gains an "Edit"
   entry point that opens the appropriate typed form; the DDL tab gains "Generate SQL" and
   "Apply DDL" through the new preview flow (replacing the current free-text DDL card).
3. **Schema object workspace** (`schema_object_view.rs`): grow from Definition/Data to
   ObjectTabs = Definition · Properties · Columns (views) · Parameters (later, routines) ·
   Dependencies · DDL · Query Data.
4. **New-object dialogs/forms**: Table wizard, View editor, Index wizard, Constraint form,
   Trigger form, Sequence form, Type editor, Schema dialog.
5. **Mutation preview panel** (shared): statement list, safety badge, effects, rollback note,
   Generate SQL / Cancel / Apply.
6. **Dependency view**: reuse `ObjectDependency` DTOs and a tree; add reverse lookup where the
   provider supports it.
7. **Audit surface**: mutation entries in `ActivityLog` with connection, class, object, and
   outcome.

### 10.3 Interaction rules

- Applying shows an indeterminate progress state for operations under ~300 ms and a
  cancellable progress bar for longer ones (SQLite rebuild, matview refresh).
- After apply, the object tab reloads metadata and the DDL tab re-renders from introspection
  (verify-by-reintrospection, not by trusting the statement).
- Failed apply keeps the form state and shows the failing statement index.
- Every destructive action requires the object name to be typed when the environment is
  Production (master goal §10.3).

## 11. Safety model

### 11.1 Classification

| Operation | Class |
|---|---|
| Create (table/view/index/sequence/type/schema) | `Mutation` |
| Alter (add column, add constraint, create index, add enum value, rename) | `Mutation` (Destructive when it narrows data or rewrites a table) |
| Drop anything | `Destructive` |
| `TRUNCATE` (offered only as an explicit table action, if at all) | `Destructive` |
| `ALTER COLUMN TYPE` (PG) | `Mutation` when widening, `Destructive` when narrowing or requiring `USING` with lossy cast |
| SQLite table rebuild | `Destructive + LongRunning` |
| `REFRESH MATERIALIZED VIEW` | `Mutation` (non-concurrent) / `Mutation` with lock warning (concurrent) |
| Matview drop with dependents | `Destructive` |
| Schema drop non-empty | `Destructive`, requires explicit cascade choice |
| Database drop | `Destructive + Administrative`, requires confirmation and session check |

### 11.2 Gates (all mandatory, in this order)

1. **Capability gate** — service rejects unsupported kind/action/provider combos.
2. **Policy gate** — `read_only` blocks every mutating class; `allow_ddl`/`allow_destructive`
   (real producers land with I01, but the policy branches already exist in
   `domain/safety.rs:560-580`) block `Ddl`/`Destructive` respectively.
3. **Drift gate** — `metadata_version` must match for Alter/Drop.
4. **Preview gate** — UI must receive and display `ObjectMutationPreview` before apply; the
   fingerprint binds the two calls.
5. **Confirmation gate** — per class, with the stronger Production variant.
6. **Transaction gate** — apply uses one transaction when the provider supports transactional
   DDL (both do), with rollback on failure; the failure carries the statement index.

### 11.3 Explicit prohibitions

- No implicit `CASCADE`.
- No implicit data-loss cast.
- No automatic retry of a partially applied destructive batch.
- No "apply all recommended changes" one-click path (that is Phase F, and even there it is
  review-gated).
- No SQL construction in `crates/ui`.

## 12. Testing strategy

### 12.1 Unit (crates/core)

- Builders: exact SQL per dialect for every builder, including identifier quoting with
  reserved words, mixed case, and embedded quotes; fragment validation rejects
  `;`, `DROP`, `DELETE`, `UPDATE`, `INSERT` inside raw expression fields.
- `ObjectMutationService::plan`: capability rejection per driver, policy rejection per class,
  drift rejection, fingerprint stability, deterministic statement order.
- Definition ↔ metadata round-trip: introspect fixture → build definition → generate DDL →
  (parser-free) structural assertions; for Alter, the diff must produce no statements for an
  unchanged object.
- Safety classifier additions: `Administrative`/`LongRunning` cases.

### 12.2 Service/integration

- `MockDbConnector` assertions: no SQL reaches the connector for a rejected operation;
  exactly the preview statements reach it for an accepted one; invalidation is targeted, not
  global.
- SQLite (always-run, `crates/infrastructure/tests/`): create → introspect → alter → drop per
  object kind; failure mid-batch rolls back; rebuild script produces a valid schema.
- PostgreSQL (live, `tests/pg_integration.rs` ignored suite): the same matrix; plus
  `ALTER COLUMN TYPE` with `USING`, trigger enable/disable, enum add-value, sequence
  alter + `nextval`, schema rename through the promoted service, matview refresh.

### 12.3 UI tests (crates/ui)

- Preview rendering (statements, class badge, effects, rollback note).
- Disabled-with-reason for every SQLite-gated action; assertion that no command is emitted.
- Drift path: apply with a stale version shows the drift message and does not execute.
- Confirmation strength: Production requires typed name; read-only disables the action.
- Redirect: "Generate SQL" opens a query document with the statements and does not execute.

### 12.4 Regression guards

- Table DDL reconstruction tests must keep passing (`schema_service.rs` tests) — a mutation
  framework that changes reconstructed DDL is a regression.
- Introspection cache tests: after apply, the affected object is re-introspected and
  unrelated cache entries survive.

## 13. Runtime verification

Prescribed evidence per object kind, recorded in each milestone's `VERIFICATION.md`
(PostgreSQL and SQLite recorded **separately**):

| Step | Evidence |
|---|---|
| 1 | Native UI: open the object in Explorer; capture the create form filled with valid values |
| 2 | Capture the preview panel with the exact SQL, safety badge, effects, rollback note |
| 3 | Apply; capture the refreshed Explorer + metadata tabs (verify by re-introspection) |
| 4 | Edit an attribute; capture the generated `ALTER`/replace statement and the refreshed state |
| 5 | Drop; capture the confirmation (including Production-strength variant if applicable) and the removal |
| 6 | Error path: attempt an invalid mutation (e.g. drop a table referenced by an FK; add a duplicate index name) and capture the structured error with no partial state |
| 7 | Gated path: attempt a SQLite-unsupported operation and capture the disabled control + reason (no SQL emitted) |
| 8 | Read-only path: attempt any mutation on a read-only connection and capture the block |

Additional runtime evidence required by milestone:

- A01: one full create/drop per provider through the framework (proves the framework, not just
  the builders).
- A02: `REFRESH MATERIALIZED VIEW` observed changing row counts in the Data tab.
- A03: index appears in `EXPLAIN` output / `pg_indexes` after creation.
- A04: FK enforcement observed by attempting an orphan insert (must fail).
- A05: trigger fire observed (row audit table) and PG enable/disable observed in
  `information_schema.triggers`.
- A06: `nextval` sequence behavior after `ALTER SEQUENCE`.
- A07: inserting a value outside an enum fails; adding the value makes it succeed.
- A08: schema rename observed to preserve contained objects.

## 14. Milestone order

| Order | ID | Title | Release | Pri | Prerequisites | Why this order |
|---:|---|---|---|---|---|---|
| 1 | A01 | Typed Object Mutation Framework | v0.2 | P1 | §9/§10 policy + capability design | Everything else is a consumer of this framework; it also fixes the "generic raw DDL" safety gap |
| 2 | A02 | View & Materialized View CRUD | v0.2 | P1 | A01 | Smallest end-to-end proof (builders exist; no constraint complexity); adds view columns |
| 3 | A03 | Index CRUD Wizard | v0.2 | P1 | A01 | Builders exist; high-frequency operation; proves the "disabled options" pattern |
| 4 | A04 | Constraint CRUD (PK/FK/Unique/Check) | v0.2 | P1 | A01 | Highest user value of the remaining table objects; needs new builders + a real unique-constraint query |
| 5 | A05 | Trigger Editor + enable/disable | v0.3 | P2 | A01 | Existing builders; requires PG trigger-state plumbing and pending live enable/disable evidence |
| 6 | A06 | Sequence Management (PG) | v0.3 | P2 | A01 | New catalog + domain; also closes the `sequences` flag-without-code conflict |
| 7 | A07 | Type / Enum / Domain Management (PG) | v0.3 | P2 | A01 | New catalog + domain; closes the `enum_types` flag-without-code conflict; largest introspection addition |
| 8 | A08 | Schema & Database Object Management | v0.3 | P2 | A01 | Requires promoting the PG inherent rename path behind a port; database create/drop is guarded/low priority |

Ordering rules: within v0.2, A02 and A03 may swap or run in parallel; A04 must not start
before A01's gate tests are green. A05–A08 are independent of each other (all depend only on
A01) and may be reordered or deferred individually without breaking the phase.

## 15. Definition of done

A milestone in this phase is done only when all of the following hold:

1. **Plan folder** `docs/plans/active/<slug>/{PLAN,CHECKLIST,FINDINGS,VERIFICATION}.md` exists
   and reflects the actual work; `docs/plans/STATUS.md` matches the state.
2. **P0 = 0, P1 = 0**; accepted findings resolved.
3. **Capability enforcement** is in the backend for every operation the milestone adds; a
   boundary test proves no unsupported SQL reaches the connector.
4. **No SQL built in `crates/ui`** for the milestone's operations (review evidence: grep for
   `CREATE`/`ALTER`/`DROP` string literals in `crates/ui` returns only display fixtures).
5. **Preview + confirmation** implemented per master goal §10.4, including the Production
   variant where the milestone can reach it.
6. **Targeted refresh** verified: after apply, the object's metadata is re-introspected and the
   UI reflects the real state (not the requested state).
7. **Audit record** written for every applied mutation (master goal §10.5).
8. **Provider evidence recorded separately** (PostgreSQL and SQLite), with the SQLite rows
   either implemented or gated-with-reason-and-test. A milestone whose feature is
   PostgreSQL-only is complete when PostgreSQL works **and** SQLite is gated with a test — not
   before.
9. **Quality gates executed and recorded**: `cargo fmt --all -- --check`,
   `cargo clippy --workspace --all-targets -- -D warnings` (with the legacy `tauri-app` scope
   stated if it still fails), `cargo test --workspace`, SQLite provider tests, and the
   PostgreSQL live suite for any PG-touching change.
10. **Runtime walkthrough** captured on the native UI per §13, with screenshots, at the
    supported resolutions in both themes for any visual change.
11. **Docs touched by the milestone** are updated in the same PR: the relevant
    `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` rows (PARTIAL-or-better), the target matrix in
    `docs/goals/goal-full-product.md` §9 if a cell changes, and any release-doc contradiction
    the milestone resolves (master goal §2.5).
12. **Non-goals respected**: no ER edit mode, no partitioning, no implicit cascade, no
    data-movement features pulled into this phase.

Phase A as a whole is done when: every row in master goal §9.2/§9.3 that names a milestone in
this phase reads NOW-or-PARTIAL-with-UI, zero flag-without-code remains in scope
(`sequences`, `enum_types`), and the Explorer can create, alter, and drop every in-scope
object kind on PostgreSQL with a recorded runtime walkthrough, with SQLite showing correct
disabled states and reasons.
