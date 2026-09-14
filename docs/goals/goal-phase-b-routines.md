# DB Pro — Phase B Goal: Functions & Procedures (Routine Workbench)

- Doc ID: `GOAL-PHASE-B`
- Phase: B — Functions / Procedures
- Release targets: v0.2 (B01–B03), v0.3 (B04)
- Priority: P1
- Authority: `docs/goals/goal-full-product.md` (master goal)
- Depends on: `A01` (typed mutation framework) for B03; existing `QueryService` execution path
  for B04.
- Status: PLANNING. No implementation has started.
- Companions: `docs/goals/goal-phase-a-object-crud.md`, `docs/goals/goal-phase-h-ai.md`.

---

## 1. Problem

Stored routines are where application logic lives in a PostgreSQL database, and DB Pro
treats them as a read-only list of names.

What exists today:

- PostgreSQL introspection reads `pg_proc` with `prokind IN ('f','p')` and returns the
  routine name, schema, `routine_type` (FUNCTION/PROCEDURE as a string), return type, and
  `pg_get_functiondef` output (`crates/infrastructure/src/postgres/introspect.rs:762-779`).
- The domain model is a single struct with a stringly-typed kind
  (`crates/core/src/domain/schema.rs:119-128`: `Function { name, schema, routine_type,
  data_type, definition }`).
- The Explorer shows Functions (and, in principle, procedures — the UI branches on the
  `routine_type` string at `crates/ui/src/explorer_folders.rs:169`) and opens a read-only
  Definition tab (`crates/ui/src/schema_object_view.rs`).
- SQLite returns `functions: Vec::new()` unconditionally (`sqlite/introspect.rs:66`), and the
  Explorer gates the Functions folder on the capability flag
  (`crates/ui/src/explorer_view.rs:639-642`).
- The agent can patch SQL in the editor (`patch_query`) but has no routine-aware tool.

What is missing, and why it blocks real work:

1. **No parameter metadata.** You cannot see a routine's arguments, their types, modes
   (`IN`/`OUT`/`INOUT`/`VARIADIC`), or defaults. Overloads collapse into indistinguishable
   rows with the same name.
2. **No execution path.** There is no argument form and no `CALL` helper. Running a procedure
   means hand-writing `CALL schema.proc(...)` in the editor, where the classifier treats
   `CALL` as `Destructive` (`domain/safety.rs:94`) — correct for safety, but with no
   routine-aware affordance the user gets a confirmation for something they cannot inspect.
3. **No lifecycle.** There is no create, replace, or drop path. Editing a function means
   copying its definition out of a read-only tab into a new query.
4. **No dependencies.** Nothing shows which routines depend on a table or on another routine
   beyond a substring heuristic in the existing dependency code.
5. **The capability flag is inconsistent.** `SchemaCapabilities.functions` is `true` for
   PostgreSQL and `false` for SQLite (`capabilities.rs:148, 203`), but **nothing in the
   backend consults it** (master goal §2.5 C2). The only gate is a UI folder check.

The result: a PostgreSQL user cannot browse, run, or manage the objects that hold their
business logic, and a SQLite user has no stated reason for the missing folder.

## 2. Scope

### 2.1 In scope

| Capability | PostgreSQL | SQLite |
|---|---|---|
| Routine list with signature and kind (function vs procedure) | yes | gated |
| Overload distinction (identity by parameter signature) | yes | gated |
| Parameter metadata: name, type, mode, default, variadic | yes | gated |
| Return type / return table columns | yes | gated |
| Attributes: language, volatility, strictness, security definer, cost/rows estimate | yes | gated |
| Source view (`pg_get_functiondef`) + parsed signature header | yes | gated |
| Dependency listing (routine → objects it uses; object → routines that use it) | yes | gated |
| Create / create-or-replace / drop with preview and confirmation | yes | gated |
| Execute with typed arguments (function `SELECT`, procedure `CALL`) | yes | gated |
| Output rendering: result grid, notices/messages, affected rows, errors | yes | gated |
| Search/filter routines by name and signature | yes | gated |
| Agent integration: explain routine, fix routine, generate routine skeleton, inspect dependencies | yes (read-only tools first) | gated |

