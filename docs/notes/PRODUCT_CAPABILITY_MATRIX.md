# DB Pro — Product Capability Matrix (Code-Evidence Audit)

- Date: 2026-09-14
- Branch: `feature/product-audit-roadmap`
- Method: source inspection of `crates/{core,infrastructure,runtime,ui,native-app,tauri-app}` + `docs/release/*` + `docs/architecture/*`. No code changed.
- Companion docs: `docs/notes/PRODUCT_ROADMAP.md`, `docs/notes/product-roadmap-dbeaver-parity.md` (original vision, retained).
- UI scope is the native `egui` app only. Anything existing solely under `_archive/` is flagged, never counted as capability.

## Legend

Status values (only these are used):

| Status | Meaning |
|---|---|
| DONE | Implemented, tested, no open P1 |
| RUNTIME_VERIFY | Source + automated tests acceptable; provider/UI runtime evidence still pending (per `FEATURE_LIFECYCLE.md`) |
| PARTIAL | Works with documented caveats or narrow scope |
| INSPECT_ONLY | Read/introspect only; no mutation path |
| BACKEND_ONLY | Backend implemented; no native UI trigger |
| UI_ONLY | UI affordance exists without real backend behind it (rare; called out explicitly) |
| MISSING | No implementation |
| DEFERRED | Intentionally excluded for now (needs explicit decision to schedule) |

Priority: `P0` = release blocker, `P1` = core product capability, `P2` = important productivity/polish, `P3` = advanced/later.
Target Phase: `v0.1` (RC closure only) / `A`..`H` (see `PRODUCT_ROADMAP.md`) / `Later`.

Columns `PostgreSQL` / `SQLite` describe provider behavior. `Backend` = domain+application+adapter state. `Native UI` = egui wiring state.

---

## 1. Sidebar / Activity Bar

Current rail (`crates/ui/src/navigation_view.rs:412-460`): Explorer, Queries, History, Transfers, Monitor, ER Diagram, Agent (right panel toggle), Settings (bottom). Topbar has fake search box → Quick Open palette, Commands palette, theme toggle.

| Feature | PostgreSQL | SQLite | Backend | Native UI | Safety | Tests | Runtime Evidence | Status | Priority | Target Phase | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Explorer activity | yes | yes | DONE | WIRED (`explorer_view.rs`, `explorer_tree.rs`) | n/a | UI + integration | PENDING native traversal | RUNTIME_VERIFY | P0 | v0.1 | Tree: connection → database → schema → Tables/Views/Functions/Triggers → Columns/FKs/Indexes |
| Search activity (dedicated) | — | — | MISSING | MISSING | n/a | — | — | MISSING | P2 | G | Only palette Quick Open (tables ≤100 + static items, `palette_view.rs:150-163`) + local Explorer/Diagram/editor find |
| Queries activity | yes | yes | DONE | WIRED (`navigation_view.rs:691-748`, open docs + new/duplicate/close) | staged-change guards | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| History activity | yes | yes | PARTIAL | WIRED (saved queries + folders + dual local history) | n/a | UI | PENDING | RUNTIME_VERIFY | P1 | v0.1/G | Server `QueryApi::history` exists but unwired; native history is local (`app.rs:286-288`, caps 20/500) |
| Data activity (dedicated) | — | — | PARTIAL | MISSING | n/a | — | — | MISSING | P2 | C | Table/Query workspaces exist as tabs; no top-level Data icon yet |
| ER activity | yes | yes | DONE | WIRED (`diagram_view.rs` + `diagram/`) | n/a | 98 diagram + workspace | PENDING native large-schema | RUNTIME_VERIFY | P0 | v0.1 | |
| Agent activity | yes | yes | DONE | WIRED (right panel, Ask/Edit/Agent) | confirmation-gated | 567-agent-era + UI | PASS panel/IME isolate (older) | RUNTIME_VERIFY | P0 | v0.1 | Preview label per LIM-007 |
| Monitoring activity | no | no | MISSING | PLACEHOLDER (`draw_activity_placeholder` + COMING SOON, `navigation_view.rs:616-621`) | n/a | — | — | MISSING | P2 | D | No `UiCommand`; no backend service |
| Transfer activity | no | no | PARTIAL (export+backup only) | PLACEHOLDER (`navigation_view.rs:610-615`) | n/a | — | — | MISSING | P1 | C | Export engine backend-only; import missing entirely |
| Settings activity | yes | yes | PARTIAL | PARTIAL (Appearance + Backup/Restore paths only) | readonly-gated backup | UI | PENDING | PARTIAL | P1 | v0.1/G | Editor/keybindings/providers/advanced settings have no page |

---

## 2. Database Explorer — object support

Introspection sources: `crates/infrastructure/src/postgres/introspect.rs`, `crates/infrastructure/src/sqlite/introspect.rs`. Domain: `crates/core/src/domain/schema.rs`.