### 2.2 Out of scope but explicitly planned for later

- Debugging (breakpoints, stepping, variable inspection) — see §3.
- Profiling/`EXPLAIN ANALYZE` of routine bodies.
- PL/pgSQL language-server-style analysis (semantic completion inside the body).

## 3. Non-goals

- **No debug server integration** (pldebugger, `pg_debugger`): no attach, breakpoints, or step
  execution in this phase. Explicitly deferred (master goal §5 area B non-goals).
- **No language server / semantic analysis inside routine bodies.** The editor provides
  SQL/PL-pgSQL highlighting and diagnostics at the same level as the query editor today — no
  more.
- **No SQLite routine emulation.** SQLite has no stored procedures; the UI must state that and
  offer nothing that pretends otherwise.
- **No routine scheduling** (that is Phase J), no routine-level permissions UI (Phase E), no
  routine diffing (Phase F adds routines to the diff, but authoring the diff is not this
  phase).
- **No editing of system/extension-owned routines** — only user routines; system schemas are
  already excluded from introspection.
- **No automatic re-write of a routine by AI without preview + confirmation.**

## 4. Current implementation (code-verified baseline)

| Element | Reality | Evidence |
|---|---|---|
| PG routine introspection | Works for `prokind IN ('f','p')` | `postgres/introspect.rs:762-779` (`pg_get_function_result`, `pg_get_functiondef`) |
| Domain model | `Function { name, schema, routine_type: String, data_type, definition }`; no `Procedure` type | `core/src/domain/schema.rs:119-128` |
| `IntrospectResult` | Carries `functions: Vec<Function>` | `core/src/domain/schema.rs:138-167` |
| Parameter metadata | **Absent** — no `pg_get_function_arguments`, no `proargnames`/`proargtypes`/`proargmodes`/`proargdefaults` query | grep of `postgres/introspect.rs` |
| Explorer rows | Functions folder with a routine-type branch; procedures appear in the same folder | `ui/src/explorer_folders.rs:164-231`, branch at `:169` |
| Capability gate | UI-only folder check | `ui/src/explorer_view.rs:639-642`; flag at `capabilities.rs:148,203`; **no backend enforcement** |
| Read-only view | Definition tab via `CodeBlock`; Data tab exists for views | `ui/src/schema_object_view.rs:131-160` |
| Execution | Generic `QueryService::execute`/`execute_multi`; `CALL` classified `Destructive` | `core/src/application/query_service.rs:137,186`; `domain/safety.rs:94` |
| Parameters in the domain | `QueryParam` enum exists and adapters bind it; **no UI** | `core/src/domain/query.rs:24-51`; `postgres/query_mapper.rs:9` |
| Dependencies | Generic `pg_depend` `n`/`a` via an inherent PG method (not a port) | `infrastructure/postgres/cross_connection.rs:5-61`, `connector.rs:606` |
| Builders for routines | **Absent** | no function/procedure builders in `ddl_builder.rs` |
| Agent tooling | No routine-aware tool; `patch_query` can edit routine SQL as text | `runtime/src/agent.rs:565-634` |

## 5. UX

### 5.1 Explorer

```text
<schema>
 └─ Functions            (functions with return type; signature visible on the row)
 └─ Procedures           (separate folder; hidden+reason on SQLite)
```

- Each row shows `name(param_types) → return_type` truncated with a tooltip for the full
  signature; overloads are separate rows with a count badge on the name
  (`name ×3`).
- Context menu: Open (source), Execute…, Generate DDL, Copy Name, Copy Signature,
  Show Dependencies, Show Referencing Objects, Drop…, Add to Favorites (Phase G).
- Filter box filters by name **and** by parameter type (e.g. `uuid` finds routines taking a
  uuid).

### 5.2 Routine workspace tab

`ObjectTabs` (master goal §4.5), in this order:

```text
Source | Parameters | Dependencies | DDL | Execute | Query Data (functions returning a set)
```

- **Source**: full `pg_get_functiondef` text, read-only by default with an **Edit** toggle
  that switches to the `CodeEditor` (same editor as the query workspace — no new editor).
  Edit mode shows a diff against the original before applying.
- **Parameters**: `PropertyGrid` of parameter rows (name, mode, type, default) plus the
  routine attributes (language, volatility, strict, security definer, cost/rows). Read-only in
  B02; editable through the form in B03.
- **Dependencies**: two lists — *Uses* (tables/views/routines this routine references) and
  *Used by* (views/routines/triggers that reference it), each row opening the target object.
- **DDL**: reconstructed `CREATE OR REPLACE` statement in the shared `DDLViewer`, with Copy,
  Open in Query, and Apply (through A01's preview flow).
- **Execute**: the argument form (§5.3).
- **Query Data**: for set-returning functions, a shortcut that runs the function with default
  or last-used arguments and shows the result grid.

### 5.3 Execute form

```text
Function: public.calculate_invoice_total(customer_id uuid, as_of date DEFAULT CURRENT_DATE)
                                                                  → numeric

Arguments
  customer_id   uuid   [ 7f3c…            ]   NULL ☐        (required)
  as_of         date   [ 2026-09-14       ]   NULL ☐        default: CURRENT_DATE

Options
  [x] Wrap in a transaction         [ ] Show generated SQL
  Result limit: [ 500 ]             Timeout: [ connection default ▾ ]

[ Generate SQL ]                                   [ Cancel ]  [ Execute ⌘⏎ ]
```

Rules:

- Each argument is a typed input bound to its parameter type (`QueryParam`) — **never string
  interpolation** (master goal §4.1).
- Required parameters with no default block execution with an inline message.
- `NULL` is a first-class toggle, distinct from "empty" and from "use default".
- "Show generated SQL" renders the exact statement with positional placeholders, so a user
  can copy it to the editor.
- Functions are executed as `SELECT * FROM schema.fn($1, $2)` (or `SELECT schema.fn(...)` for
  scalar); procedures as `CALL schema.proc($1, $2)`.
- `CALL` is `Destructive`-classified, so the Execute action always shows the confirmation
  payload (target, SQL, effect, rollback note) even in Development.
- After execution, the output pane shows: result grid (if any), notices/messages list
  (PG `NOTICE`/`WARNING`/`INFO` with severity and text), row count, duration, and errors with
  the structured error envelope.

### 5.4 Error, empty, and disabled states

- **Empty**: "No functions in schema public" with a Create action.
- **Error**: a routine whose definition cannot be fetched (dropped concurrently, permission)
  shows an inline error with Retry.
- **Disabled (SQLite)**: the Functions/Procedures folders are absent from the tree, and the
  capability reason is surfaced in the schema header's "unavailable features" affordance
  (the same surface Phase D/A use) — never a silent omission.
- **Disabled (read-only connection)**: Edit/Execute/Drop are disabled with
  "Connection is read-only"; the backend enforces the same rule.

## 6. Architecture

### 6.1 Flow

```text
Browse:   Explorer → SchemaApi::list_routines(schema)      (cached introspection + targeted fetch)
Source:   SchemaApi::routine_source(signature)             (no full re-introspection)
Edit:     RoutineForm → UiCommand::PlanObjectMutation(A01) → preview → Apply → SchemaApi::routine_source
Execute:  UiCommand::ExecuteRoutine { connection_id, signature, args: Vec<QueryParam>, options }
          → RuntimeCommand::ExecuteRoutine
          → QueryService::execute_multi(connection, sql, params)   (existing path, parameterized)
          → result grid + notices
```

### 6.2 Layering rules

- `RoutineService` (new) lives in `crates/core/src/application/` and depends on
  `DbConnector` + a new `RoutinePort`. It never touches a connector directly and never builds
  SQL in the UI.
- Routine DDL text is produced by `ddl_builder.rs` builders (A01 owns the builder file; B03
  adds routine builders there rather than creating a second DDL module).
- Execution reuses `QueryService` — no second execution path, no second connection.
- SQLite support is expressed as a capability check in `RoutineService`, not as a driver
  branch in the UI.

### 6.3 Caching

- Routine lists come from the existing introspection cache; a targeted
  `list_routines(schema)` fetch refreshes one schema without invalidating the connection.
- Source text is fetched on demand and cached per (connection, signature) with a short TTL;
  applying a change invalidates that signature plus the schema's routine list.
- Dependency lookups are computed per routine on demand (never eagerly for a whole schema).

## 7. Domain model

New module `crates/core/src/domain/routine.rs`:

```rust
pub enum RoutineKind { Function, Procedure }

pub enum ParameterMode { In, Out, InOut, Variadic, Table }

pub struct RoutineParameter {
    pub name: Option<String>,        // unnamed parameters exist in PG
    pub mode: ParameterMode,
    pub data_type: String,           // format_type() string
    pub default: Option<String>,     // expression text, never executed
    pub ordinal: u32,
}

pub struct RoutineSignature {        // identity, stable across UI and tools
    pub schema: String,
    pub name: String,
    pub kind: RoutineKind,
    pub parameters: Vec<RoutineParameter>,
}

pub struct Routine {
    pub signature: RoutineSignature,
    pub return_type: Option<String>,
    pub returns_set: bool,
    pub language: String,
    pub volatility: Volatility,      // Volatile | Stable | Immutable
    pub strict: bool,
    pub security_definer: bool,
    pub cost: Option<f64>,
    pub rows_estimate: Option<f64>,
    pub source: String,              // pg_get_functiondef output
    pub owner: Option<String>,
    pub comment: Option<String>,
}

pub struct RoutineDefinition {       // used by the create/replace form
    pub signature: RoutineSignature,
    pub returns: Option<String>,
    pub body: String,
    pub language: String,
    pub volatility: Volatility,
    pub strict: bool,
    pub security_definer: bool,
    pub replace: bool,               // CREATE OR REPLACE vs CREATE
}
```

Migration note: `Function` in `domain/schema.rs:119` is replaced by `Routine`; the UI and
`IntrospectResult` are updated in the same PR (B01). To avoid a big-bang change,
`IntrospectResult` keeps a `routines: Vec<Routine>` field and the `functions` field is removed
only after the Explorer, `schema_object_view`, the agent context builder, and the DTO layer
(`runtime/src/api.rs::FunctionSummary`) are migrated — one milestone, one migration, no
parallel types left behind.

`RoutineSignature` is the identity key used by: the Explorer row, the workspace tab, the
execute form, the agent tools, and the dependency graph. No `String`-typed routine identity
anywhere.

## 8. APIs, services, ports

### 8.1 Port

```rust
#[async_trait]
pub trait RoutinePort: Send + Sync {
    async fn list_routines(&self, handle: ConnectionHandle, schema: &str) -> Result<Vec<Routine>, DbError>;
    async fn routine_source(&self, handle: ConnectionHandle, signature: &RoutineSignature) -> Result<String, DbError>;
    async fn routine_dependencies(&self, handle: ConnectionHandle, signature: &RoutineSignature)
        -> Result<Vec<ObjectDependency>, DbError>;
    async fn routines_referencing(&self, handle: ConnectionHandle, target: &ObjectRef)
        -> Result<Vec<RoutineSignature>, DbError>;
}
```

- PostgreSQL implementation uses `pg_proc` joined to `pg_namespace`, `pg_language`,
  `pg_get_function_arguments`, `pg_get_function_result`, `pg_get_functiondef`,
  `proargmodes`/`proargnames`/`proargdefaults` for parameters, and `pg_depend` for
  dependencies.
- SQLite implementation returns `DbError::Unsupported { capability: "routines", provider:
  SQLite, reason: "SQLite has no stored functions or procedures" }` for every method, and its
  capability flag stays `false`. This is what makes the folder gating honest (master goal §9.9).

### 8.2 Service

```rust
impl RoutineService {
    pub async fn list(&self, connection_id: ConnectionId, schema: &str) -> Result<Vec<Routine>, DbError>;
    pub async fn source(&self, connection_id: ConnectionId, signature: &RoutineSignature) -> Result<String, DbError>;
    pub async fn dependencies(&self, connection_id: ConnectionId, signature: &RoutineSignature)
        -> Result<RoutineDependencyView, DbError>;                       // uses / used_by
    pub fn build_execute_request(&self, signature: &RoutineSignature, args: &[RoutineArgument])
        -> Result<RoutineExecuteRequest, MutationError>;                 // pure, testable
    pub async fn execute(&self, request: RoutineExecuteRequest) -> Result<MultiQueryResult, DbError>;
    pub async fn plan_definition(&self, def: RoutineDefinition)
        -> Result<ObjectMutationPreview, MutationError>;                 // delegates to A01
}
```

`build_execute_request` is deliberately pure so that argument validation, NULL/default
handling, and statement construction are unit-testable without a database. It produces
`(sql, Vec<QueryParam>)` where `sql` contains only placeholders.

### 8.3 Runtime/UI wiring

```text
UiCommand::ListRoutines        { request_id, connection_id, schema }
UiCommand::LoadRoutineSource   { request_id, connection_id, signature }
UiCommand::LoadRoutineDeps     { request_id, connection_id, signature }
UiCommand::ExecuteRoutine      { request_id, connection_id, signature, args, options }
UiEvent::RoutinesLoaded        { request_id, routines }
UiEvent::RoutineSourceLoaded   { request_id, signature, source }
UiEvent::RoutineDepsLoaded     { request_id, dependencies }
UiEvent::RoutineExecuted       { request_id, results, notices, duration_ms }
```

Create/replace/drop use the A01 pairs (`PlanObjectMutation`/`ApplyObjectMutation`) with
`ObjectKind::Routine` added to the A01 `ObjectKind` enum — routines are an object kind, not a
parallel pipeline.

`RuntimeEvent` additions mirror these; `native-app/src/translate.rs` maps them.

### 8.4 Agent tools (documented here, implemented in Phase H)

| Tool | Risk | Confirmation | Backend |
|---|---|---|---|
| `inspect_routines` (list/signatures) | ReadOnly | none | `RoutineService::list` |
| `inspect_routine_source` | ReadOnly | none | `RoutineService::source` |
| `explain_routine` | ReadOnly | none | source + dependencies + model |
| `fix_routine` / `generate_routine` | Mutation (patch preview) | required (patch preview) | produces a `RoutineDefinition` patch; apply via A01 |

These tools must not open their own connection and must not execute routines; execution stays
a user action in B04.

## 9. Provider behavior

### 9.1 PostgreSQL

| Behavior | Detail |
|---|---|
| List | `prokind IN ('f','p')`, user schemas only; ordered by schema, name, then parameter-type list for stable overload ordering |
| Overloads | Identity includes parameter types; the UI must never merge two overloads under one row |
| Unnamed parameters | Legal; the form shows `$1`-style positional labels and warns that named arguments are unavailable |
| `OUT`/`INOUT` parameters | Must be rendered as output columns, not inputs; a routine with `OUT` params takes fewer inputs |
| `VARIADIC` | Supported; the form takes a list value and warns that it must be last |
| Defaults | Shown as expression text; "use default" omits the argument from the call |
| `SECURITY DEFINER` | Displayed prominently (privilege implication) and requires an explicit checkbox + warning when setting it |
| Language | Displayed; create supports `sql` and `plpgsql` at minimum, with other installed languages listed read-only |
| Procedure | Executed with `CALL`; no return value; `OUT` parameters come back as a result row |
| Notices | `NOTICE`/`WARNING`/`INFO`/`DEBUG` from execution are surfaced with severity; they must not be dropped on the floor |
| Enable/disable | Not applicable (routines have no enable flag) |

### 9.2 SQLite

| Behavior | Detail |
|---|---|
| Any routine operation | `DbError::Unsupported` with a stable reason; capability flag `false` |
| UI | Functions/Procedures folders absent; an "Unavailable in SQLite" section in the schema properties lists the reason |
| Emulation | Forbidden — no user-defined-function shim, no "create a trigger instead" advice that pretends equivalence |

### 9.3 Query-parameter typing

The execute form maps each parameter's PG type to a `QueryParam` variant and a field widget:

| PG type family | `QueryParam` | Widget |
|---|---|---|
| bool | `Bool` | checkbox / tri-state with NULL |
| smallint/int/bigint | `Int64` | numeric input (i64-serialized per existing IPC contract) |
| real/double/numeric/decimal | `Float64`/`Decimal` | numeric input; `Decimal` kept as a string to avoid precision loss |
| text/varchar/char/name | `Text` | text input |
| uuid | `Uuid` | text input with UUID validation |
| timestamp/date/time | `DateTime` | date/time picker producing a canonical string |
| interval | `Interval` | text input with validation |
| inet/cidr | `Inet` | text input with validation |
| json/jsonb | `Json` | multiline editor with JSON validation |
| bytea | `Bytes` | file picker / hex input (read-only display of result bytes) |
| arrays/enums/composites | not directly bindable | must be passed as text with an explicit cast in the generated SQL, and the form states this limitation instead of failing silently |

Unsupported-to-bind types are a **typed validation failure** with the type name, never a
silently dropped argument.

## 10. UI structure

### 10.1 Components

| Need | Component | Status |
|---|---|---|
| Routine list | `ObjectList` (master goal §4.5) | new, shared with A/E/D |
| Routine header (signature, kind, badges) | `ObjectHeader` | generalize from table header |
| Tabs | `ObjectTabs` | generalize |
| Parameter/attribute display | `PropertyGrid` | new, shared |
| Source editing | `CodeEditor` (`crates/ui/src/editor/*`) | reuse — do not fork |
| DDL | `DDLViewer` | extract from table DDL card |
| Execute form | `FormEditor` + typed field widgets | reuse + add type widgets |
| Output | existing result grid + messages pane (`query_view.rs` output tabs) | reuse |
| Dependency lists | `ObjectList` + navigation to target | new |
| Notices | `ActivityLog`/messages list | reuse |

### 10.2 Screens

1. **Explorer** — Functions and Procedures folders, list rows, filter, context menus.
2. **Routine tab** — Source / Parameters / Dependencies / DDL / Execute / Query Data.
3. **Edit mode** — source editor with a diff-vs-original view and an Apply that routes through
   A01's preview panel.
4. **Execute tab** — the form in §5.3 plus an output area with Results / Messages / Errors.
5. **Schema properties** — an "Unavailable features" block listing SQLite-gated object kinds
   with reasons (shared with Phase A/D).
6. **Agent panel** — routine context chip (signature) so agent actions target the open routine.

### 10.3 Interaction rules

- Switching routines in the tree preserves the tab layout but resets the form only after
  confirming unsaved edits.
- The Execute form remembers the last-used arguments per signature **in UI storage only**,
  never including sensitive values marked as secret (there is no secret marker in v1; revisit
  if a "mask parameter" feature is added).
- Executing a procedure always confirms; executing a function that is `VOLATILE` and
  `SECURITY DEFINER` shows a warning badge before execution.
- Long-running executions are cancellable through the existing query-cancel path (note the
  provider asymmetry in master goal §9.8: per-execution cancel is currently gated on
  PostgreSQL — B04 must surface this honestly rather than offering a Stop button that does
  nothing).

## 11. Safety model

| Operation | Class | Gate |
|---|---|---|
| Browse list / source / dependencies | `ReadOnly` | None |
| Execute a function (`SELECT`) | `ReadOnly` if the routine is `IMMUTABLE`/`STABLE` **and** the planner can prove no data modification; otherwise `Mutation` | Confirmation when `Mutation`; `QueryService` policy applies |
| Execute a procedure (`CALL`) | `Destructive` (existing classifier, `domain/safety.rs:94`) | Always confirm; Production requires typed routine name |
| Create routine | `Mutation` | Preview + confirm; read-only blocked |
| `CREATE OR REPLACE` routine | `Mutation` (Destructive if a signature changes incompatibly — callers may break) | Preview + confirm + warning listing existing callers when known |
| Drop routine | `Destructive` | Preview + confirm + dependency warning (views/triggers/routines that depend on it) |
| `SECURITY DEFINER` toggle | Security-relevant | Explicit checkbox + warning text; never default-on |

Additional rules:

- Arguments are **always** bound parameters. The generated SQL contains placeholders only;
  the parameter list is sent separately through `QueryParam`. Existing tests in
  `crates/core/src/application/sql_builder.rs` and the adapter mappers are the model to
  follow.
- Routine source is never executed from the preview pane; only Apply routes to execution.
- The agent may propose routine source, but the user must preview and confirm before it is
  applied; the agent never calls `CALL` itself.

## 12. Testing strategy

### 12.1 Unit

- `RoutineSignature` identity: two overloads differ; ordering is deterministic; unnamed
  parameters produce stable keys.
- `build_execute_request`: required-missing rejection; NULL vs default; type-to-`QueryParam`
  mapping per family; unsupported type produces a typed validation error naming the type;
  variadic handling; `OUT` parameters excluded from inputs; generated SQL contains only
  placeholders (assert no literal argument value appears in the SQL string).
- Routine builders (B03): `CREATE FUNCTION`, `CREATE OR REPLACE FUNCTION`, `CREATE PROCEDURE`,
  `DROP FUNCTION`/`DROP PROCEDURE` with parameter-type lists (PG requires the argument types
  to disambiguate overloads — a classic bug source), identifier quoting, dollar-quote body
  handling.
- Capability gating: SQLite rejects every routine operation with `Unsupported` and the reason
  key; no SQL is built.

### 12.2 Service / integration

- PostgreSQL live (`crates/infrastructure/tests/pg_integration.rs`, ignored suite): list with
  overloads and parameter metadata; source fetch; dependency lookup for a routine using a table
  and another routine; create → replace → drop round-trip; execute a function with
  typed args including NULL and default; execute a procedure returning `OUT` values; capture a
  `RAISE NOTICE`.
- SQLite (always-run): every routine call returns `Unsupported`; no SQL is issued (mock
  assertion); capability flag is `false` and consistent with the implementation
  (flag/implementation consistency test — this is the test that prevents a repeat of the C2
  conflict).
- Service tests with `MockRoutinePort` for caching, invalidation after apply, and
  permission-denied mapping.

### 12.3 UI

- List rendering with overloads, truncation, tooltips, filter by name and type.
- Execute form: required/NULL/default rendering; disabled Execute when invalid; confirmation
  for `CALL`; result grid + notices rendering; error rendering with the structured envelope.
- Edit flow: source diff view, Apply routes through preview, stale-signature drift message.
- Disabled states: SQLite folders absent + reason surface; read-only connection disables
  Edit/Execute/Drop.
- Regression: the query workspace and view/table object tabs must not regress when
  `Function` is replaced by `Routine`.

### 12.4 Runtime (manual) evidence

See §13.

## 13. Runtime verification

Recorded separately per provider in the milestone `VERIFICATION.md`:

**PostgreSQL**

1. Open a schema with ≥2 overloads and a procedure; capture the Explorer showing distinct
   rows with signatures and the overload badge.
2. Open the routine tab; capture Source, Parameters (including an `OUT` parameter and a default
   value), Dependencies (routine → table), DDL.
3. Execute a function with: a required argument, a NULL argument, and an omitted defaulted
   argument; capture the result grid, the duration, and a `RAISE NOTICE` message.
4. Execute a procedure via `CALL`; capture the confirmation payload and the `OUT` result row.
5. Create a routine from the form; capture the preview SQL and the post-apply source fetch.
6. Replace the routine body; capture the diff and the refreshed source.
7. Drop the routine; capture the dependency warning and the removal from the Explorer.
8. Error path: call a routine with a wrong argument type; capture the structured error.
9. Read-only path: attempt Edit and Execute on a read-only connection; capture the block.

**SQLite**

1. Capture the Explorer without routine folders plus the "Unavailable in SQLite" reason
   surface.
2. Attempt each routine action programmatically (test) and confirm `Unsupported` with no SQL
   emitted; capture the automated result as evidence.

## 14. Milestone order

| Order | ID | Title | Release | Pri | Prerequisites | Rationale |
|---:|---|---|---|---|---|---|
| 1 | B01 | Routine Domain Model & Capability Gating | v0.2 | P1 | A01 gate patterns | `Routine` replaces the stringly-typed `Function`; backend gating fixes the flag-without-enforcement gap; everything else depends on the identity model |
| 2 | B02 | Routine Explorer & Source View | v0.2 | P1 | B01 | Makes the existing introspection usable before adding mutation; establishes the tab layout and targeted fetch |
| 3 | B03 | Routine Source Editor (create/replace/drop) | v0.2 | P1 | B01, B02, A01 | Requires the mutation framework for preview/apply; completes the lifecycle |
| 4 | B04 | Routine Execute Form & Result Rendering | v0.3 | P1 | B03 | Deliberately after authoring: the form's typed-argument machinery is the most complex part and benefits from the routine identity being stable |

Ordering rules: B01 and B02 may run as one PR if the reviewer accepts the scope, but B02 must
not start before the `Routine` migration is complete (no interim dual model). B04 may start
in parallel with A05–A08.

## 15. Definition of done

1. **Plan folder** exists for each milestone; `docs/plans/STATUS.md` matches.
2. **P0 = 0, P1 = 0.**
3. **`Function` is fully retired**: no `routine_type: String` comparisons remain in
   `crates/ui` or `crates/core`; the DTO layer and agent context use `RoutineSignature`.
4. **Parameter metadata** is introspected and displayed for overloads, modes, defaults, and
   variadic parameters with live PostgreSQL evidence.
5. **Execution** works with typed arguments, NULL, and defaults; generated SQL contains
   placeholders only (asserted by test and by code review); notices/messages are surfaced.
6. **Lifecycle** create/replace/drop works through A01's preview + confirmation with a
   dependency warning on drop.
7. **SQLite gating** is backend-enforced (not UI-only), returns `Unsupported` with a stable
   reason, emits no SQL (mock-asserted), and the reason is visible in the UI.
8. **Capability consistency test** passes: `SchemaCapabilities.functions` is `true` for
   PostgreSQL *because* `RoutinePort` is implemented, and `false` for SQLite *because* it is
   not — the test fails if the flag and the implementation disagree (master goal §4.3).
9. **Quality gates executed and recorded** (fmt, clippy with stated tauri scope,
   `cargo test --workspace`, SQLite suite, PostgreSQL live suite).
10. **Runtime walkthroughs** in §13 captured per provider, with the SQLite evidence being the
    gating tests rather than a UI walkthrough.
11. **Docs updated**: capability matrix rows for Functions/Procedures and the master goal §9.4
    matrix reflect the delivered state; any resolved contradiction (C2 for the `functions`
    flag) is recorded.
12. **Non-goals respected**: no debugger, no body-level analysis, no routine emulation on
    SQLite, no execution from agent tools.

Phase B is complete when a PostgreSQL user can browse overloaded routines with full parameter
metadata, read and edit source, execute with typed arguments and read the output, and drop with
a dependency warning — all with recorded runtime evidence — while a SQLite user sees an
explicit, backend-enforced unavailability reason.