| Object | PostgreSQL | SQLite | Backend | Native UI | Safety | Tests | Runtime Evidence | Status | Priority | Target Phase | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Connections | DONE | DONE | DONE (`connection_service.rs`, `registry.rs`) | WIRED (dialog, test, dup, SSH/SSL) | secret redaction, validation | integration | PENDING native | RUNTIME_VERIFY | P0 | v0.1 | Folders/tags/favorites: domain supports, UI drops (`translate.rs:41-42`) |
| Databases (create/drop/list) | MISSING | n/a | MISSING (only active `database` string) | MISSING | — | — | — | MISSING | P2 | A | Explorer shows connection database node only |
| Schemas | INSPECT | n/a (single `main`) | PARTIAL (inspect; no create/drop service) | WIRED (tree + combos) | n/a | integration | PENDING | RUNTIME_VERIFY | P0 | v0.1 | PG rename exists only as inherent method (`connector.rs:648-666`), not core service |
| Tables | INSPECT+DDL-gen | INSPECT+DDL-gen | PARTIAL (generic `execute_ddl` only, no typed service) | WIRED (7 sub-tabs: Data/Structure/Indexes/FKs/Constraints/Dependencies/DDL) | policy-checked `execute_ddl` | integration + UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | No typed create/alter/drop service |
| Columns | INSPECT | INSPECT (no generated/identity/collation) | PARTIAL (add/drop builders; no alter-type/rename builders) | WIRED (Structure tab) | — | integration | PENDING | RUNTIME_VERIFY | P1 | A | Capability flags `alter_column_type/rename_column` exist but unchecked in `schema_service.rs` |
| Views | INSPECT+create/drop builders | INSPECT+create/drop builders | PARTIAL (no ALTER VIEW; no view columns) | WIRED read-only (Definition + Data) | policy-checked | integration | PENDING | PARTIAL | P1 | A | `build_create_view/drop_view` (`ddl_builder.rs:114-128`) |
| Materialized Views | MISSING (no `pg_matviews` query) | n/a | MISSING (no domain struct) | MISSING | — | — | — | MISSING | P2 | A | |
| Primary Keys (incl. composite) | INSPECT | INSPECT | PARTIAL (inspect + DDL re-emit) | WIRED (Structure/Constraints) | n/a | composite tests | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| Foreign Keys (incl. composite) | INSPECT | INSPECT | PARTIAL (inspect + DDL re-emit; no add/drop service) | WIRED (Relations tab) | n/a | composite tests | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| Unique | PARTIAL (single-col derived; multi-col as Index) | PARTIAL (same) | PARTIAL | WIRED (Constraints/Indexes) | n/a | integration | PENDING | PARTIAL | P2 | A | No `contype='u'` catalog query on PG |
| Check constraints | INSPECT (`pg_constraint`) | PARTIAL (parsed from table SQL) | PARTIAL (inspect + re-emit) | WIRED (Constraints) | n/a | parser tests | PENDING | RUNTIME_VERIFY | P1 | v0.1 | LIM-011 disposition still pending (#68) |
| Indexes | INSPECT (incl. functional/GIN/GiST w/ method+definition) | PARTIAL (btree hardcoded; expr/partial lose structure) | PARTIAL (builders + generic exec; no typed CRUD) | WIRED (Indexes tab) | policy-checked | parser tests | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| Triggers | INSPECT (incl. enabled flag + funcdef) | PARTIAL (header parser; always enabled) | PARTIAL (create/drop builders; no enable/disable) | WIRED read-only (Definition) | policy-checked | integration | PENDING (PG live enable/disable) | PARTIAL | P1 | A | |
| Functions | INSPECT (`pg_proc f+p`) | n/a (always `[]`) | PARTIAL (inspect only; no create/alter/drop/call) | WIRED read-only (Definition) | n/a | integration | PENDING | PARTIAL | P1 | B | Capability-gated in Explorer (`explorer_view.rs:639-642`) |
| Procedures | PARTIAL (bundled as `routine_type`) | n/a | PARTIAL (no CALL helper; classifier treats CALL as Destructive) | MISSING (bundled row only) | fail-closed classify | — | — | PARTIAL | P2 | B | No `Procedure` struct |
| Sequences | MISSING (no catalog query; `nextval` heuristic only) | n/a | MISSING (flag `sequences:true` with zero implementation) | MISSING | — | — | — | MISSING | P2 | A | Flag-without-code case (see §16) |
| Types / Enums / Domains | MISSING (only `format_type()` strings) | n/a | MISSING (flag `enum_types:true`, zero implementation) | MISSING | — | — | — | MISSING | P2 | A | Arrays arrive as `Text`, no element parsing |
| Users | BACKEND_ONLY (PG `pg_roles`) | n/a (gated `Unsupported`) | DONE PG (`user_service.rs` + `PostgresUserManager`) | MISSING (hidden per LIM-005) | writable-gate + allowlist quoting | service tests | PENDING | BACKEND_ONLY | P2 | E | No native `UiCommand` |
| Roles / Memberships / Grants | BACKEND_ONLY PG (role + table grants) | n/a | PARTIAL (no column/db grants, no password rotation, no RLS) | MISSING | privilege allowlist (`user_manager.rs:24-31`) | service tests | PENDING | BACKEND_ONLY | P2 | E | |

---

## 3. CRUD capability matrix

`Create/Alter/Drop` = typed service or builder+UI. `Generic DDL` = hand-built SQL via `ddl_builder.rs` + `execute_ddl` (policy-checked, NOT capability-checked — known gap). `View DDL` = reconstructed dialect-aware DDL.

### 3a. PostgreSQL

| Object | Inspect | Create | Alter/Edit | Drop | View DDL | Generate DDL | Refresh | Dependencies | Status |
|---|---|---|---|---|---|---|---|---|---|
| Table | YES | GENERIC | GENERIC | GENERIC | YES | YES | YES | YES | PARTIAL |
| Column | YES | GENERIC (add) | NO (no alter-type/rename builder) | GENERIC | in-table | PARTIAL | via table | NO | PARTIAL |
| View | YES | BUILDER | NO | BUILDER | YES (stored) | YES | YES | YES (as dependent) | PARTIAL |
| Materialized View | NO | NO | NO | NO | NO | NO | NO | NO | MISSING |
| Index | YES | BUILDER | NO (drop+recreate) | BUILDER | YES (in-table) | YES | YES | NO | PARTIAL |
| FK constraint | YES | NO | NO | NO | YES (in-table) | EMIT only | YES | YES bidir | PARTIAL |
| PK/Unique/Check | YES | NO | NO | NO | YES (in-table) | EMIT only | YES | FK only | PARTIAL |
| Trigger | YES | BUILDER | NO (no enable/disable) | BUILDER | YES | YES | YES | attached | PARTIAL |
| Function | YES | NO | NO | NO | PREFIX only | NO | YES | substring | PARTIAL |
| Procedure | BUNDLED | NO | NO | NO | NO | NO | bundled | NO | MISSING |
| Sequence | HEURISTIC | NO | NO | NO | NO | NO | NO | synthetic | MISSING |
| Type/Enum/Domain | NO | NO | NO | NO | NO | NO | NO | NO | MISSING |
| User/Role/Grant | YES | YES | YES (grant/revoke) | YES | NO | NO | NO | NO | BACKEND_ONLY |
| Schema | YES | NO | RENAME (PG-only bypass) | NO | NO | NO | YES | NO | PARTIAL |
| Database | NO | NO | NO | NO | NO | NO | NO | NO | MISSING |

### 3b. SQLite

| Object | Inspect | Create | Alter/Edit | Drop | View DDL | Generate DDL | Refresh | Dependencies | Status |
|---|---|---|---|---|---|---|---|---|---|
| Table | YES | GENERIC | GENERIC | GENERIC | YES | YES | YES | YES | PARTIAL |
| Column | YES | GENERIC (add) | NO (cannot alter type) | GENERIC | in-table | PARTIAL | via table | NO | PARTIAL |
| View | YES | BUILDER | NO | BUILDER | YES (stored) | YES | YES | YES | PARTIAL |
| Index | YES (btree) | BUILDER | NO | BUILDER | YES (in-table) | YES | YES | NO | PARTIAL |
| FK constraint | YES | NO | NO | NO | YES (inline) | EMIT only | YES | YES bidir | PARTIAL |
| PK/Unique/Check | YES | NO | NO | NO | YES (in-table) | EMIT only | YES | FK only | PARTIAL |
| Trigger | YES | BUILDER | n/a (no disable) | BUILDER | YES | YES | YES | attached | PARTIAL |
| Function/Procedure/Sequence/Type/Enum/User | n/a | — | — | — | — | — | — | — | MISSING (by design; capability-gated) |

Key gap: `execute_ddl` enforces `readonly/allow_ddl/allow_destructive` + single-statement, but never consults `DatabaseCapabilities.schema.*` — capability flags are advisory only (except user manager + cancel + backup + functions-visibility).

---

## 4. Query Workspace

Sources: `crates/ui/src/query_view.rs`, `crates/ui/src/editor/*`, `crates/ui/src/query/*`, `crates/runtime/src/api.rs:261-315`.

| Feature | PostgreSQL | SQLite | Backend | Native UI | Safety | Tests | Runtime Evidence | Status | Priority | Target Phase | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Native editor + syntax highlight | yes | yes | n/a (UI) | WIRED (dialect-aware tokenizer) | n/a | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| Multi query tabs + persistence + dirty guard | yes | yes | n/a | WIRED (`query-documents` key) | close-guard | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | Tab-close does NOT cancel running query (gap) |
| Execution (selection → statement → all) | DONE | DONE | DONE (`execute`/`execute_multi`, atomic routing) | WIRED (Run/Run selection/Run all) | policy per statement | integration | PENDING native | RUNTIME_VERIFY | P0 | v0.1 | |
| Cancellation | PORT-ONLY (`Unsupported`) | DONE (VM interrupt) | PARTIAL | WIRED (Stop/Esc, capability-gated) | idempotent cancel | cancel tests | PENDING | PARTIAL | P1 | v0.1/D | **Inverted vs release matrix** (see §16) |
| History | DONE | DONE | DONE (repo + service) | WIRED local dual (20 + 500 persisted) | n/a | UI | PENDING | RUNTIME_VERIFY | P1 | v0.1 | Server `QueryApi::history` unwired |
| Saved queries + folders | DONE | DONE | DONE (7 repo methods) | WIRED (grouped, open/copy/rename/delete) | connection-required save | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | Rename UX awkward (reuses folder input) |
| Drafts (dirty/snapshot) | n/a | n/a | n/a | WIRED | n/a | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| Snippets | n/a | n/a | MISSING (no domain) | MINIMAL (2 hardcoded inserts) | n/a | — | — | MISSING | P2 | G | No library, no persistence |
| Scratch SQL | n/a | n/a | MISSING | MISSING (unsaved doc is closest) | n/a | — | — | MISSING | P2 | G | |
| Formatting | n/a | n/a | n/a | WIRED minimal (clause-break, literal-safe) | never rewrites literals | UI | PENDING | PARTIAL | P2 | G | Not `pg_format`-class |
| Completion (keyword/table/column/CTE) | yes | yes | n/a | WIRED (manual + auto popup) | n/a | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| Diagnostics (parser + delimiter + DB errors) | yes | yes | PARTIAL (server error mapping in UI) | WIRED | n/a | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| Multi-result rendering | yes | yes | DONE (`MultiQueryResult`) | WIRED (Result 1..N + Messages) | n/a | integration + UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| Explain / query plan | DONE (`FORMAT JSON`) | DONE (text) | DONE | PARTIAL (raw monospace; visual `ExplainPlanTree` demo-only) | single-statement | integration | PENDING | PARTIAL | P1 | v0.1/G | `components/explain.rs` never fed real plan |
| Export result | DONE x3 (service) | DONE x3 | DONE (CSV/JSON/XLSX + BIGINT guards) | PARTIAL (local CSV/TSV writer, no quoting) | readonly policy | service tests | PENDING | BACKEND_ONLY | P1 | C | No native trigger for backend exporters |
| Query parameters | PLUMBING | PLUMBING | PARTIAL (builders parameterized; runtime passes `&[]`) | MISSING (no dialog) | n/a | builder tests | — | MISSING | P2 | G | Binary-cell hint references params that don't exist in UI |
| Keyboard workflow | n/a | n/a | n/a | PARTIAL (run/save/palette/find wired; `Ctrl+N` advertised but unwired) | n/a | — | — | PARTIAL | P2 | G | No keybinding editor; shortcuts hardcoded |

Gap-to-IDE summary: editor core is competitive; missing are params UI, snippet library, scratch, visual plan, real export wiring, keybinding customization.

---

## 5. Data Workspace

Sources: `crates/ui/src/table_editor_view.rs`, `crates/ui/src/result_grid*.rs`, `crates/ui/src/change_set.rs`, `crates/core/src/application/table_data_service.rs`.

| Feature | PostgreSQL | SQLite | Backend | Native UI | Safety | Tests | Runtime Evidence | Status | Priority | Target Phase | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Pagination | DONE | DONE | DONE | WIRED table (100/page); query results unbounded | n/a | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | Query results have no server pagination |
| Virtualization | n/a | n/a | n/a | PARTIAL (index projection + culling; no row recycling) | n/a | benches | bench-only | PARTIAL | P2 | Later | 1M-row projection ~3.15ms (bench) |
| Typed filters (server) | DONE | DONE | DONE (11 operators) | WIRED table; contains-only in query grid | parameterized | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| Multi-sort (server) | DONE | DONE | DONE | WIRED table; single-col query grid | blocked while staged | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| Selection + keyboard nav | n/a | n/a | n/a | WIRED (cell/row, arrows/Home/End) | n/a | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| Copy | n/a | n/a | n/a | PARTIAL (raw cell/row/CSV/JSON/SQL; staged values NOT copied) | n/a | UI | PENDING | PARTIAL | P2 | G | Staged-copy gap |
| Inline edit (staged, Enter-to-stage) | DONE | DONE | DONE (parameterized, 1-row guard) | WIRED | NOT NULL/binary guards | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | No auto-commit (by design) |
| Insert / Update / Delete | DONE | DONE | DONE (atomic `apply_mutations_detailed`) | WIRED staged + dialogs | 0-row→Conflict; >1→violation | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | Complex-type widgets incomplete (LIM-003) |
| Composite PK | DONE | DONE | DONE | WIRED (identity cache + reload) | targeted reload | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| Conflict handling (3-way Original/Local/DB) | DONE | DONE | DONE | WIRED (dialog + per-cell tint + retry) | rollback preserved | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| Pending-changes review + navigation guards | n/a | n/a | n/a | WIRED | guards block nav | UI | PENDING | RUNTIME_VERIFY | P0 | v0.1 | |
| Commit / rollback | DONE | DONE | DONE (commit) | PARTIAL (commit wired; rollback = local discard) | `RolledBack` never misreported | integration | PENDING | RUNTIME_VERIFY | P0 | v0.1 | No explicit BEGIN/COMMIT UI, no partial commit |
| JSON viewer | n/a | n/a | n/a | PARTIAL (expanded editor + pretty copy; no tree) | n/a | — | — | PARTIAL | P2 | G | |
| Binary/BLOB handling | DONE (decode) | DONE (decode) | PARTIAL | PLACEHOLDER ("binary editor unavailable", read-only) | fail-closed edit | — | — | PARTIAL | P2 | C/G | |
| Null / Default | DONE | DONE | DONE (null) | WIRED null; default = placeholder text only | NOT NULL guard | UI | PENDING | PARTIAL | P2 | G | No DEFAULT apply UI |
| Large-result behavior | bounded | bounded | bounded (`max_rows`) | PARTIAL (table paged; query grid full in-memory) | n/a | benches | — | PARTIAL | P2 | Later | No cap warning / progressive fetch |

---

## 6. ER Diagram (inventory only — implementation closed)

Sources: `crates/ui/src/diagram/*`, `crates/ui/src/diagram_view.rs`.

| Feature | PostgreSQL | SQLite | Backend | Native UI | Safety | Tests | Runtime Evidence | Status | Priority | Target Phase | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Graph model (`ErGraph`) | yes | yes | n/a | INVENTORY | n/a | 98 diagram | PASS (source/auto) | RUNTIME_VERIFY | P0 | v0.1 | Precomputed edge bbox |
| Async layout worker (coalescing, degraded mode) | n/a | n/a | n/a | INVENTORY | degraded-mode | worker tests | PASS (source/auto) | RUNTIME_VERIFY | P0 | v0.1 | `MAX_COALESCE_DRAIN 64` |
| Spatial index (bounded span 32) | n/a | n/a | n/a | INVENTORY | n/a | spatial tests | PASS (source/auto) | RUNTIME_VERIFY | P0 | v0.1 | |
| LOD (3-tier) | n/a | n/a | n/a | INVENTORY | n/a | LOD tests | PASS (source/auto) | RUNTIME_VERIFY | P0 | v0.1 | Compact<0.75 / Standard<1.15 / Detailed |
| Search + large-schema gate (>200) | yes | yes | n/a | INVENTORY | n/a | transition tests | PASS SQLite 202-table | RUNTIME_VERIFY | P0 | v0.1 | PG live pending |
| Neighborhood BFS (depth 1-2, cap 100) | yes | yes | n/a | INVENTORY | n/a | BFS tests | PASS (source/auto) | RUNTIME_VERIFY | P0 | v0.1 | |
| Fit / zoom / pan | n/a | n/a | n/a | INVENTORY | n/a | — | PASS screenshots | RUNTIME_VERIFY | P0 | v0.1 | 0.5–2.0 zoom |
| Composite FK edges | yes | yes | n/a | INVENTORY (`[a,b]→[c,d]` labels; anchors use first col) | n/a | edge tests | PASS (source/auto) | RUNTIME_VERIFY | P0 | v0.1 | |
| Position persistence | — | — | — | MISSING (rebuilt grid each load) | n/a | — | — | MISSING | P3 | Later | No `dbpro.native.diagram*` keys |
| Design / edit mode | — | — | — | DOES NOT EXIST | — | — | — | DEFERRED | P3 | Later | Confirmed absent; keep deferred |

---

## 7. Agent / AI

Tools (`crates/runtime/src/agent.rs:589-600`, exec `crates/runtime/src/agent_executor.rs`). Modes Ask/Edit/Agent (`agent_workflow.rs:519-542`). All DB tools reuse canonical `SchemaService`/`QueryService` — no parallel implementation (verified).

| Capability | Tool support | Missing tool | Missing context | Missing confirmation/safety | Missing UI | Status | Priority | Target Phase | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Ask Database | `run_query` + inspects | — | — | — (gated) | — | RUNTIME_VERIFY | P0 | v0.1 | Bounded sampling (20 rows/50 cols) |
| Generate SQL | `patch_query` + draft | — | — | preview diff wired | — | RUNTIME_VERIFY | P0 | v0.1 | UTF-8 + version-checked patches |
| Explain SQL | `explain_query` (no ANALYZE) | — | — | — | visual plan | PARTIAL | P1 | G | Truncated 4000 chars |
| Fix SQL | `patch_query` + Investigate-error action | — | — | preview wired | — | RUNTIME_VERIFY | P0 | v0.1 | |
| Optimize SQL | prompt-level `Optimize` action only | dedicated tool | plan + stats context | — | — | PARTIAL | P2 | H | No cost/stat input |
| Explain Query Plan | `explain_query` | — | — | — | visual plan UI | PARTIAL | P1 | G | Raw text only |
| Find Schema | `inspect_schema/table/columns/foreign_keys` (caps 12/24/12) | cross-schema search | — | — | — | RUNTIME_VERIFY | P0 | v0.1 | No column-name global search tool |
| Generate Migration | NONE | `generate_migration` | diff context (diff is row-count/table-list narrow) | confirmation design | preview UI | MISSING | P2 | F/H | Blocked on diff depth |
| Compare Schemas | NONE | `compare_schema` | — | — | diff UI | MISSING | P2 | F/H | Backend diff narrow (tables+cols+indexes) |
| Analyze Slow Query | NONE | `analyze_slow_query` | pg_stat/sessions (all missing) | — | monitoring UI | MISSING | P3 | D/H | Blocked on monitoring |
| Analyze Data | NONE | `inspect_sampled_data` | sampling policy | read-only gate exists | sample viewer | MISSING | P3 | H | `inspect_query_result` is closest (in-memory only) |
| Sessions/locks inspection | NONE | future | monitoring backend | — | — | MISSING | P3 | D/H | Listed as future in parity note |
| Index suggestion | NONE | future | plan + stats | explicit confirmation required | — | MISSING | P3 | H | No autonomous DDL ever |

AI rules (binding): no parallel DB implementation; destructive ops always explicit confirmation with re-classification at execution (`agent_executor.rs:368-390` anti-TOCTOU).

---

## 8. Monitoring / Administration

No `AdminService`, no `pg_stat*` queries anywhere in `crates/`. Only adjacent PG extras (`object_dependencies`, `partitions`, `tablespaces` via inherent connector methods + `PostgresApi`, Tauri-exposed, no native UI).

| Feature | PostgreSQL | SQLite | Backend | Native UI | Safety | Tests | Runtime Evidence | Status | Priority | Target Phase | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Active sessions | MISSING | n/a | MISSING (flag `server_sessions:true`, no impl) | MISSING | — | — | — | MISSING | P1 | D | Flag-without-code |
| Running queries | MISSING | n/a | MISSING | MISSING | — | — | — | MISSING | P1 | D | |
| Locks | MISSING | n/a | MISSING | MISSING | — | — | — | MISSING | P2 | D | |
| Transactions view | MISSING | n/a | MISSING (user txns exist, no listing) | MISSING | — | — | — | MISSING | P2 | D | |
| Cancel query (admin) | PORT-ONLY | ADAPTER, no admin UI | PARTIAL | Stop-button only | idempotent | cancel tests | PENDING | PARTIAL | P1 | D | See §16 inversion |
| Terminate session | MISSING | n/a | MISSING | MISSING | — | — | — | MISSING | P2 | D | |
| DB size | MISSING | MISSING (backup reports bytes only) | MISSING | MISSING | — | — | — | MISSING | P2 | D | PG has `reltuples` estimate only |
| Table size | MISSING (`reltuples` only) | MISSING (`None`) | MISSING | MISSING | — | — | — | MISSING | P2 | D | |
| Statistics | MISSING | MISSING | MISSING | MISSING | — | — | — | MISSING | P3 | D | |
| Vacuum / Analyze (user op) | MISSING | MISSING (backup-internal `VACUUM INTO` only) | MISSING | MISSING | — | — | — | MISSING | P2 | D | Needs destructive-adjacent confirmation |
| Server information | MISSING (no `version()`) | n/a | MISSING | MISSING | — | — | — | MISSING | P3 | D | |

---

## 9. Data Transfer

| Feature | PostgreSQL | SQLite | Backend | Native UI | Safety | Tests | Runtime Evidence | Status | Priority | Target Phase | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Import CSV | MISSING | MISSING | MISSING (no reader deps; LIM-012) | MISSING | — | — | — | MISSING | P1 | C | |
| Import Excel | MISSING | MISSING | MISSING | MISSING | — | — | — | MISSING | P2 | C | |
| Import JSON | MISSING | MISSING | MISSING | MISSING | — | — | — | MISSING | P2 | C | |
| Export CSV | DONE | DONE | DONE (`csv` crate) | UNWIRED (native has separate local writer) | readonly policy | service tests | PENDING | BACKEND_ONLY | P1 | C | Native CSV/TSV writer lacks quoting |
| Export Excel | DONE | DONE | DONE (`rust_xlsxwriter` + BIGINT/Decimal guards) | MISSING | readonly policy | service tests | PENDING | BACKEND_ONLY | P1 | C | Lossless `2^53` handling is a strength |
| Export JSON | DONE | DONE | DONE (dup-col + non-finite guards) | MISSING | readonly policy | service tests | PENDING | BACKEND_ONLY | P2 | C | |
| Export SQL INSERT / COPY | MISSING | MISSING | MISSING (`export_sql` absent) | MISSING | — | — | — | MISSING | P2 | C | |
| Database-to-database transfer | MISSING | MISSING | MISSING (diff is count-only) | MISSING | — | — | — | MISSING | P3 | Later | |
| Preview / column+type mapping / conflict strategy / progress / cancellation / streaming | — | — | MISSING (export materializes bounded `QueryResult`) | — | — | — | — | MISSING | P1 | C | Architecture must be designed in Phase C |

---

## 10. Backup / Restore

| Feature | PostgreSQL | SQLite | Backend | Native UI | Safety | Tests | Runtime Evidence | Status | Priority | Target Phase | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Backup | DONE (PATH `pg_dump`, SSH-aware) | DONE (`VACUUM INTO`, atomic publish) | DONE (`backup_service.rs` + engines) | WIRED minimal (Settings path-pick + run) | readonly-restore block; no-overwrite publish | engine tests | PENDING live | RUNTIME_VERIFY | P1 | v0.1 | `pg_dump` NOT bundled (LIM-015); SQLite is VACUUM INTO, not file-copy |
| Restore | DONE (`psql`/`pg_restore`) | DONE (staged + `quick_check`) | DONE | WIRED minimal (confirm-overwrite dialog) | overwrite confirmation; active-DB block (SQLite) | engine tests | PENDING live | RUNTIME_VERIFY | P1 | v0.1 | No progress bar, no schedule, no format choice (hardcoded) |

---

## 11. Security (Users / Roles / Grants)

Backend PG-only (`user_service.rs`, `PostgresUserManager`); SQLite capability-gated `Unsupported`. No native UI at all.

| Feature | PostgreSQL | SQLite | Backend | Native UI | Safety | Tests | Runtime Evidence | Status | Priority | Target Phase | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Users list | DONE | gated | DONE | MISSING | — | service tests | PENDING | BACKEND_ONLY | P2 | E | |
| Roles create/drop | DONE | gated | DONE | MISSING | writable-gate | service tests | PENDING | BACKEND_ONLY | P2 | E | |
| Memberships | MISSING | gated | MISSING | MISSING | — | — | — | MISSING | P2 | E | |
| Table grants | DONE (8-priv allowlist) | gated | DONE | MISSING | identifier quoting | service tests | PENDING | BACKEND_ONLY | P2 | E | |
| Schema privileges | MISSING | gated | MISSING | MISSING | — | — | — | MISSING | P3 | E | |
| Column/DB grants, password rotation, RLS | MISSING | gated | MISSING | MISSING | — | — | — | MISSING | P3 | Later | |
| Role DDL view | MISSING | gated | MISSING | MISSING | — | — | — | MISSING | P3 | E | |

---

## 12. Compare / Migration

| Feature | PostgreSQL | SQLite | Backend | Native UI | Safety | Tests | Runtime Evidence | Status | Priority | Target Phase | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Schema Compare | PARTIAL (tables+columns+types+indexes only) | PARTIAL | PARTIAL (`schema_diff.rs`, driver-agnostic) | MISSING | read-only | diff tests | PENDING | BACKEND_ONLY | P2 | F | No views/triggers/functions/constraints |
| DDL Diff (generate ALTERs) | MISSING | MISSING | MISSING | MISSING | — | — | — | MISSING | P2 | F | |
| Migration Preview | MISSING | MISSING | MISSING (`meta/migration.rs` is app-store only) | MISSING | — | — | — | MISSING | P2 | F | Do not confuse with app meta migrations |
| Migration SQL generation | MISSING | MISSING | MISSING | MISSING | — | — | — | MISSING | P2 | F | |
| Migration Apply (safe) | MISSING | MISSING | MISSING | MISSING | — | — | — | MISSING | P3 | F | Must reuse `execute_ddl_batch` + confirmation |
| Data Compare | COUNT-ONLY | COUNT-ONLY | PARTIAL (`data_diff.rs`) | MISSING | read-only | diff tests | PENDING | BACKEND_ONLY | P3 | F | Capability flag overstates coverage |

---

## 13. Productivity

| Feature | Backend | Native UI | Status | Priority | Target Phase | Notes |
|---|---|---|---|---|---|---|
| Global Search | MISSING | MISSING (palette covers tables ≤100 + static cmds) | MISSING | P2 (P1 search-in-palette) | G | Needs unified index: connection/schema/table/column/view/function/query/command/agent-action |
| Command Palette | n/a | WIRED (`palette_view.rs`, QuickOpen + Commands) | RUNTIME_VERIFY | P0 | v0.1 | Not searchable: saved-query SQL, history SQL, columns, settings |
| Favorites (connections/objects/queries) | PARTIAL (tags/group in domain, dropped in UI) | MISSING | MISSING | P2 | G | Seed uses tags UI can't show |
| Recent | n/a | PARTIAL (reverse-chron history only) | PARTIAL | P2 | G | |
| Pinned Objects | MISSING | MISSING | MISSING | P3 | G | |
| Saved Queries (+folders) | DONE | WIRED | RUNTIME_VERIFY | P0 | v0.1 | No tags, no move-between-folders |
| Query History (filter/search) | DONE | WIRED local | RUNTIME_VERIFY | P1 | v0.1 | Server history unwired; 500-cap local |
| Snippets library | MISSING | 2 hardcoded | MISSING | P2 | G | |
| Scratch SQL | MISSING | MISSING | MISSING | P2 | G | |
| Keyboard navigation (grid/tabs/app) | n/a | WIRED (grid + tabs) | RUNTIME_VERIFY | P0 | v0.1 | Shortcuts hardcoded; cheatsheet is gallery demo |
| Keybinding customization | MISSING | MISSING | MISSING | P3 | Later | |

---

## 14. Settings

| Area | Backend | Native UI | Status | Priority | Target Phase | Notes |
|---|---|---|---|---|---|---|
| Connections | DONE | WIRED (dialog) | RUNTIME_VERIFY | P0 | v0.1 | Import/export profiles missing |
| Editor (font/completion/snippets/prediction) | n/a (local flags) | PARTIAL (per-query `···` menu; only prediction persisted) | PARTIAL | P2 | G | Needs a real Editor page + persistence |
| Appearance (theme/reduce-motion) | n/a | WIRED + persisted (`DbProTheme`, version reset) | RUNTIME_VERIFY | P0 | v0.1 | Token contract owned by theme |
| AI Providers (key, auto-run) | keyring-backed | WIRED in Agent panel header | RUNTIME_VERIFY | P0 | v0.1 | Provider list is text; multi-provider UX minimal |
| Keybindings | MISSING | MISSING | MISSING | P3 | Later | |
| Database/provider settings (timeouts, max rows, SSL default) | PARTIAL (`RunConfig` repo exists, unwired) | MISSING | BACKEND_ONLY | P2 | G | `run_config_repo` has no native UI |
| Advanced / Privacy-security (telemetry, secret mgmt) | PARTIAL (secret store) | MISSING | MISSING | P2 | G | Keyring fallback is dev-only (#142) |

Settings currently live in: sidebar Settings page (appearance + backup paths), per-query `···` menu, Agent panel header, connection dialog, eframe storage keys (`dbpro.native.*`). Consolidation is Phase G work.

---

## 15. Connection management

| Feature | PostgreSQL | SQLite | Backend | Native UI | Status | Priority | Target Phase | Notes |
|---|---|---|---|---|---|---|---|---|
| Create / Edit / Delete (+confirm) | YES | YES | DONE | WIRED | RUNTIME_VERIFY | P0 | v0.1 | |
| Duplicate (password/SSL/SSH reset by design) | YES | YES | DONE | WIRED | RUNTIME_VERIFY | P1 | v0.1 | Document the reset; don't silently change |
| Test (with stale-retest nudge) | YES | YES | DONE | WIRED | RUNTIME_VERIFY | P0 | v0.1 | |
| Connect / Disconnect / Reconnect / Refresh schema | YES | YES | DONE | WIRED | RUNTIME_VERIFY | P0 | v0.1 | Disconnect is local-only clear |
| Startup reconnect + workspace recovery | YES | YES | DONE | WIRED | RUNTIME_VERIFY | P0 | v0.1 | RC1 wave; evidence pending |
| Folders / Tags / Favorites / Color | domain YES | domain YES | DONE (dropped at UI boundary) | MISSING | BACKEND_ONLY | P2 | G | `UiConnectionSummary` drops them |
| Import/export connection profiles | — | — | MISSING | MISSING | MISSING | P3 | Later | |
| SSH tunnel (external `ssh`, `sshpass` secrecy) | EXISTS unqualified | n/a | DONE | WIRED (PG dialog) | RUNTIME_VERIFY | P1 | v0.1/D | E2E pending; PATH dependency |
| SSL/TLS (mode-only, rustls) | EXISTS unqualified | n/a | DONE | WIRED (segmented control) | RUNTIME_VERIFY | P1 | v0.1 | No CA/cert fields; all tests use Disable |

---

## 16. Evidence conflicts found (code vs docs — decisions required)

1. **Query-cancel inversion.** Code truth: `capabilities.rs:128-132,183-186` + `postgres/connector.rs:199-203` (`Unsupported`) + `sqlite/connector.rs:117`/`actor.rs:106` (interrupt) ⇒ cancel works on **SQLite, not PG**. Release docs say the opposite (`provider-capability-matrix.md:75` PG SUPPORTED/SQLite NOT SUPPORTED; `LIM-014`). One side must be corrected before v0.1 notes ship. UI gates Stop on the (code) flag, so behavior follows code.
2. **Flags without implementation.** `sequences:true`, `enum_types:true` (PG), `server_sessions:true`, `partitions/tablespaces/object_dependencies:true`, `schema_diff/data_diff:true` advertise more than exists (no catalog/service/UI or narrow coverage). Either implement (roadmap) or downgrade flags + matrix wording.
3. **Backup mechanics wording.** Release matrix says SQLite "file copy/VACUUM"; code is `VACUUM INTO` + atomic hard-link publish (`sqlite_backup.rs:84-103`) — not a file copy. Minor doc fix.
4. **`frontend/` dir still present at repo root** alongside `_archive/frontend/` — audit confirms product UI is `crates/ui`; root `frontend/` + demo HTML files are legacy artifacts that confuse inventory. Recommend removal or explicit marker (separate cleanup task, not this audit).
5. **`Ctrl+N` advertised, unwired** (`navigation_view.rs:584`, `workspace_view.rs:533-536` vs `events.rs:1102-1160`). Either wire or drop the hint.
6. **Tab-close doesn't cancel** a running query (`app.rs:1018-1034` cancels prediction/agent only). Decide: cancel-on-close or explicit behavior.
7. **Copy ignores staged values** (`result_grid_view.rs` copy helpers read `result.rows`, not `ChangeSet`). Decide before claiming copy-correctness.
8. **Legacy `tauri-app` blocks `cargo check --workspace`** (dirty `commands/query.rs:50,114` per core-safety VERIFICATION). Any "gates green" claim must scope the legacy host out or fix it; removal is already declared, execution pending.

---

## 17. Aggregate estimate

~150 tracked cells above. Rough count: DONE/RUNTIME_VERIFY ≈ 45%, PARTIAL/BACKEND_ONLY/INSPECT_ONLY ≈ 25%, MISSING/DEFERRED ≈ 30%.

- **v0.1-shippable core (engine): ~85%** — query, data editing, explorer introspection, agent workflow, ER, safety, connections.
- **DBeaver-class breadth: ~35%** — object CRUD workbench, functions/procedures exec, monitoring, transfer, users UI, compare/migration, productivity layer are the missing 65%.

Strongest areas (§7 parity checkpoint concurs): safety architecture, staged data editor with 3-way conflicts, query intelligence (completion/diagnostics/multi-result), typed agent tools reusing canonical services, ER large-schema architecture, provider capability gating.

Five biggest gaps: (1) typed object-CRUD workbench (builders exist, UI/services don't); (2) monitoring/admin (zero); (3) import + export wiring (engine done, UI missing); (4) functions/procedures/sequences/types (inspect-only or missing); (5) productivity (global search, snippets, favorites, keybindings) + users/roles UI + compare/migration depth.
