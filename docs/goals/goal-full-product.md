# DB Pro — Master Product Goal (Full Product Roadmap)

- Doc ID: `GOAL-FULL-PRODUCT`
- Status: **Authoritative for long-term product direction.** Phase execution goals and
  `docs/notes/PRODUCT_ROADMAP.md` derive their scope from this document; where a note,
  release doc, or plan contradicts this file, this file wins until it is amended.
- Date: 2026-09-14
- Baseline: `main@b2cc33e` (post `docs(release): define v0.1 closure queue from product audit`)
- Method: direct source inspection of `crates/{core,infrastructure,runtime,ui,native-app,tauri-app}`
  plus `docs/release/*`, `docs/architecture/*`, `docs/notes/*`, `docs/plans/*`. No code was
  changed while producing this document. Every "exists / missing" claim below is tied to a
  file path; claims that could not be settled from source are labeled **unverified**.
- Companion documents:
  - `docs/goals/goal-phase-a-object-crud.md` … `goal-phase-h-ai.md` (execution goals)
  - `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` (cell-level capability audit)
  - `docs/notes/PRODUCT_ROADMAP.md` (adjusted phase/release plan)
  - `docs/notes/V0_1_CLOSURE_PLAN.md` (v0.1 release closure queue — do not mix with this roadmap)
  - `docs/plans/FEATURE_LIFECYCLE.md` (mandatory lifecycle + evidence rules)
- Scope rule for v0.1: **this document does not change the v0.1 release queue.** Per
  `docs/notes/V0_1_CLOSURE_PLAN.md` §1, no Phase A–L feature work is admitted into v0.1.
  This goal starts at v0.2.

---

## 1. Product Vision

### 1.1 Vision statement

> **DBeaver-class database capability coverage + Codex/VS Code-class native interaction
> quality + an AI-native, safety-first workflow — delivered as a fast native Rust
> application.**

DB Pro already has a competitive query/data/agent engine. The remaining work is not "more
editor": it is the rest of the database IDE — object lifecycle management, routines,
transfer, administration, security, compare/migration, and productivity — built on the
existing typed architecture instead of beside it.

### 1.2 Product formula

```text
DBeaver feature depth
+ Codex-quality native interaction          (already the ratified visual direction)
+ desktop IDE information density
+ contextual, typed database AI
= DB Pro
```

### 1.3 Product-level definition of done

DB Pro is "the full product" when a professional user can complete every one of these
workflows without leaving the application, on both supported providers where the provider
supports it, with previews and confirmations where the operation mutates data or schema:

1. Connect, organize, and safely target connections (including Production/Staging labels).
2. Explore every object type the provider supports, with accurate DDL and dependencies.
3. Create/alter/drop those objects through typed forms — never by hand-writing DDL.
4. Manage routines: browse, read source, edit, execute with typed arguments, read output.
5. Move data in and out: import (CSV/Excel/JSON) and export (CSV/TSV/Excel/JSON/INSERT/COPY)
   with mapping, preview, progress, cancellation, and error reporting on large files.
6. See what the database is doing (sessions, queries, locks, sizes) and act on it safely.
7. Administer PostgreSQL users, roles, memberships, and grants.
8. Compare two schemas, review an ordered migration plan, and apply it with a rollback plan.
9. Find anything (object, query, command) in one search surface, and reuse SQL via
   snippets/favorites/templates.
10. Ask the agent for the above, with every mutation requiring an explicit confirmation.

### 1.4 NON-GOALS (binding)

1. **No 1:1 DBeaver UI copy.** DBeaver is a *capability* reference. Visual language stays
   the ratified native Codex-aligned system (`crates/ui/src/theme.rs`, `tokens.rs`).
2. **No "one Activity Bar icon per object type."** Database objects live inside the
   Explorer tree (and, later, a Security page); the rail stays compact (§3).
3. **No bypass of the application/runtime safety layer.** No UI path may call a provider
   adapter directly, no DDL may be executed without policy + capability gating, no
   read-only connection may mutate.
4. **No autonomous destructive AI behavior.** The agent never executes a mutation or
   destructive operation without an explicit, re-classified, user-confirmed approval
   (existing anti-TOCTOU rule retained and extended to every new tool).
5. **No provider-specific logic in the UI.** PostgreSQL/SQLite behavior differences are
   expressed as capability flags and provider adapters, never as `if driver == Postgres`
   inside a view.
6. **No new database drivers** in v0.2–v0.4 (PostgreSQL + SQLite only, `LIM-002`).
7. **No WebView/React reintroduction** and no second state model
   (`docs/08-technology-decisions.md` §4, `docs/07-fe-architecture.md`).
8. **No ER design/edit mode, no job scheduler, no MCP, no telemetry-by-default** before
   their listed milestone, and never as a placeholder icon (§7 rule).
9. **No "fake support."** A capability that a provider cannot express must be hidden or
   disabled with a surfaced reason — never emulated by emitting unsupported SQL.
10. **No feature shipped without provider-independent evidence** (`docs/plans/FEATURE_LIFECYCLE.md`):
    PostgreSQL evidence never proves SQLite and vice versa.

### 1.5 Standing product rules (apply to every milestone)

- Reuse `domain → application → ports → provider adapters → runtime worker → native
  workbench`. New behavior arrives as new `UiCommand`/`RuntimeCommand`/`UiEvent`/
  `RuntimeEvent` pairs.
- Every new capability is capability-gated **in the backend**, not only in the UI.
- Every milestone PR references its plan directory (`PLAN/CHECKLIST/FINDINGS/VERIFICATION`)
  and records PostgreSQL and SQLite disposition separately.
- No new rail icon ships as a placeholder; icons appear with the milestone that wires them.

---

## 2. Current Baseline (Code-Verified)

### 2.1 Method and evidence discipline

Evidence levels are those of `docs/plans/FEATURE_LIFECYCLE.md`: **source evidence**,
**automated evidence**, **provider runtime evidence**, **UI runtime evidence**. This audit
produced **source evidence only**. No `cargo build/test` was executed and no live database
was contacted while writing this document, so nothing here is claimed as runtime evidence.
Where `docs/plans/STATUS.md` and source disagreed, source wins in this document and the
disagreement is listed in §2.5.

### 2.2 Subsystem classification

| Subsystem | Classification | Source evidence | Note |
|---|---|---|---|
| Connection management | **RUNTIME_VERIFY** (strong) | `core/application/connection_service.rs:63-434`, `runtime/src/api.rs:140-249`, `ui/src/connection_view.rs:282-927` | Create/edit/delete/duplicate/test/connect/disconnect/SSH/SSL wired; folders/tags dropped at DTO boundary |
| Database Explorer / introspection | **RUNTIME_VERIFY** | `postgres/introspect.rs:7-89` (10 catalog queries), `sqlite/introspect.rs:14-68`, `ui/src/explorer_view.rs:204-221` | Tables/views/columns/PK/FK/index/check/trigger/function read paths exist |
| Query Editor | **RUNTIME_VERIFY** | `ui/src/query_view.rs`, `ui/src/editor/*`, `ui/src/query/*`, `runtime/src/api.rs:252-402` | Execution, multi-result, completion, diagnostics, format, drafts |
| Table Data Editor | **RUNTIME_VERIFY** | `core/application/table_data_service.rs:30-300`, `ui/src/table_editor_view.rs`, `ui/src/change_set.rs` | Staged mutation, 3-way conflict, composite PK, commit |
| Schema introspection surface | **RUNTIME_VERIFY** | `ui/src/table_metadata_view.rs` (Structure/Indexes/Constraints/Relations/Dependencies), `ui/src/table_ddl_view.rs` | Read-only metadata; **all mutation is generic raw-DDL** |
| ER Diagram | **RUNTIME_VERIFY** | `ui/src/diagram/*` (98 tests), `ui/src/diagram_view.rs` | Large-schema architecture done; position persistence missing |
| Agent Workflow | **RUNTIME_VERIFY** | `runtime/src/agent.rs:565-634` (9 tools), `agent_executor.rs:52-251`, `agent_orchestrator.rs:338-547` | Typed tools reuse canonical services; confirmation + anti-TOCTOU verified in source |
| Core Safety (policy/classification) | **RUNTIME_VERIFY** | `core/src/domain/safety.rs:8-580` | 4-class classifier; read-only enforcement; **capability flags not enforced server-side** |
| Native UI foundation | **DONE** | `docs/plans/STATUS.md:41-43`, `crates/ui/src/theme.rs`, `tokens.rs`, `components/` (43 modules) | **[status correction 2026-09-14]** the Native Visual Redesign is implemented (Waves 1–14); what is not established is its runtime/visual verification — audited `EVIDENCE_GAP` (`docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md`). The earlier wording ("still in progress / not finished") contradicted `docs/notes/V0_1_CLOSURE_PLAN.md` §3 |
| Object mutation framework (typed) | **MISSING** | `ddl_builder.rs` exists but is dead code (`grep` shows only `application/mod.rs:4`); no `ObjectAction`/`MutationRequest` anywhere | Builders exist; no service, no preview, no UI |
| Object CRUD (views/indexes/FKs/triggers/sequences/types) | **MISSING / PARTIAL** | `ui` grep for `build_create_view`/`build_create_index`/`create_trigger` call sites → 0; `ddl_builder.rs:114-183` unused | Introspection strong, mutation absent |
| Routines (functions/procedures) | **PARTIAL** | `postgres/introspect.rs:762-779` (`pg_proc` f+p), `core/domain/schema.rs:119` (`Function`), `ui/src/explorer_folders.rs:169`, `ui/src/schema_object_view.rs` | Browse + read-only Definition; no Procedure type, no execute form, no CRUD |
| Monitoring / administration | **MISSING** | zero `pg_stat_activity` / `pg_locks` / `pg_stat_user_tables` matches under `crates/` | `ui/src/navigation_view.rs:616-621` renders a `COMING SOON` placeholder |
| Data import | **MISSING** | no reader dependency anywhere (`csv` is write-only via `export_service.rs`) | `LIM-012`; Transfers rail = placeholder (`navigation_view.rs:610-615`) |
| Data export | **BACKEND_ONLY** | `core/application/export_service.rs:65-249` (CSV/JSON/XLSX + BIGINT/2^53 & dup-column guards) | Native UI uses a *separate unquoted* local writer (`query_view.rs:1357-1389`) |
| Backup / restore | **RUNTIME_VERIFY** | `infrastructure/backup/pg_dump.rs:79-206`, `sqlite_backup.rs:60-157` | Live qualification pending; no progress/cancel |
| Users / roles / grants | **BACKEND_ONLY** | `core/application/user_service.rs:53-136`, `postgres/user_manager.rs:62-209` (8-privilege allowlist) | Zero native UI |
| Schema/data compare | **BACKEND_ONLY (narrow)** | `core/application/schema_diff.rs:11-156` (tables/columns/indexes only), `data_diff.rs:20-77` (count only) | No DDL diff, no migration generation |
| Productivity / search | **MISSING** | `Activity` enum has no Search variant (`app.rs:114-123`); palette covers tables ≤100 (`palette_view.rs`) | Favorites/recent/pinned/snippets/scratch/keys absent |
| Settings | **PARTIAL** | `ui/src/navigation_view.rs:940-1062` (Appearance + Backup/Restore only); `SettingsRepository` has **no consumer** | Editor/Keybindings/Providers/Advanced pages absent |
| Release / packaging / runtime qualification | **PARTIAL** | `docs/release/0.1.0-readiness.md`, `docs/notes/V0_1_CLOSURE_PLAN.md` §2 | Packaging build+package verified in CI (final run 34860902181); manual smoke NOT RUN (0/165) |
| Legacy `crates/tauri-app` host | **DEFERRED (removal)** | `docs/architecture/system-overview.md:50`; `crates/tauri-app/tauri.conf.json` points at a non-existent `_archive/frontend/dist` | Not reachable from the shipped binary |

### 2.3 What is genuinely strong (do not rebuild)

- **Safety architecture**: token/quote/dollar-quote-aware classifier
  (`domain/safety.rs:66-223`), policy validation (`:560-580`), multi-statement rejection
  (`application/sql_policy.rs:3-16`), read-only enforcement across query/DDL/export/table
  data, and agent execution-time re-classification (`agent_orchestrator.rs:171-193`).
- **Staged data editing with 3-way conflict resolution** (`ui/src/change_set.rs`,
  `table_editor_view.rs:2203`, `table_data_service.rs:216-300`) — composite PK identity,
  single-row invariant, ordered atomic transaction.
- **Query intelligence**: dialect-aware editor, completion, delimiter UX, structured
  diagnostics, multi-result rendering with explicit result kinds.
- **Typed agent tooling over canonical services**: all 9 tools call
  `SchemaApi`/`QueryApi` (`agent_executor.rs:70-350`) — the correct pattern to extend.
- **Provider capability model exists** (`domain/capabilities.rs:11-232`) with per-driver
  sets and unit tests, even though enforcement is client-side today.
- **Reconstructed dialect-aware DDL** (`schema_service.rs:264-620`) — reusable as the
  preview baseline for mutation work.
- **Export engine breadth with precision guards** (`export_service.rs:65-249`).
- **Native design system**: `theme.rs` + `tokens.rs` + 43 component modules with tests.
- **ER large-schema architecture**: spatial index, 3-tier LOD, BFS neighborhood, async
  coalescing layout worker.

### 2.4 What is missing or partial (the actual work)

1. **Typed object CRUD workbench** — builders exist (`ddl_builder.rs`), services/UI do not.
   Capability flags are **not consulted by the backend** (`execute_ddl` in
   `schema_service.rs:340` checks only readonly/multi-statement), so unsupported SQL can be
   emitted today by a generic DDL path.
2. **Routine execution and management** — no argument form, no `CALL` helper, no
   `Procedure` type (`domain/schema.rs` has only `Function` with a `routine_type` string),
   no create/replace/drop.
3. **Monitoring/administration** — zero backend; `server_sessions` is a flag whose only
   implementation is *user/role management*, not session listing.
4. **Import + export wiring** — import entirely absent; export engine unreachable from the
   native UI, which instead writes an unquoted local file.
5. **Security UI** — a complete PostgreSQL backend (`user_manager.rs`) with no surface.
6. **Compare/migration depth** — diff covers 3 object kinds; no DDL diff, no ordered
   migration, no apply path.
7. **Productivity** — no Search activity, no unified index, no snippets/scratch/favorites,
   no query parameters dialog, keybindings hardcoded.
8. **Settings** — only two of nine planned pages exist; the settings repository port is dead.
9. **Release/runtime qualification** — packaging, manual smoke, and every active plan still
   sit in `RUNTIME_VERIFY`.

### 2.5 Documentation conflicts found during this audit

Conflict IDs here are `C1`…`C15` and are **distinct from milestone IDs** (`C01`…`C05` are Phase C
milestones; a two-digit conflict ID such as `C10` is a conflict, not a milestone).
Disposition column: what this document does about it. Several fixes belong to files outside
this task's edit scope (`docs/release/*`, `docs/architecture/*`); those are recorded here as
explicit follow-up doc chores, not silently changed.

| # | Conflict | Evidence | Disposition |
|---|---|---|---|
| C1 | **Query-cancel inversion.** Release docs say PostgreSQL supports cancel and SQLite does not. Code says the opposite. | Docs: `docs/release/provider-capability-matrix.md:75-76,137`, `docs/release/known-limitations.md:206-218` (LIM-014). Code: `core/domain/capabilities.rs:129-132,185-187` (PG `cancel=false`, SQLite `true`), `infrastructure/postgres/connector.rs:199-203` (`Unsupported`), `sqlite/actor.rs:106-124` (interrupt + ack). | **Code is truth.** §9 and Phase D use code semantics. Follow-up chore: amend `provider-capability-matrix.md` and `LIM-014` (file not in this task's scope). |
| C2 | **Flag-without-code.** `sequences: true`, `enum_types: true` (PG) and release matrix "SUPPORTED + QUALIFIED" for both, with no catalog query. | Flags: `capabilities.rs:150-151`. Absence: no `pg_sequence`/`information_schema.sequences`/`pg_enum`/`pg_type` anywhere in `crates/infrastructure`. Release claim: `provider-capability-matrix.md:58-59`. | Recorded as **not implemented** here and in the capability matrix §16. A06/A07 either implement them or the flags are downgraded in the same PR. |
| C3 | **`server_sessions` means users, not sessions.** Flag implies session listing; implementation is `UserService` (`user_service.rs:53-136`). | `capabilities.rs:167`; no `pg_stat_activity` in repo. | §9 splits this into two capabilities: *role/user management* (exists) and *session monitoring* (missing, D01). |
| C4 | **`docs/architecture/security-boundaries.md` classifier table is wrong**: it puts DROP/TRUNCATE under **Ddl**. | Doc table "Statement Classifier". Code: `domain/safety.rs:94` maps DROP/TRUNCATE/CALL/DO → `Destructive`. | §10 restates the real four classes. Follow-up chore: fix the doc table. |
| C5 | **`allow_ddl` / `allow_destructive` are unreachable.** Security doc claims backend-enforced policy with these flags; no code path ever sets them `false`. | `domain/safety.rs:560-580` branches; grep shows only `read_only()`/`full_access()` constructors in `schema_service.rs:34-41`, `query_service.rs:124-135`, `table_data_service.rs:49-60`, `export_service.rs:48-52`. | §10.2 defines the **target** policy producer (operation class + environment + connection flags). Implementing it is part of A01. |
| C6 | **STATUS.md vs code on ER position persistence.** `STATUS.md:16` records "position persistence" merged; matrix says MISSING. | `docs/plans/STATUS.md:16`; `docs/notes/PRODUCT_CAPABILITY_MATRIX.md:186`; no `dbpro.native.diagram*` keys in `crates/ui/src/app.rs` storage list (audited). | Treated as **unverified/absent**; recorded as a documentation conflict for ER owner. Not a Phase A–L blocker. |
| C7 | **`docs/architecture/query-runtime.md:5-8` points the runtime registry at `crates/tauri-app/src/cancel.rs`.** The shipped path uses `crates/runtime/src/worker.rs` maps; the Tauri registry never calls DB-level cancel. | `worker.rs:367-388, 1312-1395, 1524-1553` vs `tauri-app/src/cancel.rs:25-225` (no `QueryApi::cancel` call under `crates/tauri-app`). | Shipped path is authoritative; follow-up chore: re-scope that architecture doc to the native worker + add a legacy note. |
| C8 | **`docs/architecture/backend-contract-map.md` documents the Tauri command surface as the contract.** | Doc header; `docs/architecture/system-overview.md:50` marks `tauri-app` legacy. | Read as historical. New work documents `RuntimeCommand`/`UiCommand` pairs. |
| C9 | **Component names in roadmap notes do not exist in code**: `ObjectList`, `PropertyGrid`, `DDLViewer`, `FormEditor`, `Confirmation`, `SearchResults`, `ActivityPanel`, `CommandPalette` (type), `DataGrid`. | `docs/notes/PRODUCT_ROADMAP.md:32`; audit of `crates/ui/src/components/` (real names in §4.5). | §4.5 replaces the invented names with a **name → real implementation → gap** contract table. The roadmap note itself is corrected in this task. |
| C10 | **Legacy Tauri build-blocker claim is stale.** Docs say `crates/tauri-app/src/commands/query.rs:50,114` fails to compile; those lines are a comment and a struct field in the current tree. The real compile blocker is the missing `_archive/frontend/dist` referenced by `tauri.conf.json`. | `docs/notes/PRODUCT_CAPABILITY_MATRIX.md:348`, `docs/plans/active/core-safety-hardening/VERIFICATION.md:134`; current `query.rs` content; `tauri-app/tauri.conf.json` `frontendDist`. | Recorded. **Not verified by build** (no build was run in this task); the claim "gates green" must still be scoped or `tauri-app` removed. |
| C11 | **SQLite backup mechanics misdescribed**: matrix says "file copy/VACUUM". | `docs/release/provider-capability-matrix.md:103,155`; code `sqlite_backup.rs:60-103` = `VACUUM INTO` + temp file + atomic hard-link publish. | §9 states the real mechanism; follow-up chore: fix two doc lines. |
| C12 | **SSH capability flag is not enforced.** Flag says PostgreSQL-only; `CompositeConnector::effective_config` will start a tunnel for any driver whose config carries `ssh_tunnel`. | `capabilities.rs:171,226`; `infrastructure/connector.rs:106-127`; `runtime/src/lib.rs:72-76` | Recorded as a backend-gating gap (Phase I01). |
| C13 | **`SettingsRepository` is dead code.** Port + implementation exist; nothing constructs or calls them. | `core/ports/settings_repository.rs:7-11`, `infrastructure/meta/settings_repo.rs:9-33`; grep finds no consumer. | K01 defines the target settings architecture and decides: reuse this port or delete it. |
| C14 | **Stale line counts / endpoints in release docs.** `provider-capability-matrix.md:151-152` claims 734/423-line introspectors; actual 995/726. | File lengths audited. | Cosmetic follow-up chore; no product impact. |
| C15 | **Release-boundary drift between notes.** `V0_1_CLOSURE_PLAN.md:59` puts Phase G in v0.2–v0.3; `PRODUCT_ROADMAP.md:22-25` puts Phase G remainder in "Later"; the 20-milestone packing at `PRODUCT_ROADMAP.md:226` omits M18/M20 from v0.2/v0.3. | Both docs. | §6 and §7 of this document are the single release/milestone authority; the notes are pointed here instead of restating. |

### 2.6 Baseline metrics (estimates, from §2.2 classification)

- v0.1-shippable engine surface: **~85%** (`PRODUCT_CAPABILITY_MATRIX.md` §17 concurs).
- DBeaver-class breadth: **~35%**.
- Highest-value absent subsystems, in order: typed object CRUD → routines → import/export
  wiring → monitoring → security UI → compare/migration → productivity → settings depth.

---

## 3. Final Information Architecture / Activity Bar

### 3.1 Final rail

```text
▣ Explorer      — connections, databases, schemas, objects, security nodes
⌕ Search        — first-class search activity (Phase G / G02)
⌘ Queries       — open queries, saved queries, history, snippets, scratch
◫ Data          — table data editor, open results, data transfer shortcuts
◇ ER            — diagram, search/neighborhood (design mode stays deferred)
✦ Agent         — Ask / Edit / Agent, sessions, tool activity, confirmations
▥ Monitoring    — Overview, Sessions, Active Queries, Locks, Transactions, Size/Stats
⇄ Transfer      — Import, Export, Backup, Restore, Schema Compare, Data Compare, Migration
⚙ Settings      — General, Appearance, Editor, Data Grid, Connections, AI Providers,
                  Keybindings, Backup/Restore, Security, Advanced
```

Deferred top-level activities (add only with a wiring milestone, never as placeholders):

```text
♙ Security      — currently a node under Explorer → database → Security (Phase E pages)
▶ Tasks/Jobs    — no scheduler exists anywhere; J01 is the gate (Later)
★ Favorites     — a filter inside Search/Queries, not a rail icon
```

### 3.2 Sidebar groups per activity

| Activity | Groups |
|---|---|
| Explorer | Connections (folders → tags → favorites) → Databases → Schemas → Tables, Views, Materialized Views, Functions, Procedures, Sequences, Types, Security (Users, Roles) |
| Search | Objects (connection, database, schema, table, column, view, routine, sequence, type), Queries (saved, history), Commands, Agent actions, Settings; filters for favorites/recent/pinned |
| Queries | Open Queries, Saved Queries (folders/tags), Query History, Snippets, Scratch SQL, Templates |
| Data | Open table workspaces, Open result sets, Recent data operations, Transfer shortcuts (import/export for current target) |
| ER | Diagram, search/neighborhood entry, schema selector (design mode deferred) |
| Agent | Ask, Edit, Agent mode, Recent sessions, Tool activity, Pending confirmations, Audit log |
| Monitoring | Overview, Sessions, Active Queries, Locks, Transactions, Size & Stats, Maintenance |
| Transfer | Import, Export, Backup, Restore, Schema Compare, Data Compare, Migration history/preview |
| Settings | General, Appearance, Editor, Data Grid, Connections, AI Providers, Keybindings, Backup/Restore, Security, Advanced |

### 3.3 Rail rules

1. The rail is fixed-width (48 px, `tokens.rs::ACTIVITY_BAR_WIDTH`) and never grows a
   scrollbar. Context lives in the sidebar, not in the rail.
2. A rail item may only be added by the milestone that wires it. Until then, no icon.
3. Placeholder surfaces (`draw_activity_placeholder`, `navigation_view.rs:637-648`) are
   removed in the milestone that replaces them — Monitoring in D02, Transfer in C05.
4. Every activity owns a sidebar header with (a) activity title, (b) target context
   selector (connection / database / environment), (c) a single primary action, (d) a
   `SearchInput` filter (`components/input.rs::SearchInput`).
5. Activity state is persisted (`dbpro.native.activity`), and switching activity never
   discards workspace tabs or staged changes (existing close-guard semantics retained).

### 3.4 Command surface

- `⌘K` / `Ctrl+K` opens the palette; palette modes are Quick Open and Commands today
  (`palette_view.rs`, `PaletteMode`), and Phase G01 expands coverage to columns, views,
  routines, saved-query SQL, history SQL, settings, and new commands.
- All commands route through `UiCommand`; **no view may execute a database operation
  directly** (§4.1).
- Keybindings are data, not code, after K01 (`AppCommand` → binding table), with a
  discoverable cheatsheet and conflict detection.

### 3.5 Workspace tab model

Existing tab kinds (`ui/src/app.rs:222-231`, `workspace_view.rs:17-542`): Welcome, Query
document, Table, SchemaObject, Diagram, ComponentGallery. Target additions are **variants of
existing kinds**, not new chrome:

| New surface | Tab kind | Reuses |
|---|---|---|
| View / MatView / Sequence / Type / Schema / Database | `SchemaObject` (typed payload) | `schema_object_view.rs` + new ObjectTabs |
| Routine (source/params/execute) | `SchemaObject` (routine payload) | B01–B04 |
| Monitoring page | Workspace tab opened from Monitoring activity | D02 |
| Schema compare / migration review | New `Compare` tab kind | F01–F04, `components/diff.rs` |
| Import/Export job | `Transfer` tab kind (job detail) | C01–C05 |

---

## 4. Architecture Principles

### 4.1 The dependency rule (unchanged, and enforced)

```text
UI (egui view)
  → UiCommand                     (crates/ui/src/runtime.rs)
  → RuntimeCommand                (crates/runtime/src/worker.rs)
  → Application Action / Service  (crates/core/src/application/)
  → Port trait                    (crates/core/src/ports/)
  → Provider Adapter              (crates/infrastructure/src/{postgres,sqlite}/)
  → Database
  → UiEvent / RuntimeEvent        (bounded, request-scoped)
  → reducer applies to AppState
```

Forbidden, and to be checked in review:

- A view importing or calling anything under `db-pro-infrastructure`.
- Building SQL in the renderer (`crates/ui`). Existing exception to remove: the local CSV
  writer in `query_view.rs:1357-1389`.
- A service reaching for a concrete connector instead of a port.
- A capability check that lives only in the UI (`app.rs:674-683,927-936` are today's only
  capability consumers — this must move server-side).

### 4.2 Command/event rule

- Every new user-facing operation adds: a `UiCommand` variant, a `RuntimeCommand` variant,
  a `RuntimeEvent` variant, and a `UiEvent` variant, plus the translation in
  `crates/native-app/src/translate.rs`.
- Long operations are `RequestId`-scoped, cancellable, and must emit a terminal event even
  on failure (`docs/07-fe-architecture.md` state rules).
- Dead variants found in this audit (`RuntimeCommand::CancelOperation`,
  `UiEvent::QueryQueued`, `UiCommand::OpenQuery`) must be removed or wired, not copied.

### 4.3 Capability gating rule

1. `DatabaseCapabilities` (`domain/capabilities.rs`) is the single source of truth, extended
   with the groups in §9.
2. **The backend consults it.** Target: the mutation/DDL/transfer/monitoring services reject
   an unsupported operation with `DbError::Unsupported` **before** emitting SQL.
3. The UI renders disabled state with a reason string from the same capability source —
   never a hardcoded per-driver branch.
4. A capability flag with no implementation is a defect (C2/C3); flags and code land in the
   same PR.

### 4.4 Potential new domains and services

| Service | Port (new/extended) | Consumed by |
|---|---|---|
| `ObjectMutationService` | `DbConnector::{execute, execute_batch}` + new `ObjectDdlPort` (typed) | A01–A08 |
| `RoutineService` | `DbConnector` + new `RoutinePort` (list/source/params/create/drop) | B01–B04 |
| `TransferService` | new `TransferSource`/`TransferSink` traits (streaming) | C01–C05 |
| `MonitoringService` (Admin) | new `AdminPort` (sessions, queries, locks, sizes, maintenance, terminate) | D01–D04, H03 |
| `SecurityService` | extend `UserManager` (memberships, schema grants, password) | E01–E03 |
| `CompareService` | extend `SchemaService::diff_schemas` + new `MigrationPlanner` | F01–F04, H02 |
| `SearchService` | new `SearchIndex` port (local meta store backed) | G01–G02, H01 |
| `TaskService` (jobs) | new `TaskRepository` + scheduler | J01 (Later), C01 job records |

Rules for new services: constructor-injected ports only; no `egui`, `sqlx`, `rusqlite`, or
`tauri` imports inside `crates/core` (already true today — keep it true); one service per
capability group, never one service per object type.

### 4.5 Shared UI architecture (component contract)

The names below are the **contract names for this roadmap**. The "Implementation today"
column is real code (verified); "Gap" is what the milestone must add. No feature may invent
its own styling system: all surfaces compose these components and read semantic tokens from
`DbProTheme` (`theme.rs`) + `tokens.rs`.

| Contract component | Implementation today | Gap / action |
|---|---|---|
| `ActivityBar` | `components/workspace.rs::{ActivityBar, ActivityBarItemKind}` (gallery-only) + real rail in `navigation_view.rs:412-460` | Adopt the component as the single rail renderer |
| `SidebarHeader` | `components/nav.rs::{PageHeader, SectionHeader, Breadcrumb}` (gallery-only) | Promote into a real header used by every activity |
| `ObjectTree` | `explorer_tree.rs::draw_codex_tree_row` + `components/tree.rs::{DatabaseTreeNode, reveal_children}` | Add multi-root support (Explorer + Monitoring + Transfer job trees) |
| `ObjectList` | *does not exist* (Explorer folders are bespoke: `explorer_folders.rs`, `explorer_details.rs`) | New shared list (header row, sort, filter, multi-select, context menu) used by Explorer, routines, security, monitoring, transfer jobs |
| `ObjectHeader` | `table_view.rs:139-327` breadcrumb header; `components/nav.rs::Breadcrumb` | Extract as shared header (target badge, environment badge, capability-disabled actions) |
| `ObjectTabs` | `app.rs:222-231` (`TableView`) + `table_view.rs:243-251`; `components/tabs.rs::{SegmentedTabs, UnderlineTabs}` | Generalize to any object kind (Data/Structure/Indexes/FKs/Constraints/Dependencies/DDL/Definition/Parameters/Execute) |
| `PropertyGrid` | *does not exist*; metadata rendered ad hoc in `table_metadata_view.rs` | New two-column name/value grid with edit mode; used by all object detail pages |
| `DDLViewer` | `table_editor_view.rs:1363-1431` (`draw_ddl_script_card` + confirmation) + `components/code.rs::CodeBlock` | Extract shared DDL viewer (read + generate + copy + open in query + apply with preview) |
| `CodeEditor` | `crates/ui/src/editor/*` (buffer, cursor, selection, syntax, renderer, completion, brackets, diagnostics, decorations, prediction) | Reuse for routine source editing (B03) |
| `DataGrid` / `ResultGrid` | `result_grid.rs` + `result_grid_view.rs::draw_result_grid` (shared by table editor + query results) | Extend for import preview/mapping (C02–C04) and compare results (F01) |
| `FormEditor` | `components/form.rs::{FormState, FormField, Label, FieldRule, ValidationMode}` + connection form | Reuse for every create/alter dialog; add field types for SQL types/expressions |
| `SearchInput` | `components/input.rs::SearchInput` (exists, used) | Extend with scope selector + result count |
| `CommandPalette` | `palette_view.rs` + `PaletteAction`/`PaletteItem` (`app.rs:131-159`) | Promote to a `components/` widget; extend the index (G01/G02) |
| `DiffViewer` | `components/diff.rs::{DiffViewer, DiffLine, DiffLineType}` (gallery-only) | Wire into F01/F02 (schema + data diff) and the agent patch flow |
| `ConfirmationDialog` | `components/dialog.rs::Dialog`, `components/agent_primitives.rs::{ExecutionApproval, RiskLevel}`, `components/transaction.rs::DestructiveOperationDialog` (gallery-only) | One confirmation contract (target, operation class, SQL preview, environment) used by every mutation |
| `ProgressView` | `components/feedback.rs::{Progress, Spinner}`, `components/dev_tools.rs::ProgressRing`, `RuntimeEvent::OperationProgress` | Streaming progress for transfer/backup/monitoring refresh |
| `EmptyState` | `components/chrome.rs::EmptyState` + `legacy.rs::empty_state` | Standardize (empty vs loading vs disabled-with-reason) |
| `ErrorState` | `components/alert.rs::Alert` + status-bar error surfaces | Add a first-class error state with retry + copy diagnostics |
| `ActivityLog` | `components/logs.rs::{LogViewer, LogEntry, LogLevel}` (gallery-only) + `AgentAuditEntry` + `OutputTab::Messages` | Promote as the audit/activity surface (mutation audit, transfer jobs, agent tool activity) |

Migration rule: a milestone that needs one of these and finds it gallery-only must **wire
the existing component** (with tests) rather than fork a view-local variant.

### 4.6 Persistence and settings architecture

Two stores exist and must not be mixed:

| Store | Location | Holds | Rule |
|---|---|---|---|
| Meta store (SQLite) | `<data_dir>/meta.db` via `SQLiteMetaStore` (`infrastructure/meta/`) | connections, secrets **references**, query history, saved queries + folders, workspaces, settings row, introspection cache, run configs | Server-side facts. The `SettingsRepository` port (`meta/settings_repo.rs`) is currently unused (C13) and is the intended home for cross-window/shared settings |
| eframe storage | `dbpro.native.*` keys (`ui/src/app.rs:445-478`, `app_state.rs:13-102`) | theme version, dark mode, reduce motion, prediction mode, grid layouts/widths, query documents, local history, panel widths | UI-local view state only. Never credentials, never query results |

Rules for new persistent state:

1. UI layout/view state → eframe storage, version-stamped, with a reset path
   (`dbpro.native.theme-version` pattern).
2. Anything the runtime must know (favorites, snippets, tags, search index, job history,
   audit records, environment labels) → meta store through a port, with a migration in
   `infrastructure/meta/migration.rs` (`LATEST_VERSION` bump + idempotent `migrate`).
3. Credentials → keyring only (`KeyringVault`), never meta store, never eframe storage.
4. Keys audited as dead in this task (`dbpro.native.connections-pane-height`,
   `dbpro.native.schemas-pane-height` are persisted but never read) are either wired or
   removed.

### 4.7 Provider-specific logic rule

- Provider differences live in: `DatabaseCapabilities` (flags), adapter dialect
  (`SqlDialect`), and adapter implementations. Nowhere else.
- A view may ask "capability X?" and render disabled; it may not ask "is this PostgreSQL?".
- SQL dialects: quote identifier + placeholder + pagination are already dialect-owned
  (`ports/dialect.rs`), and mutation DDL must go through the same mechanism.
- Cross-provider diffs/transfers must refuse operations that cannot be expressed in the
  target dialect (F01: PG→SQLite structural-only) instead of generating lossy SQL.

### 4.8 Anti-patterns (rejected in review)

- Building SQL strings in a view, a component, or the palette.
- New per-object dialog variants that duplicate `FormEditor` behavior with different visuals.
- A rail placeholder for a future feature.
- A capability flag without implementation, or a release doc claiming a capability the code
  does not have (§2.5).
- Adding a second cancellation registry. The native path owns cancellation in
  `crates/runtime/src/worker.rs` maps; the legacy Tauri registry
  (`crates/tauri-app/src/cancel.rs`) must not be extended.
- Reading `DatabaseCapabilities` in the UI for enforcement; the UI may only use it for
  presentation.

---

## 5. Feature Areas A–L

Every area uses the same 14 fields: Goal / User value / Current implementation / Missing
functionality / Proposed architecture / UI surfaces / Provider differences / Safety model /
Testing strategy / Runtime verification / Performance concerns / Dependencies / Non-goals /
Exit criteria.

### A. Database Object CRUD

- **Goal.** Every object the provider can introspect becomes manageable through typed forms:
  inspect → generate DDL → preview → create/alter/drop → refresh metadata → view DDL →
  dependencies/references.
- **User value.** The single largest DBeaver-class gap: turns DB Pro from a viewer into a
  workbench for schemas, not just rows.
- **Current implementation.** Introspection is strong for tables/columns/PK/FK/index/check/
  trigger/function (`postgres/introspect.rs:113-779`, `sqlite/introspect.rs:82-572`);
  reconstructed DDL is dialect-aware (`schema_service.rs:264-620`); typed builders exist but
  are **dead code** (`application/ddl_builder.rs:19-183`, only referenced by `mod.rs:4`);
  table DDL tab already supports preview + confirmation (`table_editor_view.rs:1363-1431`).
- **Missing functionality.** Typed create/alter/drop services; `ObjectMutation*` types;
  view columns; materialized views; sequences; enum/domain/composite types; schema and
  database objects; add/drop/rename constraint; alter column type/rename/set default;
  unique-constraint catalog fidelity; capability enforcement in the DDL path.
- **Proposed architecture.** New `ObjectMutationService` + `ObjectDdlPort`, driven by
  `ObjectAction` → `ObjectMutationRequest` → `ObjectMutationPreview` →
  `ObjectMutationResult`. Service order: validate against `DatabaseCapabilities` → build DDL
  via a typed builder (extend `ddl_builder.rs`) → enforce `StatementSafety` policy → preview
  in UI → execute via `execute_batch` in one transaction where supported → invalidate
  introspection cache → emit refreshed schema event.
- **UI surfaces.** `ObjectHeader` + `ObjectTabs` + `PropertyGrid` + `DDLViewer` +
  `FormEditor` + `ConfirmationDialog`; Explorer context menus per object type; a "Generate
  SQL" escape hatch that opens the query editor instead of executing.
- **Provider differences.** PostgreSQL: full column/constraint/type/index/sequence CRUD,
  matviews, enums/domains, `ALTER COLUMN TYPE`, schemas. SQLite: table create/alter
  (add/drop/rename column), index create/drop, view create/drop, trigger create/drop; **no**
  alter-type, no routines, no sequences/types, no schemas — these are hidden with a reason.
- **Safety model.** Mutation class by default; drop/rename/type-narrowing are Destructive
  (multi-step confirmation); read-only connections block all Mutation+; preview is mandatory
  and shows affected object, environment, and the exact SQL; transactional batch where
  supported (both providers support transactional DDL per `provider-capability-matrix.md`).
- **Testing strategy.** Builder unit tests (exact SQL per dialect), service tests with
  `MockDbConnector` (policy rejection, capability rejection, cache invalidation), SQLite
  integration round-trips (create → introspect → alter → drop), PostgreSQL live tests in the
  ignored suite, UI tests for preview/disabled states.
- **Runtime verification.** Per object kind, on native UI, both providers: create → verify
  in Explorer → alter → verify DDL → drop → verify removal; plus error paths (permission
  denied, dependent object, read-only connection).
- **Performance concerns.** Nothing heavy; ensure a drop/alter of a table with many
  dependents does not trigger a full re-introspection storm (targeted cache invalidation
  only), and that large property grids (10k columns) are virtualized.
- **Dependencies.** A01 framework; §9 capability enforcement; §10 policy producer;
  introspection cache invalidation (exists).
- **Non-goals.** ER design/edit mode; data-modeling diagrams; migration authoring by drag
  (that is F03/F04); table partitioning management (Later).
- **Exit criteria.** For every object kind in scope: matrix rows reach PARTIAL-or-better
  with UI wired and provider evidence recorded; zero flag-without-code in scope; zero SQL
  string built in `crates/ui`.

### B. Functions / Procedures (Routine Workbench)

- **Goal.** First-class routine surface: browse → read source → edit → execute with typed
  arguments → read output → manage lifecycle.
- **User value.** Application logic lives in routines; today DB Pro can only list them and
  show a Definition tab.
- **Current implementation.** `pg_proc` introspection for `prokind IN ('f','p')`
  (`postgres/introspect.rs:762-779`), `Function` domain struct with `routine_type`
  (`domain/schema.rs:119-128`), Explorer rows and read-only Definition view
  (`explorer_folders.rs:164-231`, `schema_object_view.rs`), agent `patch_query` for edit
  flows. SQLite returns `functions: Vec::new()` (`sqlite/introspect.rs:66`).
- **Missing functionality.** Parameter metadata (names/types/modes/defaults), overload
  identity/signature, `Procedure` as a real object, source editor with preview, create/
  replace/drop, `CALL` helper, execute form with typed inputs and output rendering, notices,
  dependency listing for routines.
- **Proposed architecture.** `RoutineService` + `RoutinePort` returning a `Routine` domain
  aggregate (kind, signature, params, return type, language, volatility, security definer,
  source). Execution reuses `QueryService::execute`/`execute_multi` with `QueryParam`
  binding; classification treats `CALL` as Destructive so it always confirms. Creation uses
  `ObjectMutationService` (A01).
- **UI surfaces.** Explorer → Functions/Procedures; `ObjectTabs`: Source / DDL /
  Parameters / Dependencies / Execute; execute form with typed inputs, NULL toggle, default
  support; output pane with result grid, notices/messages, and errors.
- **Provider differences.** PostgreSQL only for parameter metadata, execution, and
  lifecycle. SQLite: hide the whole activity; the Explorer "Functions" folder is already
  capability-gated (`explorer_view.rs:639-642`) and must stay empty-with-reason, not
  erroring.
- **Safety model.** `CREATE OR REPLACE` = Mutation with preview; `DROP` = Destructive with
  confirmation; execution classified (`CALL`→Destructive) and confirmed; read-only
  connections block all three; arguments are always bound parameters, never interpolated.
- **Testing strategy.** Domain tests for signature/overload identity; builder tests for
  `CREATE OR REPLACE`/`DROP`/`CALL`; service tests with mocks for capability rejection;
  PostgreSQL live tests (create → call with args → read result → replace → drop);
  SQLite gating tests (capability boundary, no unsupported SQL emitted).
- **Runtime verification.** Live PostgreSQL walkthrough on native UI: parameter form
  renders metadata, execution returns rows/notices, replace changes source, drop removes the
  row, all with screenshots.
- **Performance concerns.** Routine list can be large (thousands) — `ObjectList` must
  virtualize and filter server-side or cached; source viewing must not re-introspect the
  whole schema (targeted fetch).
- **Dependencies.** A01 (mutation framework), §4.5 components, `QueryParam` (exists),
  B01 before B02–B04.
- **Non-goals.** Debug server/step debugging; PL/pgSQL language server; routine profiling;
  SQLite routine emulation.
- **Exit criteria.** Browse/source/execute/manage work end-to-end on PostgreSQL with
  evidence; SQLite shows a correct disabled state with reason; matrix Functions/Procedures
  rows PARTIAL-or-better.

### C. Data Transfer (Import / Export / Backup / Restore)

- **Goal.** Move data in and out safely at scale: import wizards, wired export workbench,
  reliable backup/restore UX, and a job list for long transfers.
- **User value.** Import is the most common non-query task; export already works in the
  backend but is unreachable, so users get a lossy local writer today.
- **Current implementation.** `ExportService` CSV/JSON/XLSX with BIGINT/`2^53` and
  duplicate-column guards (`export_service.rs:65-249`) — BACKEND_ONLY; native UI has a
  separate unquoted writer (`query_view.rs:1357-1389`); result-grid clipboard copy matrix
  (`result_grid_view.rs:1648-1783`); backup/restore engines for both providers
  (`backup/pg_dump.rs`, `backup/sqlite_backup.rs`) with a minimal Settings UI and no
  progress/cancel.
- **Missing functionality.** Entire import pipeline (readers, encoding, preview, column
  mapping, type inference/coercion, null/default policy, conflict strategy, transactional
  apply, progress, cancellation, streaming, error-row report); native wiring for backend
  exporters; SQL INSERT/COPY export; transfer job model; backup progress/format choice.
- **Proposed architecture.** New `TransferService` with streaming `TransferSource`
  (file/query/table) and `TransferSink` (table/file) traits, a `TransferJob` record in the
  meta store, chunked batched writes inside a transaction policy (`per-batch` vs
  `whole-file`), cancellation via the existing request-scoped cancellation maps, and
  `OperationProgress` events. Export reuses `ExportService`; backup reuses `BackupEngine`
  with a new progress-aware variant.
- **UI surfaces.** Transfer activity (Import / Export / Backup / Restore / Compare /
  Migration); import wizard steps: File → Preview → Mapping → Options → Review SQL →
  Progress → Result (error rows download); export workbench: source (result/table/selection/
  query), format, options, destination; job list with per-job progress, cancel, and log.
- **Provider differences.** Both providers support CSV/JSON/XLSX export and CSV/JSON import.
  Excel import needs a reader dependency (new). PostgreSQL `COPY` export is PG-only; SQLite
  gets `INSERT` statements only. Backup: PostgreSQL shells out to `pg_dump`/`pg_restore`
  (PATH requirement, `LIM-015`); SQLite uses `VACUUM INTO` + atomic publish — restore on
  SQLite requires the target connection to be disconnected (`backup_service.rs:67-71`).
- **Safety model.** Import is Mutation (whole-file transaction default; per-batch is opt-in
  with an explicit warning that partial data may remain); export is ReadOnly but reads must
  pass policy (already enforced); restore is Destructive with overwrite confirmation and
  target preview; read-only connections block import and restore; no silent overwrite of
  output files (existing `create_new` behavior retained).
- **Testing strategy.** Golden-file round-trips per provider and format; malformed/edge
  matrices (BOM, CRLF, quoted newlines, mixed encodings, duplicate headers, type conflicts,
  oversized fields); cancellation mid-import leaves a deterministic state; large-file
  streaming tests assert bounded memory; backup/restore live runs per provider.
- **Runtime verification.** 100k-row CSV import and export on both providers with observed
  memory and progress; cancellation at ~50% with a recorded final state; error-row report
  for an intentionally broken file; backup/restore live runs.
- **Performance concerns.** Streaming only — never materialize a full dataset. Import must
  not hold the whole file in memory (bounded chunk + backpressure), export must stream rows
  (not build one giant `QueryResult`), progress events must be rate-limited (the UI must not
  repaint per row), and the grid used for preview must be windowed.
- **Dependencies.** §4.5 `DataGrid` reuse; cancellation contract; `OperationProgress`;
  A01 not required; C01 before C02–C04.
- **Non-goals.** Database-to-database transfer (Later, `D2D`), scheduled/periodic transfer,
  cloud destinations, compressed archive formats, schema inference into a full migration
  (that is F03).
- **Exit criteria.** Matrix import rows PARTIAL-or-better on both providers; export
  BACKEND_ONLY eliminated; Transfers placeholder removed; large-file evidence recorded per
  provider.

### D. Monitoring / Administration

- **Goal.** See what the database is doing and act on it: overview → sessions → active
  queries → locks → transactions → size/stats → maintenance.
- **User value.** The clearest boundary between "SQL client" and "database IDE/admin tool";
  also the prerequisite for AI slow-query analysis.
- **Current implementation.** **Nothing.** No `AdminService`, no `AdminPort`, and zero
  `pg_stat_activity` / `pg_locks` / `pg_stat_user_tables` occurrences under `crates/`. Only
  capability flags (`capabilities.rs:167-170` for `server_sessions`, `partitions`,
  `tablespaces`, `object_dependencies`), `c.reltuples` estimates in the table query
  (`postgres/introspect.rs:129-140`), and PG-only inherent connector extras
  (`object_dependencies`, `list_partitions`, `list_tablespaces` via `runtime/src/api.rs:916-952`).
- **Missing functionality.** Session listing, active-query listing with duration/wait state,
  lock and blocking graph, transaction listing, cancel-query from the admin surface,
  terminate-session, database/table/index size, scan statistics, `VACUUM`/`ANALYZE`, server
  information, and refresh/polling UX.
- **Proposed architecture.** New `MonitoringService` + `AdminPort` with read-only queries
  for listings and explicitly typed admin operations (cancel, terminate, vacuum, analyze)
  that pass through the §10 policy with `Administrative`/`Destructive` classification.
  Monitoring views poll on an interval owned by the UI's request registry (never a background
  thread mutating state), with pause/resume.
- **UI surfaces.** Monitoring activity: Overview, Sessions, Active Queries, Locks,
  Transactions, Size & Stats, Maintenance; each as an `ObjectList`/grid with actions in the
  context menu; "Explain query" and "Open SQL" as safe hand-offs to the query editor.
- **Provider differences.** PostgreSQL: full surface (subject to role privileges —
  `pg_stat_activity` visibility depends on `pg_monitor`/superuser; the UI must show what the
  role can see rather than pretending). SQLite: no sessions/locks/terminate; only
  meaningful metrics (file size, page count/freelist, `PRAGMA`-derived stats) and
  `VACUUM`/`ANALYZE`/`PRAGMA optimize`. Everything else hidden with a reason.
- **Safety model.** Listings are ReadOnly. Cancel Query = Administrative (confirmation
  showing the target pid/user/query text). Terminate Session = Destructive (typed
  confirmation + environment warning; Production shows the strong variant). VACUUM/ANALYZE =
  Administrative with confirmation for `VACUUM FULL` (locks table exclusively). No
  auto-kill, no auto-vacuum policy, no scheduled maintenance.
- **Testing strategy.** Query-accuracy tests against fixtures (deterministic with a seeded
  workload), service tests with `MockAdminPort` for classification/confirmation, UI tests for
  polling lifecycle and stale-response handling, permission-denied paths, and PostgreSQL live
  tests in the ignored suite.
- **Runtime verification.** Live PostgreSQL: observe a long-running query, cancel it from the
  Monitoring page, verify `pg_stat_activity` reflects it and the query editor's Stop path
  agrees; lock graph reproduced with two sessions; terminate session; sizes match `psql`
  values. SQLite: verify only the meaningful metrics render.
- **Performance concerns.** Polling must not re-introspect or block the UI thread; large
  session/lock lists need virtualization and a row cap with "showing N of M"; query-text
  columns can be huge (truncate with a viewer).
- **Dependencies.** §2.5 C1 (cancel semantics), §9 capability enforcement, D01 before D02–D04.
- **Non-goals.** Replication/monitoring dashboards, alerting, historical metric storage,
  connection-pool inspection, extension inventory, automatic remediation.
- **Exit criteria.** Matrix §8 core rows PARTIAL-or-better on PostgreSQL, correctly gated on
  SQLite; Monitor placeholder removed; cancel/terminate evidence recorded; zero auto-kill
  behavior.

### E. Users / Roles / Permissions

- **Goal.** Expose the existing PostgreSQL security backend as a safe workbench: roles,
  memberships, grants, and DDL visibility.
- **User value.** Completes the admin story; the backend is already implemented and idle.
- **Current implementation.** `UserService` (`core/application/user_service.rs:53-136`,
  PostgreSQL-only guard at `:130-136`), `PostgresUserManager`
  (`infrastructure/postgres/user_manager.rs:62-209`) with identifier quoting
  (`quote_identifier:11-19`) and an 8-privilege allowlist (`quote_privilege:21-33`), wired to
  `UserApi` (`runtime/src/api.rs:797-861`) and Tauri commands — **no native UI**.
- **Missing functionality.** Native surface; role detail (attributes, DDL); memberships
  (backend gap); schema-level and column-level grants; database grants; password rotation;
  role DDL view; ownership display; default privileges; RLS (Later).
- **Proposed architecture.** Extend `UserManager` port with `memberships`,
  `grant_schema_privilege`, `revoke_schema_privilege`, `alter_role`, `change_password`;
  `SecurityService` composes it with `ConnectionRegistry` and the §10 policy. All identifiers
  go through the existing quoting/allowlist helpers — no free-form SQL text from the UI.
- **UI surfaces.** Explorer → database → Security → Users / Roles; `ObjectTabs`: Properties /
  Memberships / Privileges / DDL / Owned objects; privilege matrix (grants × objects) with a
  filter and an explicit "generate SQL" view.
- **Provider differences.** PostgreSQL only. SQLite must render the Security node with a
  disabled state and reason (`UserService` already rejects before any provider call —
  `user_service.rs:130-136`, tested at `:139`).
- **Safety model.** Grants/revokes = Mutation (preview + confirm). Create/drop role =
  Destructive (confirm; drop shows a dependency warning if the role owns objects). Password
  change = Destructive-like (typed confirmation, never logs or renders the value, keyring is
  never involved). Superuser/createdb/replication attribute changes warn explicitly. Every
  statement is composed from the allowlist — never string-interpolated privilege text.
- **Testing strategy.** SQL-generation unit tests (quoting, injection attempts, allowlist
  rejection), service tests for the PG-only guard and read-only gate, PostgreSQL live tests
  (create role → grant table/schema → verify with `list_privileges` → revoke → drop), UI tests
  for disabled states and confirmation copy.
- **Runtime verification.** Live PostgreSQL walkthrough on native UI: create a login role,
  apply a table grant and a schema grant, verify membership tree, change password, drop the
  role; evidence recorded separately from SQLite's disabled-state evidence.
- **Performance concerns.** Role/privilege lists are small; the matrix must still paginate or
  filter when a database has thousands of tables.
- **Dependencies.** §4.5 components; E01 before E02/E03; memberships backend addition.
- **Non-goals.** Full IAM administration, LDAP/AD integration, cloud IAM, RLS policy editing,
  auditing/logins dashboards.
- **Exit criteria.** Matrix §11 BACKEND_ONLY eliminated for in-scope rows; SQLite gated with
  reason and test; no password ever rendered or logged (review evidence).

### F. Schema Compare / Migration

- **Goal.** Diff → migration plan → review → safe apply, with dependency-aware ordering and a
  rollback plan.
- **User value.** Team workflows (dev→staging→prod) without leaving the IDE.
- **Current implementation.** `SchemaService::diff_schemas` (`schema_diff.rs:11-156`) diffing
  tables/columns/types/indexes only, driver-agnostic, deterministic; `DataDiffService` is
  count-only (`data_diff.rs:20-77`); both BACKEND_ONLY; no DDL diff, no migration
  generation/apply; `meta/migration.rs` is the app-store migration, **not** this feature.
- **Missing functionality.** Diff depth (PK/FK/unique/check/views/triggers/routines/
  sequences/types); snapshot sources (connection/database/schema/saved snapshot);
  `Added/Removed/Changed` semantics per object; DDL diff generation; ordered migration
  operations; warnings; apply path; rollback plan; diff UI.
- **Proposed architecture.** `CompareService` extends the existing diff (keep it
  driver-agnostic; add per-object comparators), then a `MigrationPlanner` converts the diff
  into ordered `MigrationOperation` records (create type → create table → alter → constraints
  → indexes → FKs → views/routines), each carrying its own SQL per target dialect, its
  operation class, and an optional inverse. Apply reuses `execute_ddl_batch` in one
  transaction where supported; snapshots persist in the meta store so a comparison can be
  re-run without both databases online.
- **UI surfaces.** New `Compare` workspace tab: source/target pickers, filterable diff tree
  (`DiffViewer`), per-object property diff, generated SQL pane with copy/export, ordered
  operation checklist with warnings, Apply (gated) with confirmation, and a rollback plan
  view.
- **Provider differences.** PG↔PG and SQLite↔SQLite produce executable plans. Cross-provider
  (PG↔SQLite) is **structural comparison only** and must refuse to generate DDL that the
  target cannot express (e.g. type changes SQLite cannot perform) — reported as
  "cannot migrate" rows, never dropped silently.
- **Safety model.** Reading/compare = ReadOnly. Migration apply = Mutation per operation,
  Destructive for any `DROP`/type-narrowing/data-loss operation; **never auto-applied**;
  destructive operations require a dedicated typed confirmation and an explicit plan review;
  apply is transactional where supported with a documented rollback plan (or an explicit
  "no rollback available" warning); read-only connections cannot apply.
- **Testing strategy.** Fixture-pair diffs per provider (identical, additive, destructive,
  rename-lookalike, type change, index-only, constraint-only); planner ordering tests;
  dialect-correctness tests for generated SQL; refusal tests for cross-provider lossy
  operations; apply tests against SQLite fixtures and PG live; rollback script validity.
- **Runtime verification.** Real migration preview→apply walkthrough per provider (SQLite
  fixture, PostgreSQL live), including a destructive migration being blocked until
  confirmed, and a rollback executed after apply.
- **Performance concerns.** Comparing large schemas (1000 tables, 10k columns) must be
  bounded: compare on hashes/targeted keys, stream results into a virtualized tree, never
  build an unbounded operation list in memory; snapshot persistence must be bounded.
- **Dependencies.** A01 (DDL services + policy), D (optional context), F01 before F02–F04.
- **Non-goals.** Data compare beyond PK-matched sampled/limited rows (Later), auto-apply,
  migration history as a versioned changelog product, ORM migration frameworks, cross-engine
  data type translation matrices beyond explicit refusal.
- **Exit criteria.** Matrix §12 Schema Compare PARTIAL-or-better with UI; DDL diff exists;
  apply gated, transactional where supported, and evidenced; zero auto-apply path in code.

### G. Productivity / Search

- **Goal.** Make daily work fast: find anything, reuse SQL, tune the workspace.
- **User value.** This is where DB Pro can feel *better* than mature clients instead of
  merely equal.
- **Current implementation.** Palette with Quick Open + Commands (`palette_view.rs`,
  `PaletteMode`, `PaletteAction`), table coverage ≤100 (`palette_view.rs:150-163`);
  saved queries + folders (`meta/saved_query_repo.rs`, 7 methods); local dual history
  (20 in-memory + 500 persisted, `app.rs:286-288`); completion infrastructure
  (`query/schema_completion.rs`); grid layout persistence (`dbpro.native.grid-layouts`);
  2 hardcoded snippets (`query_view.rs:276-288`); `RunConfigRepository` unwired;
  `SearchInput` component exists.
- **Missing functionality.** Unified searchable item model + index; Search activity; palette
  coverage for columns/views/routines/saved-query SQL/history SQL/settings; snippet library
  with variables; scratch SQL; favorites/recent/pinned (objects, queries, connections);
  saved-query tags/folders move; connection folders/tags UI; query parameters dialog;
  keybinding customization; staged-copy fix; JSON tree viewer; DEFAULT handling.
- **Proposed architecture.** `SearchService` + `SearchIndex` port backed by the meta store
  (not a background crawler): lazily populate per connection on introspection, index rows for
  objects/columns/queries/commands/agent actions, keep it secret-free, and rebuild on
  schema-change events. Favorites/recent/pinned/snippets/tags are meta-store entities with
  migrations. Keybindings become an `AppCommand` → binding table resolved at event time.
- **UI surfaces.** Search activity (grouped results with type icons, keyboard-first
  navigation, preview pane); palette expansion; Queries sidebar sections; Settings → Editor/
  Keybindings; context-menu "Add to favorites / Pin"; snippet insertion with variable
  prompts.
- **Provider differences.** None for the index itself; object rows come from each provider's
  introspection. Queries/history/commands are provider-neutral.
- **Safety model.** The index must never contain credentials, connection secrets, or full
  result data (`docs/architecture/security-boundaries.md`). Search results that open a
  mutating surface still route through the normal confirmation path. Snippet variables are
  always bound parameters, never interpolated.
- **Testing strategy.** Index unit tests (insert/update/delete, schema-change invalidation,
  no-secret assertions), ranking/typing tests, palette command coverage tests, keybinding
  conflict tests, UI tests for keyboard-only flows and empty/error states.
- **Runtime verification.** Native walkthrough: `⌘K` → "Search column customer_id" opens the
  right object; find a saved query by SQL text; open a table from search; keyboard-only pass
  with screenshots.
- **Performance concerns.** Index build must be incremental and bounded (10k columns, 1000
  tables per connection); search must return within an interactive budget with debouncing;
  palette must not block the frame on first open.
- **Dependencies.** §4.6 meta-store migration pattern; G01 is independent and can ship first.
- **Non-goals.** Fuzzy-AI semantic search, cross-connection federated query, cloud sync of
  snippets, plugin/marketplace.
- **Exit criteria.** Matrix §13/§14 MISSING rows for in-scope items eliminated; Search
  activity real; index proven secret-free by test; keyboard-only verification recorded.

### H. Advanced AI

- **Goal.** Extend the agent from SQL assistance to database-workflow assistance on top of
  the stable typed tool layer — never autonomous destruction.
- **User value.** Optimize, migrate, and administer with an agent that has real database
  context and a confirmation gate.
- **Current implementation.** 9 typed tools (`runtime/src/agent.rs:565-634`) all routed
  through canonical `SchemaApi`/`QueryApi` (`agent_executor.rs:52-251`); Ask/Edit/Agent modes
  (`domain/agent.rs:57-61`, `agent_workflow.rs:519-542`); confirmation + anti-TOCTOU
  (`agent_orchestrator.rs:120-194, 338-547`); `patch_query` diff preview; bounded sampling
  (`MAX_AGENT_*` caps); `explain_query` (no `ANALYZE`).
- **Missing functionality.** Tools for compare/migration, slow-query analysis, sampled data
  analysis, sessions/plan inspection, index suggestion; visual plan rendering of real plans
  (`ExplainPlanTree` is gallery-only); context depth (plan + stats + locks).
- **Proposed architecture.** Every new AI capability is a typed tool registered in
  `tool_definitions()` with (a) a canonical backend call, (b) a declared risk class, (c) a
  confirmation requirement, and (d) a bounded output budget. Tools whose backends do not
  exist yet (D, F) are **documented in Phase H, not implemented** until those phases land.
  Read-only tools may auto-run under the existing `AllowReadOnlyAutoRun` policy; every
  mutating tool is preview + explicit confirmation with execution-time re-classification.
- **UI surfaces.** Agent panel tool cards with risk badges, plan/diff rendering in the
  workspace (`DiffViewer`, `ExplainPlanTree`), confirmation cards showing target +
  environment + SQL, and an audit trail in `AgentAuditEntry`/ActivityLog.
- **Provider differences.** PostgreSQL-only tools (sessions, locks, maintenance, plan
  analysis with stats) must be capability-gated per connection; SQLite gets SQL-level
  assistance only and reports unavailability instead of guessing.
- **Safety model.** No new tool may open its own database connection, bypass the
  application layer, or mutate without confirmation. `EXPLAIN ANALYZE` remains an execution
  (confirmation) — not a "read-only" tool. Any DDL produced by the agent goes through the
  A01 preview→apply path. No autonomous index creation, ever.
- **Testing strategy.** Tool-contract tests (parameter validation, bounded outputs), risk
  classification tests per new tool, confirmation/replay/idempotency tests mirroring
  `agent_orchestrator.rs` coverage, prompt-injection resistance for tool inputs, and
  provider-gating tests.
- **Runtime verification.** Per assistant: live provider E2E with a recorded confirmation
  step and a recorded refusal/capability-gated path; SQLite unavailable-tool behavior
  recorded separately.
- **Performance concerns.** Tool outputs must stay bounded (row/char caps already exist);
  plan/session payloads can be large — summarize + truncate deterministically; never stream
  unbounded rows into the model context.
- **Dependencies.** D01–D04 and F01–F04 backends for the administration/compare assistants;
  A01 for any generated-DDL path; monitoring for slow-query analysis.
- **Non-goals.** Autonomous multi-step database changes, self-scheduled agent runs, model
  training, embeddings/vector search over data, MCP (`LIM-008`).
- **Exit criteria.** Each shipped assistant has its own evidence record and a capability
  matrix row; zero paths where an AI-generated mutation executes without a user-approved,
  re-classified confirmation.

### I. Connections & Environment Management

- **Goal.** Complete, organize, and safely target connections across environments.
- **User value.** Users work across local/dev/staging/prod; the app must make the target
  unmistakable and the operations appropriate.
- **Current implementation.** Create/edit/delete(+confirm)/duplicate/test/connect/disconnect/
  reconnect/refresh, startup reconnect and workspace recovery, SSH tunnel (external `ssh`,
  `sshpass` secrecy), SSL mode (rustls), read-only flag, timeouts, max rows
  (`connection_view.rs:282-927`, `connection_service.rs:63-434`, `domain/connection.rs:80-256`).
  Folders/tags/favorites exist in the domain but are **dropped at the UI DTO boundary**
  (`UiConnectionSummary`, `translate.rs:41-42`). No import/export of profiles.
- **Missing functionality.** Connection folders/tags/favorites/color UI; profile
  import/export; explicit `allow_ddl`/`allow_destructive` connection flags; environment
  labels (Production/Staging/Development) with visual treatment; per-connection default
  schema; connection health/status detail; SSH/SSL further qualification (CA/cert fields are
  absent per `PRODUCT_CAPABILITY_MATRIX.md` §15).
- **Proposed architecture.** Environment becomes a first-class field on `ConnectionConfig`
  (enum, not free text) with a `Production` guard that the §10 policy consumes; connection
  organization reuses the existing domain folders/tags by fixing the DTO to carry them;
  profile export/import is a versioned JSON document **without secrets** (secrets re-entered
  or resolved from keyring by name), validated on import.
- **UI surfaces.** Connection dialog (existing) + Environment selector + advanced flags;
  Explorer connection list with folders/tags/color/favorite, environment badge on every
  connection header; `ObjectHeader` environment badge on every workspace tab; a
  "Production" banner variant for destructive confirmations.
- **Provider differences.** SSH tunneling applies to PostgreSQL (external binary, PATH
  dependency); SQLite is a local file with no credentials/SSL — the SSH field must not be
  accepted for SQLite (currently not enforced; C12). SQLite file picker is the equivalent
  affordance.
- **Safety model.** Read-only connections block Mutation+ at the service layer (exists).
  Target flags `allow_ddl`/`allow_destructive` become real producers for the existing policy
  branches (C5). Production connections: destructive operations require typed confirmation
  (type the object name), always show environment + host in the dialog, and Agent mutating
  tools default to requiring confirmation with no auto-run override.
- **Testing strategy.** Domain validation tests (environment parsing, profile import
  validation, no-secret assertions on export), service tests for policy derivation from
  environment + flags, UI tests for badges/disabled states, integration tests for
  reconnect/startup recovery, SSH E2E remains a tracked gap until a fixture exists.
- **Runtime verification.** Native walkthrough: create connections in all three
  environments, verify badges and confirmations differ, export/import a profile and prove no
  secret is in the file, attempt a destructive operation on a Production connection and
  capture the stronger confirmation.
- **Performance concerns.** Connection list virtualization for hundreds of connections;
  startup reconnect must not block first paint (existing async pattern retained).
- **Dependencies.** §10 policy producer; K01 for settings parity; A01 for DDL-blocking
  behavior on Production.
- **Non-goals.** Cloud secret managers, credential sharing between users, team connection
  sync, VPN management, non-SSH tunnels.
- **Exit criteria.** Folder/tag/favorite UI works and persists; environment labels appear on
  every target surface; profile export/import provably secret-free; `allow_ddl`/
  `allow_destructive` have a real producer with tests.

### J. Database Tasks / Jobs

- **Goal.** Run user-defined SQL/scripts on a schedule, visibly and safely.
- **User value.** Long-term convenience (nightly maintenance, periodic exports) — the
  lowest-value area of this roadmap.
- **Current implementation.** **Nothing.** No scheduler, no task entity, no worker loop.
- **Missing functionality.** Task model (SQL/script, target, schedule, timeout, failure
  policy), persistence + migration, executor with concurrency limits, run history, and UI.
- **Proposed architecture.** `TaskService` + `TaskRepository` (meta store) + a single
  scheduler owned by the runtime with at most one active run per task, reusing
  `QueryService::execute_multi`, the §10 policy per target connection, and stored run records.
  The C01 `TransferJob` model is the antecedent for job state; do not build two job systems.
- **UI surfaces.** Deferred activity (Tasks/Jobs) with list, editor, run history, and
  enable/disable; task status surfaced in the status bar.
- **Provider differences.** Provider-neutral; capability comes from the target connection's
  flags and environment.
- **Safety model.** A scheduled task on a Production connection is administrative by nature:
  creation requires an explicit confirmation showing environment + SQL + schedule; read-only
  connections reject task creation; destructive SQL inside a task is allowed only with a
  distinct "I understand this runs unattended" acknowledgement; tasks never run while a
  capability is unsupported.
- **Testing strategy.** Scheduler unit tests (no overlapping runs, missed-run policy,
  cancellation), repository migration tests, execution tests against SQLite fixtures, and
  failure-policy tests.
- **Runtime verification.** Run a task twice on a schedule with observed history entries and
  a captured cancellation; verify a read-only connection cannot host a task.
- **Performance concerns.** Scheduler must not compete with interactive work (single worker,
  bounded concurrency), and run history must be pruned by retention policy.
- **Dependencies.** §4.6 meta store; C01 job model; §10 environment policy.
- **Non-goals.** OS-level cron integration, distributed scheduling, dependency chains/ETL
  orchestration, alerting integrations.
- **Exit criteria.** Only if J01 is scheduled: tasks run, are recorded, are cancellable, and
  cannot bypass policy. Otherwise J stays an explicit non-goal beyond v0.4.

### K. Settings / Customization

- **Goal.** One settings center covering behavior that varies per user and per environment.
- **User value.** Users expect to tune editor, grid, connections, AI, and keybindings without
  editing files; today only Appearance and Backup paths have a page.
- **Current implementation.** Settings sidebar has Appearance (theme, reduce motion) and a
  Backup/Restore card (`navigation_view.rs:940-1062`); per-query overflow menu holds editor
  flags (only prediction is persisted); AI key lives in the Agent panel header; connection
  settings live in the connection dialog; `RunConfigRepository` exists but is unwired;
  `SettingsRepository` has no consumer (C13).
- **Missing functionality.** Pages: General, Appearance (exists, expand), Editor, Data Grid,
  Connections, AI Providers, Keybindings, Backup/Restore, Security, Advanced; a single
  settings model with validation, search, and reset-to-default; migration of ad-hoc flags
  into it.
- **Proposed architecture.** A `Settings` domain aggregate persisted through the existing
  `SettingsRepository` port (decide: reuse or delete — do not leave it dead), read by the
  runtime at startup and pushed to the UI via an event; UI-local view state stays in eframe
  storage. Every setting declares scope (global vs per-connection), default, and validation.
- **UI surfaces.** Settings activity with a sectioned page (nav list + content), searchable
  settings, per-setting reset, and a "modified" indicator; Keybindings page with conflict
  detection; Security page summarizing read-only/keyring/telemetry posture.
- **Provider differences.** Connection-scoped settings (timeouts, max rows, default schema,
  read-only) are per connection and hint at provider defaults; UI must not present a
  PostgreSQL-only setting on a SQLite connection.
- **Safety model.** Security-relevant settings (read-only default, telemetry, keyring
  fallback) require explicit confirmation and never weaken a policy silently; credential
  fields use `PasswordInput` and never round-trip values to the UI after save.
- **Testing strategy.** Settings round-trip tests, migration tests (existing storage keys →
  new model), validation tests, UI tests for reset/search/conflict detection.
- **Runtime verification.** Change a setting, restart, verify persistence; verify a
  per-connection setting applies only to that connection; verify keybinding remap takes
  effect and conflicts are reported.
- **Performance concerns.** Settings load must be instant at startup and must not block the
  first frame; a large keybinding table must not be scanned per keystroke.
- **Dependencies.** §4.6; K01 underpins G04 (editor/grid settings) and I01 (environment
  policy toggles).
- **Non-goals.** Sync/cloud settings, per-workspace profiles beyond connection scoping,
  theming engine/marketplace, localization framework.
- **Exit criteria.** All ten sections exist with persisted values; no dead settings port; no
  ad-hoc persisted flag outside the model or documented eframe keys.

### L. Release / Packaging / Telemetry / Diagnostics

- **Goal.** Ship the product reliably and diagnose user problems without leaking secrets.
- **User value.** Trust: signed builds, clear crash/diagnostic information, and privacy
  defaults.
- **Current implementation.** `docs/release/*` readiness/verification/checklist docs exist;
  packaging builds and packages in CI (final run `34860902181`); diagnostics domain exists with redaction
  (`domain/diagnostics.rs:14-154`, `redact_sensitive:139`); `tracing` initialization exists
  (`native-app/src/main.rs:119-125`); **no telemetry, no crash reporting, no diagnostic
  bundle export**; governance risks `R001` (name), `R-LICENSE` (license), `R003` (signing) and
  `R009` (SSH) remain
  open in `docs/release/risk-register.md`.
- **Missing functionality.** Reproducible release builds per platform, signing/notarization,
  versioning/update policy, a user-triggered diagnostics bundle (redacted logs + versions +
  capability snapshot), and a telemetry decision (default off).
- **Proposed architecture.** Diagnostics as a domain service that assembles a redacted bundle
  (`DiagnosticEvent` + `DiagnosticsSummary` + driver/capability snapshot) written to a
  user-chosen path; no automatic upload. Telemetry, if ever added, is opt-in, documented, and
  off by default with a visible Settings toggle.
- **UI surfaces.** Help/About with version + build info; Settings → Advanced → "Export
  diagnostics"; error surfaces gain "Copy diagnostics".
- **Provider differences.** None; capability snapshots record per-provider flags for support
  triage.
- **Safety model.** Diagnostics must be redacted by construction (reuse
  `redact_sensitive`), never include credentials, connection strings with passwords, bound
  values, or row data; export is an explicit user action to a user-chosen path.
- **Testing strategy.** Redaction tests with fixture secrets, bundle-content assertions
  (allowlist of fields), packaging smoke tests per platform, version-string consistency
  tests.
- **Runtime verification.** Produce a diagnostics bundle on a machine with a failing
  connection and verify no secret appears; install a release artifact per platform.
- **Performance concerns.** Bundle creation must be bounded (log rotation/size cap) and
  must not freeze the UI.
- **Dependencies.** Governance decisions (name/license/signing) are outside engineering;
  tracked in the risk register.
- **Non-goals.** Automatic crash upload, A/B analytics, usage telemetry by default, auto
  -update infrastructure before signing exists.
- **Exit criteria.** Release artifacts build and install on the supported platforms with
  recorded evidence; diagnostics bundle provably redacted; telemetry default-off documented.

---

## 6. Release Roadmap

### 6.1 Release rules

1. **v0.1 is frozen.** Only `docs/notes/V0_1_CLOSURE_PLAN.md` governs it. No Phase A–L work
   is admitted.
2. A release ships only complete milestones: a milestone is complete when the plan folder
   exists, P0=P1=0, and provider/runtime evidence is recorded separately for PostgreSQL and
   SQLite where applicable.
3. No release may contain a rail placeholder, a capability flag without code, or a feature
   whose provider behavior is "assumed".
4. Release scope is a hard cap. New ideas go to the next boundary or Later, not into the
   current release.
5. Each release records: scope, milestones, exit criteria, known limitations, and the
   provider matrix delta.

### 6.2 v0.1 — RC closure (unchanged, tracked elsewhere)

Runtime verification of active plans, native packaging, manual desktop smoke, governance
sign-off, tag. See `docs/notes/V0_1_CLOSURE_PLAN.md`. **Not part of this goal's plan.**

### 6.3 v0.2 — "Object workbench I + routines I" (proposed scope)

Thesis: the engine already has builders, introspection, and a safety layer; v0.2 turns them
into a workbench for the most-used object kinds and makes routines usable.

| Included | Why now |
|---|---|
| A01 Typed Object Mutation Framework | Every later object feature depends on it; without it each feature invents `execute_ddl` calls |
| A02 View & Materialized View CRUD | Builders exist; smallest end-to-end proof of the framework |
| A03 Index CRUD Wizard | Builders exist; high frequency in real work |
| A04 Constraint CRUD (PK/FK/Unique/Check) | Closes the "can't add a foreign key" gap that blocks real schema work |
| B01 Routine Domain Model & Capability Gating | Fixes the flag/enforcement gap that B02–B04 depend on |
| B02 Routine Explorer & Source View | Makes existing introspection actually usable |
| B03 Routine Source Editor (create/replace/drop) | Completes the routine lifecycle |
| G01 Palette & Quick Open Search Expansion | Cheap, high-visibility findability win with existing palette |
| K01 Settings Center | Every later feature needs settings; do it before the pile grows |

Explicitly **not** in v0.2: monitoring, import, users/roles UI, compare/migration, sequences/
types, execution forms for routines (B04), Search activity (G02).

Exit criteria for v0.2: the nine milestones are complete with provider evidence; Explorer can
create/alter/drop views, matviews (PG), indexes, and constraints with preview; routines can be
browsed, read, edited on PostgreSQL; palette finds columns/views/routines and executes the new
commands; Settings has the ten sections; v0.1 runtime-verified features show no regression.

### 6.4 v0.3 — "Admin + transfer + routines II" (proposed scope)

Thesis: the product becomes a database IDE rather than a query tool.

| Included | Why now |
|---|---|
| B04 Routine Execute Form & Result Rendering | The routine workbench is useless without execution |
| A05 Trigger Editor + enable/disable | Completes table-level object coverage; introspection already exists |
| A06 Sequence Management (PG) | Flag-without-code cleanup (C2); common in real schemas |
| A07 Type / Enum / Domain Management (PG) | Closes the last schema-authoring gap |
| A08 Schema & Database Object Management | Schema create/rename/drop is table-stakes for PG users |
| C01 Transfer Engine (streaming pipeline + job model) | Foundation for all import/export work |
| C02 CSV Import | The single most requested non-query task |
| C03 Excel Import | Completes the analyst workflow |
| C04 JSON Import | API-first workflows |
| C05 Export Workbench | Removes the lossy local CSV writer; wires the real engine |
| D01 MonitoringService Core | Unlocks monitoring + AI administration tools |
| D02 Monitoring Activity UI | Makes the service visible; removes the placeholder |
| E01 Security Workbench | Backend is done and idle; high perceived value |
| E02 Privilege Matrix | Completes the security story |

Exit criteria for v0.3: the Transfers and Monitor placeholders are **removed** (replaced by
real surfaces); CSV/Excel/JSON import works with mapping/preview/progress/cancel on both
providers; export writes through the backend engine; sessions/queries/cancel are live on PG
and correctly gated on SQLite; users/roles/grants are manageable on PG; routine execution
works with typed arguments.

### 6.5 v0.4 — "Depth + productivity" (proposed scope)

| Included | Why now |
|---|---|
| D03 Locks & Blocking Analysis | Needs D01/D02 and real lock fixtures |
| D04 Size, Statistics & Maintenance | Completes the admin surface with VACUUM/ANALYZE |
| E03 Membership & Password Management | Requires new backend methods; lower urgency |
| F01 Schema Compare UI | Existing diff backend becomes visible |
| F02 Diff Depth Expansion | Needed before any migration generation is trustworthy |
| F03 DDL Diff & Migration Generation | The migration product |
| F04 Migration Apply + Rollback Plan | Completes compare→apply; must follow A01 |
| G02 Search Activity + Unified Index | Graduates the palette into a real activity |
| G03 Workspace Organization (favorites/recent/pinned/snippets/scratch/tags) | Productivity layer |
| G04 Editor, Grid & Connection Customization | Consumes K01 |
| H01 AI Tool Surface v2 (read-only tools + contracts) | Only after D/F backends exist |
| I01 Connection Organization & Environment Labels | Environment safety is a prerequisite for unattended/agent operations |

Exit criteria for v0.4: monitoring depth and maintenance work on PG with strong
confirmations; schema compare produces an ordered, reviewable, dialect-correct migration with
rollback where available and refuses lossy cross-provider operations; search finds objects,
columns, queries, and commands; favorites/snippets/keybindings persist; environment labels are
visible on every target surface and drive confirmation strength.

### 6.6 Later (explicitly unscheduled)

- H02 Schema/Migration Assistant, H03 Administration Assistant (slow query, index
  suggestion, session analysis) — need v0.4 backends and their own safety review.
- J01 Task/Job Scheduler.
- L01 Release/Diagnostics/Telemetry completion (partly governed by open risk decisions).
- Data Compare depth (PK-matched, sampled), database-to-database transfer, ER design mode,
  additional drivers, MCP, replication/pool/extension surfaces.

### 6.7 Packing rules

- Never more than ~10 milestones per release; each release must leave every rail surface it
  touches in a non-placeholder state.
- A milestone may move earlier only if its prerequisites are complete; it may not move later
  to make room for a new idea of equal priority.
- Any milestone split must keep its ID and add a suffix (`A04a`), never silently renumber.

---

## 7. Milestone Breakdown (39 milestones)

### 7.1 Milestone record schema

Every milestone carries: **ID, Title, Scope, Prerequisites, Implementation surfaces,
Acceptance criteria, Verification, Priority (P0–P3), Release.** Priority semantics follow
`PRODUCT_CAPABILITY_MATRIX.md`: `P0` release blocker, `P1` core product capability, `P2`
important productivity/polish, `P3` advanced/later. Roadmap milestones are P1–P3; `P0` is
reserved for v0.1 closure work.

Each milestone is independently implementable: it has a single coherent deliverable, a
prerequisite list that is either already done or earlier in the same phase, and its own
plan folder (`docs/plans/active/<slug>/{PLAN,CHECKLIST,FINDINGS,VERIFICATION}.md`).

### 7.2 Milestone table

| ID | Title | Release | Pri | Prerequisites | Scope | Implementation surfaces | Acceptance criteria | Verification |
|---|---|---|---|---|---|---|---|---|
| A01 | Typed Object Mutation Framework | v0.2 | P1 | §9 capability enforcement design agreed | `ObjectAction`/`ObjectMutationRequest`/`ObjectMutationPreview`/`ObjectMutationResult`; `ObjectMutationService`; capability + policy gate before SQL; DDL preview payload; operation class; cache invalidation; audit record | `crates/core/src/domain/object_action.rs` (new), `application/object_mutation_service.rs` (new), `application/ddl_builder.rs` (extend), `ports/` (extend `DbConnector` usage), `runtime/src/{api,worker}.rs`, `crates/ui/src/runtime.rs`, `crates/ui/src/components/` (preview dialog) | One real operation per provider runs end-to-end through the framework (PG + SQLite index create) with preview, policy gate, capability gate, and refreshed Explorer; zero SQL built in `crates/ui`; unsupported operation returns `DbError::Unsupported` with reason | Core unit tests for request/preview/result; service tests with `MockDbConnector` (policy reject, capability reject, cache invalidation); SQLite integration; PG live (`--ignored` suite); native UI screenshot of preview + confirmation |
| A02 | View & Materialized View CRUD | v0.2 | P1 | A01 | Create/replace/drop view via typed service; view columns introspection; PG materialized views (list/refresh/browse/drop); view DDL in `DDLViewer` | `postgres/introspect.rs` (view columns, `pg_matviews`), `infrastructure/sqlite/introspect.rs` (view columns), `core/domain/schema.rs` (`MaterializedView`), `ddl_builder.rs:114-128`, `ui/src/schema_object_view.rs`, explorer folders | View create/edit/drop on both providers with preview; matview list + refresh + drop on PG; SQLite shows matviews disabled with reason; after apply, Explorer and DDL tab reflect the new state without app restart | Builder tests; introspection tests (view columns + matview catalog) per provider; SQLite round-trip (create→introspect→replace→drop); PG live round-trip incl. `REFRESH MATERIALIZED VIEW`; UI preview test |
| A03 | Index CRUD Wizard | v0.2 | P1 | A01 | Typed create/drop index (columns, order, unique, method, include columns, partial predicate where supported); drop+recreate for edits with explicit warning; index detail edit form | `ddl_builder.rs:130-158`, `application/object_mutation_service.rs`, `ui/src/table_metadata_view.rs:290-486`, `explorer_details.rs` | Index created/dropped from the Indexes tab and Explorer context menu with generated SQL preview; unsupported options (GIN/GiST on SQLite, INCLUDE, partial) are disabled with reason; index list refreshes from real introspection | Builder tests per dialect; SQLite integration create/drop; PG live live-index lifecycle test (already exists as a base in `tests/pg_integration.rs:342`); UI test for disabled options |
| A04 | Constraint CRUD (PK/FK/Unique/Check) | v0.2 | P1 | A01 | Add/drop PK, FK (incl. composite), Unique, Check; multi-column unique fidelity via `pg_constraint contype='u'`; constraint editor form with referenced-table picker | `postgres/introspect.rs` (add `contype='u'` query), `ddl_builder.rs` (new constraint builders), `object_mutation_service.rs`, `ui/src/table_metadata_view.rs:487-846` | Add/drop each constraint kind on PG; SQLite add/drop where the engine allows (FK requires table rebuild → explicit warning, or capability-disabled); FK picker lists real tables/columns from introspection; DDL preview matches what introspection will report afterward | Builder tests; PG live add/drop per constraint kind + re-introspection equality; SQLite capability-boundary tests; UI form tests |
| A05 | Trigger Editor + enable/disable | v0.3 | P2 | A01 | Create/alter/drop trigger; PG enable/disable/`ENABLE REPLICA|ALWAYS`; SQLite drop/create only; trigger function binding (PG) | `ddl_builder.rs:159-183`, `object_mutation_service.rs`, `postgres/connector.rs` (trigger state), `ui/src/explorer_folders.rs:252-291`, `table_metadata_view.rs` | Trigger lifecycle works on both providers with preview; PG enable/disable reflected in Explorer and in `information_schema.triggers`; SQLite shows no enable/disable control | Builder tests; SQLite trigger lifecycle tests (existing `schema_triggers_runtime_verification.rs` extended); PG live enable/disable evidence (currently pending in STATUS S4) |
| A06 | Sequence Management (PG) | v0.3 | P2 | A01 | `pg_sequences`/`information_schema.sequences` introspection + `Sequence` domain; create/alter (increment/min/max/cache/cycle/owned-by)/drop; dependency display | `postgres/introspect.rs`, `core/domain/schema.rs` (`Sequence`), `IntrospectResult`, `ddl_builder.rs`, Explorer sequences folder, `object_mutation_service.rs` | Sequences appear as first-class Explorer nodes with properties; create/alter/drop works with preview; the `sequences` capability flag is now backed by implementation; SQLite keeps the flag false and hides the folder | Introspection unit tests; PG live create/alter/drop + `nextval` behavior; capability-flag test asserting flag ↔ implementation for both drivers |
| A07 | Type / Enum / Domain Management (PG) | v0.3 | P2 | A01 | `pg_type`/`pg_enum` introspection for enum/domain/composite; create/alter (add value, rename value)/drop; usage sites listing; array element type parsing for columns | `postgres/introspect.rs`, `core/domain/schema.rs` (`EnumType`, `DomainType`), Explorer types folder, `object_mutation_service.rs` | Enum list with labels and usage; create enum, add value, drop enum (with dependency warning) via preview; `enum_types` capability flag backed by implementation; SQLite hides the folder with reason | Introspection tests; PG live enum/domain lifecycle incl. usage detection; flag/implementation test |
| A08 | Schema & Database Object Management | v0.3 | P2 | A01 | Create/drop/rename schema (PG) with owner and default privileges; database list/create/drop where connected user may (guarded, PG); SQLite: no-op gated | `postgres/connector.rs:648-666` (`rename_schema_object`) promoted into a core service, `ddl_builder.rs`, Explorer schema context menu, `object_mutation_service.rs` | Schema create/rename/drop from Explorer and from the schema header with preview and destructive confirmation; PG rename no longer bypasses the application layer; SQLite shows no schema controls | Core service tests (promotion of the inherent PG method); PG live create/rename/drop; SQLite gating tests; UI test for confirmation strength |
| B01 | Routine Domain Model & Capability Gating | v0.2 | P1 | A01 (gate reuse) | `Routine` aggregate (function/procedure, signature, parameters with mode/type/default, return type, language, volatility, security definer, source); `RoutinePort`; capability enforcement for routines in the backend | `crates/core/src/domain/schema.rs` / new `routine.rs`, `ports/`, `postgres/introspect.rs` (parameter metadata via `pg_proc`/`pg_get_function_arguments`), `sqlite/introspect.rs` (unchanged empty + reason), `runtime/src/api.rs` | `Routine` replaces the `Function` stringly-typed model without breaking the Explorer; backend rejects routine operations when the provider lacks support; procedure vs function distinguished by type, not string compare | Domain tests (signature/overload identity, parameter modes); introspection tests with a multi-overload fixture; SQLite capability-boundary test; API mapping tests |
| B02 | Routine Explorer & Source View | v0.2 | P1 | B01 | Explorer Functions/Procedures folders with signatures and overload grouping; search/filter; Definition/DDL/Source tabs; dependency listing for routines | `ui/src/explorer_folders.rs`, `ui/src/schema_object_view.rs`, `core/application/schema_service.rs` (targeted routine fetch), `ObjectTabs`/`ObjectList` components | Routines browse with signatures; selecting one shows source + DDL + parameters without a full schema re-introspection; overloads are distinguishable; empty and error states defined; SQLite hides the folders | UI tests (list rendering, overload grouping, empty state); service test for targeted routine fetch; native screenshot per provider state |
| B03 | Routine Source Editor (create/replace/drop) | v0.2 | P1 | B01, B02, A01 | Create/replace/drop routine through `ObjectMutationService`; editor with SQL/PL-pgSQL highlighting; preview + confirmation; dependency-aware drop warning; template insertion | `ddl_builder.rs` (new routine builders), `object_mutation_service.rs`, `ui/src/schema_object_view.rs`, `crates/ui/src/editor/*` | `CREATE OR REPLACE FUNCTION` and `DROP` work end-to-end on PG with preview and confirmation; SQLite disabled with reason; editor reuses the existing `CodeEditor` (no new editor); drop warns when dependent objects exist | Builder SQL tests; service tests; PG live create→replace→drop; UI test for preview + confirmation + disabled state |
| B04 | Routine Execute Form & Result Rendering | v0.3 | P1 | B03 | Typed argument form from parameter metadata: input values with NULL toggle, default handling, type-aware widgets; execute function/`CALL` procedure; output rendering (result grid, notices/messages, errors, affected rows) | `runtime/src/api.rs` (`QueryParam` binding path), `ui/src/schema_object_view.rs` (Execute tab), `ui/src/query_view.rs` output pane reuse, `components/form.rs` | Executing a routine with typed arguments returns the correct result set / notices on PG; `CALL` is Destructive-class and therefore always confirmed; NULL and default arguments are expressible; failing execution shows a structured error, not a raw string | Form binding tests; service tests for argument typing (no interpolation); PG live execution of function + procedure with output; UI test for confirmation and result rendering |
| C01 | Transfer Engine (streaming pipeline + job model) | v0.3 | P1 | §4.6 meta store pattern | `TransferService` with streaming `TransferSource`/`TransferSink`; chunked batched writes with transaction policy; `TransferJob` records + progress events + cancellation; error-row collection; bounded memory guarantees | `crates/core/src/application/transfer_*.rs` (new), `ports/transfer.rs` (new), `infrastructure/meta/*` (job table + migration), `runtime/src/worker.rs` (job routing + cancel), `crates/ui/src/runtime.rs` | A 100k-row transfer completes with bounded memory; a cancelled transfer leaves a deterministic, documented state; progress events are rate-limited; job records survive restart and show final status | Unit tests (chunking, transaction policy, error rows); streaming memory test; cancellation test at fixed offsets; meta-store migration test; SQLite fixture end-to-end |
| C02 | CSV Import | v0.3 | P1 | C01 | Import wizard: file pick, encoding + delimiter + quote detection (overridable), header row, preview grid, column mapping (source → target, skip, create-table option), type inference + coercion with explicit failures, null mapping, default handling, batch size, transaction mode, conflict mode (insert/skip/update-by-key), progress, cancel, error-row report | New reader dependency (`csv` read side), `application/transfer_import.rs`, `inject`/`TableDataService` write path or dedicated sink, `ui/src/` new `Transfer` tab (`components/*`) | CSV import into an existing table works on both providers with preview before any write; every coercion failure is reported per row with line numbers; conflict mode behaves as documented; a cancelled import reports exactly how many rows committed | Golden-file tests (BOM, CRLF, quoted newlines, escapes, duplicate headers, empty file, wrong types); service tests for mapping/conflict/transaction modes; SQLite + PG live imports; UI wizard tests; 100k-row runtime evidence |
| C03 | Excel Import | v0.3 | P2 | C02 | `.xlsx` read: sheet selection, header detection, type inference from cell types, same mapping/conflict/transaction pipeline as CSV; explicit handling of dates/formulas-cached-values/merged cells | New reader dependency, `application/transfer_import.rs` (shared pipeline), wizard step for sheets | Excel import shares ≥90% of the CSV pipeline; sheet and range selection work; date/formula cells either import correctly or are reported, never silently wrong; errors identify sheet+cell | Reader unit tests (types, dates, formulas, merged cells); pipeline reuse test; live import per provider; UI wizard test |
| C04 | JSON Import | v0.3 | P2 | C02 | JSON array / NDJSON input; path selection for nested arrays; field → column mapping; nested value policy (stringify/reject/extract path); same mapping/conflict/transaction pipeline | New reader dependency, `application/transfer_import.rs` (shared pipeline), wizard step for JSON options | Both JSON array and NDJSON import; nested objects handled by an explicit documented policy; malformed JSON fails with the byte offset; pipeline shared with CSV/Excel | Reader unit tests (NDJSON, nesting, unicode, malformed); pipeline reuse test; live import per provider; UI test |
| C05 | Export Workbench | v0.3 | P1 | C01 | Wire backend `ExportService` (CSV/JSON/XLSX) to the UI for result/table/selection/query sources; remove the local unquoted writer; add SQL `INSERT` generation and PostgreSQL `COPY` export; destination + options + overwrite policy + progress | `application/export_service.rs` (extend with `export_sql`/`copy`), `runtime/src/{api,worker}.rs`, `crates/ui/src/query_view.rs:1357-1389` (delete local writer), new Export UI, Transfers activity (placeholder removed) | Export writes correctly quoted CSV/TSV and lossless XLSX/JSON from a real source; the local writer is gone; SQL INSERT export produces replayable statements; `COPY` is offered only on PostgreSQL; Transfers activity replaces the `COMING SOON` placeholder | Export-service tests extended (`export_sql`, quoting, escaping, delimiters); round-trip test (export → import → compare) per provider; lossless big-integer test already exists (retain); UI test for format/capability gating |
| D01 | MonitoringService Core | v0.3 | P1 | C1 cancel decision (source truth) | `AdminPort` + `MonitoringService`: sessions, active queries (duration/wait/state), cancel query, terminate session, database/table sizes, server information; PG implementations; SQLite meaningful-subset implementation | `crates/core/src/ports/admin.rs` (new), `core/src/application/monitoring_service.rs` (new), `infrastructure/postgres/admin.rs` (new: `pg_stat_activity`, `pg_stat_database`, `pg_total_relation_size`, `pg_terminate_backend`, `pg_cancel_backend`, `version()`), `infrastructure/sqlite/admin.rs` (new: file size, page stats), `runtime/src/{api,worker}.rs` | Sessions and active queries list accurately on PG with a real fixture; cancel from the service cancels the same query the query editor's Stop targets; terminate works; sizes match `psql`; SQLite returns only meaningful metrics; everything is read-only by default | Query-accuracy tests against a seeded fixture; mock-port tests for classification/confirmation; permission-denied path test (non-`pg_monitor` role); PG live tests; SQLite metric tests |
| D02 | Monitoring Activity UI | v0.3 | P1 | D01 | Monitoring activity with Overview/Sessions/Active Queries groups; polling with pause/resume; virtualization and row caps; row actions (Cancel Query, Terminate Session, Explain Query, Open SQL) with confirmations; remove the Monitor placeholder | `ui/src/navigation_view.rs:616-621` (replace placeholder), new `ui/src/monitoring_view.rs` + `monitoring/` module, `components/workspace.rs::{ActivityBar, StatusBar, ConnectionHealth}` adoption, `components/ConfirmationDialog` | Monitoring surface is real: overview + session/query lists refresh on an interval, actions confirm with target/production awareness; stale responses never overwrite newer data; SQLite shows the meaningful subset with a reason for the rest; zero `COMING SOON` text remains in the rail | UI tests (polling lifecycle, stale-response guard, confirmation copy); native screenshots per provider; manual live PG walkthrough (long query → cancel → observe) |
| D03 | Locks & Blocking Analysis | v0.4 | P2 | D02 | Lock listing (`pg_locks` joined to `pg_stat_activity`), blocking/blocked graph, wait duration, lock mode, relation, and a "terminate blocker" action with strong confirmation | `infrastructure/postgres/admin.rs`, `core/application/monitoring_service.rs`, `ui/src/monitoring/locks.rs`, `components/tree.rs` (blocking tree) | Two-session lock conflict is reproduced and displayed as a blocker→blocked relationship; terminate blocker is confirmed and its effect observable; SQLite has no lock surface (reason shown) | PG fixture test creating a real lock conflict; service tests for graph construction; live walkthrough evidence with screenshots |
| D04 | Size, Statistics & Maintenance | v0.4 | P2 | D02 | Database/table/index size, row estimates vs exact counts with an explicit distinction, seq/index scan statistics, `VACUUM`, `VACUUM ANALYZE`, `ANALYZE`, `PRAGMA optimize` (SQLite); maintenance history in the activity log | `infrastructure/postgres/admin.rs`, `infrastructure/sqlite/admin.rs`, `core/application/monitoring_service.rs`, `ui/src/monitoring/stats.rs` + maintenance actions with confirmation | Sizes and scan stats display with a clear "estimate vs exact" distinction; maintenance actions run with confirmation and report duration/result; `VACUUM FULL` carries a stronger (lock-implying) warning; SQLite offers optimize/vacuum/analyze only | PG live size/stat comparisons against `psql`; maintenance action tests; SQLite `PRAGMA optimize` test; UI confirmation tests |
| E01 | Security Workbench (roles, users) | v0.3 | P2 | §4.5 components; existing `UserService` | Users/Roles pages: role list with attributes, role details (`pg_roles`), create/alter/drop role, role DDL view, owned-object summary; Security node in Explorer | `ui` new `security/` module + Explorer Security node, `runtime/src/api.rs` extensions, `core/application/user_service.rs` (alter role), `infrastructure/postgres/user_manager.rs` | Role list/detail/create/alter/drop work on PG with preview and confirmation; the DDL view shows the statements that the UI would run; SQLite shows the Security node disabled with a reason; no password is ever rendered | Service tests for new operations; PG live walkthrough; SQLite gating test; UI tests for confirmation + disabled state; secret-redaction review |
| E02 | Privilege Matrix (grants) | v0.3 | P2 | E01 | Table/schema/sequence grant and revoke with an allowlisted privilege set; privilege matrix view (grants × objects) with filter; generate-SQL view; "who can access this object" lookup | `infrastructure/postgres/user_manager.rs:113-180` (extend to schema grants + reverse lookup), `core/application/user_service.rs`, `ui/src/security/privileges.rs` | Grant/revoke on table and schema works with preview; the matrix reflects real `information_schema` state after apply; privilege names come only from the allowlist; SQLite has no surface | SQL-generation tests incl. injection attempts; PG live grant→verify→revoke; UI matrix tests; allowlist rejection tests |
| E03 | Membership & Password Management | v0.4 | P3 | E02 | Role memberships (grant/revoke role), membership tree view, `ALTER ROLE ... PASSWORD` with typed confirmation and no echo, role rename, role attribute editing (login/superuser/createdb/createrole/replication/bypassrls) | `core/ports/user_manager.rs` (new methods), `infrastructure/postgres/user_manager.rs`, `core/application/user_service.rs`, `ui/src/security/memberships.rs` | Memberships displayed as a tree and editable with preview; password change requires typed confirmation, never appears in logs/UI/history, and is verified by re-authentication; attribute changes warn for superuser/replication | Port/service tests; PG live membership + password change (using a throwaway role); redaction assertions; UI tests |
| F01 | Schema Compare UI | v0.4 | P2 | Existing diff backend; §4.5 `DiffViewer` | Compare workspace tab: source/target pickers (connection, database, schema, saved snapshot), filterable diff tree with Added/Removed/Changed, per-object property diff, raw diff JSON export; snapshot persistence | `runtime/src/api.rs` (`diff_schemas` wiring), `core/application/schema_diff.rs` (snapshot input), `infrastructure/meta/*` (snapshot table + migration), `ui/src/compare/`, `components/diff.rs` (wire the existing widget) | Diff renders for PG↔PG and SQLite↔SQLite with object-level Added/Removed/Changed and property-level detail; saved snapshots allow comparison against an offline schema; cross-provider comparison is offered as structural-only; empty and error states defined | Diff unit tests (fixture pairs: identical/additive/destructive/rename/type-change); snapshot round-trip test; UI tests for tree/filters; native screenshots |
| F02 | Diff Depth Expansion | v0.4 | P2 | F01 | Extend diff to PK/FK/unique/check, views, matviews, triggers, routines, sequences, types, and column attributes (nullability/default/collation); deterministic ordering; rename heuristics clearly labeled as suggestions | `core/application/schema_diff.rs` (comparators), `core/domain/cross_connection.rs` (diff types), introspect results per provider | Diff covers every object kind in the target matrix; results are deterministic (existing ordering rule retained and extended); rename suggestions are labeled and never auto-applied; nil-diff produces an explicit "no differences" state | Comparator unit tests per object kind; determinism test (shuffled input); provider fixture tests; cross-provider refusal test |
| F03 | DDL Diff & Migration Generation | v0.4 | P2 | F02, A01 | `MigrationPlanner`: diff → ordered `MigrationOperation` records with per-dialect SQL, operation class, and optional inverse; dependency-aware order (type → table → constraint → index → FK → view/routine); warnings (data loss, lock, unsupported); plan export as SQL file | `core/application/migration_planner.rs` (new), `ddl_builder.rs` reuse, `ui/src/compare/migration.rs`, `components/code.rs` (script view) | A diff produces an ordered, reviewable plan whose SQL is dialect-correct for the target provider; destructive and lossy operations are flagged; unsupported operations are reported as "cannot migrate" rather than dropped; plan SQL is copyable/exportable without applying | Planner ordering tests; dialect-correctness tests; lossy-operation refusal tests; end-to-end fixture diff → plan → apply on SQLite; PG live plan generation |
| F04 | Migration Apply + Rollback Plan | v0.4 | P2 | F03 | Apply the ordered plan inside a transaction where supported, with per-operation progress, stop-on-error, and post-apply verification (re-introspect + re-diff); generate a rollback script from operation inverses when available; explicit "no rollback" warning otherwise | `core/application/object_mutation_service.rs` (`execute_ddl_batch` reuse), `runtime/src/worker.rs` (progress + cancel), `ui/src/compare/apply.rs`, `components/ConfirmationDialog` | Apply is never automatic: the user reviews the plan, confirms, and sees per-operation results; a destructive migration cannot be applied without the dedicated confirmation; after apply, a re-diff shows the expected end state; rollback script executes successfully in the fixture where inverses exist | Apply/rollback integration tests per provider; partial-failure test (operation 3 of 5 fails → deterministic state + report); destructive-confirmation test; live PG walkthrough |
| G01 | Palette & Quick Open Search Expansion | v0.2 | P1 | none | Palette index expansion: columns/tables of the active connection (bounded), views, routines, saved-query SQL text, history SQL text, commands (new object/transfer/monitoring commands gated by capability), settings entries; grouping + keyboard navigation; result-type icons | `ui/src/palette_view.rs`, `ui/src/app.rs:131-159` (`PaletteAction`), `core/application/schema_service.rs` (cached lookup), `components/input.rs` (`SearchInput`) | `⌘K` finds a table, a column, a view, a routine, a saved query by SQL text, and a history query, and executes the resulting action; results are grouped and keyboard-navigable; no secrets or row data are indexed; search never blocks the frame | Unit tests for index/ranking/bounding; command coverage test (every `PaletteAction` reachable); no-secret assertion test; UI keyboard-only test |
| G02 | Search Activity + Unified Index | v0.4 | P2 | G01, §4.6 | First-class Search activity on a unified index (connections, databases, schemas, tables, columns, views, routines, sequences, types, saved queries, history, commands, agent actions, settings); incremental population from introspection + schema-change events; filters (favorites/recent/pinned/type/connection); preview pane with actions | `core/application/search_service.rs` (new) + `ports/search_index.rs` (new), `infrastructure/meta/search_index_repo.rs` + migration, `ui/src/search_view.rs` (new activity), `crates/ui/src/app.rs` (`Activity::Search`) | Search returns objects and commands across connections with type filters and keyboard navigation; the index updates after DDL without a manual rebuild; index provably contains no credentials or row data; activity replaces nothing else (no placeholder pattern) | Index tests (insert/update/delete/invalidate/no-secrets); ranking tests; migration test; UI tests (filters, keyboard, empty/error); native walkthrough screenshots |
| G03 | Workspace Organization (favorites, recent, pinned, snippets, scratch, tags) | v0.4 | P2 | G02 (index reuse) | Favorites/pinned/recent for objects, connections, and queries; snippet library with variables and folders; scratch SQL documents; saved-query tags and move-between-folders; recents surfaced in Search and Queries | `core/domain/` (favorites/snippet/scratch types), `ports/` + `infrastructure/meta/*` (tables + migration), `ui/src/navigation_view.rs` (Queries sections), context menus across Explorer/grid/editor | Favoriting a table/query/connection persists and is filterable; snippets insert with variable prompts and always bind parameters; scratch documents persist like query documents; tags round-trip; no secret or result data persists in these entities | Meta-store migration + repo tests; UI tests for each entity lifecycle; parameter-binding test for snippet variables; restart persistence test |
| G04 | Editor, Grid & Connection Customization | v0.4 | P2 | K01 | Migrate ad-hoc editor flags into Settings; grid defaults (row height, null display, copy format, date/number formats); connection defaults (timeout, max rows, default schema, read-only); keybinding editor with conflict detection and cheatsheet; query parameters dialog | `core/domain/history.rs` (`Settings`), `ports/settings_repository.rs` (or its replacement), `ui/src/navigation_view.rs:940-1062` (settings pages), `ui/src/events.rs:1102-1160` (keybinding resolution), `ui/src/query_view.rs` (params UI) | Every grid/editor/connection default is a persisted setting with a default and validation; keybindings are remappable, conflicts are reported, and defaults are restorable; the query parameters dialog binds typed values (no interpolation) and unblocks the binary-cell hint | Settings tests (round-trip, defaults, validation); keybinding conflict tests; parameter-binding tests; UI tests; restart test |
| H01 | AI Tool Surface v2 (read-only tools + contracts) | v0.4 | P2 | D01–D04, F01–F03, G02 | Documented + implemented read-only tools: `list_sessions`, `list_active_queries`, `inspect_locks`, `explain_query_plan` (with stats context), `compare_schemas` (read-only diff), plus the contract for future mutating tools (`generate_migration`, `suggest_index`) with risk class + confirmation requirement | `runtime/src/agent.rs:565-634` (tool definitions), `runtime/src/agent_executor.rs` (implementations over `MonitoringApi`/`CompareApi`), `core/domain/agent.rs` (risk classes), `ui/src/agent_view.rs` | New tools are registered with declared risk classes and bounded outputs; read-only tools may auto-run only under the existing explicit setting; mutating tools either do not exist yet or go through preview + confirmation with execution-time re-classification; SQLite reports unavailable tools instead of guessing | Tool-contract tests; classification tests; confirmation/replay tests mirroring `agent_orchestrator.rs`; provider-gating tests; live PG E2E per read-only tool |
| H02 | Schema / Migration Assistant | Later | P3 | H01, F04 | Agent workflow: given a diff, propose a migration plan, explain each operation, flag risks, and hand the reviewed plan to the F04 apply flow (never applying directly) | `runtime/src/agent*.rs`, `ui/src/compare/*` hand-off, `components/diff.rs` rendering | Assistant produces a plan the user can review and apply through the existing gated path; no path exists from assistant output directly to execution; refusal behavior is explicit when the backend cannot diff | Workflow tests (plan proposal → user review → apply), refusal tests, live E2E with confirmation evidence |
| H03 | Administration Assistant (slow query, index suggestion, sessions) | Later | P3 | H01, D03, D04 | Agent tools: analyze a slow query (plan + stats + lock context), suggest indexes with explicit preview + apply confirmation, summarize session/lock state | `runtime/src/agent*.rs`, `core/application/monitoring_service.rs`, `ui/src/monitoring/*` | Analysis tools are read-only and bounded; index suggestion produces a DDL preview that must be explicitly confirmed and then applied through A01/F04; no autonomous DDL under any mode | Analysis accuracy tests on fixtures; suggestion→confirmation→apply workflow test; capability-gated SQLite behavior; live E2E |
| I01 | Connection Organization & Environment Labels | v0.4 | P2 | §4.6, K01 | Connection folders/tags/favorites/color surfaced (fix the DTO boundary); `Environment` enum (Production/Staging/Development) on `ConnectionConfig`; environment badge on connection rows and every workspace `ObjectHeader`; profile export/import (versioned JSON, no secrets); `allow_ddl`/`allow_destructive` per-connection flags with real producers | `core/domain/connection.rs`, `runtime/src/api.rs` (`ConnectionSummary` DTO), `native-app/src/translate.rs:41-42`, `ui/src/connection_view.rs`, `ui/src/explorer_view.rs`, `ui/src/table_view.rs` (header badge), `core/application/*` (policy derivation) | Folders/tags/favorites/color persist and render; every target surface shows the environment; Production destructive operations use the stronger confirmation; profile export contains no secret (test) and imports with validation; `allow_ddl`/`allow_destructive` demonstrably gate execution | Domain tests (environment parse, validation, profile no-secret); service tests for policy derivation; UI tests for badges/confirmation variants; round-trip import/export test |
| J01 | Task / Job Scheduler | Later | P3 | C01 job model, §4.6, I01 (environment policy) | Task entity (SQL/script, target connection, schedule, timeout, failure policy, enable flag), persisted + migrated; single-concurrency executor owned by the runtime; run history with status/duration/error; Tasks/Jobs activity UI | `core/domain/task.rs`, `ports/task_repository.rs`, `infrastructure/meta/task_repo.rs` + migration, `runtime/src/worker.rs` (scheduler), `ui/src/tasks/` (new activity) | Tasks run on schedule with recorded history; overlapping runs are prevented; read-only connections cannot host tasks; a Production task requires explicit acknowledgement of unattended execution; cancelling a run works | Scheduler unit tests (no overlap, missed runs, cancel); repo/migration tests; SQLite fixture execution; UI tests; safety tests (read-only, production acknowledgement) |
| K01 | Settings Center | v0.2 | P2 | §4.6 | Ten settings sections with a single persisted model, validation, search, reset-to-default, and scope (global vs per-connection); decision recorded for the currently unused `SettingsRepository`; migration of ad-hoc flags (prediction mode, panel widths, theme) into the model where they are behavior, not layout | `core/domain/history.rs` (`Settings`), `ports/settings_repository.rs` (reuse/delete decision), `infrastructure/meta/settings_repo.rs`, `runtime/src/*` (settings load/push), `ui/src/navigation_view.rs:940-1062`, `ui/src/app_state.rs` | All ten sections render and persist; a changed setting survives restart; per-connection settings apply to that connection only; reset-to-default works; no dead settings port remains; the settings model is the only home for behavior settings | Settings round-trip/migration/validation tests; UI tests for each section; restart persistence test; dead-key audit test (no persisted-but-unread key) |
| L01 | Release, Diagnostics & Telemetry | Later | P3 | Governance decisions (name/license/signing) | Redacted diagnostics bundle (versions, capability snapshot, redacted logs, recent errors) with user-chosen export path; About/Help surface; packaging/signing/notarization process recorded; telemetry decision documented as default-off | `core/domain/diagnostics.rs` (extend), `runtime/src/*` (bundle assembly), `ui/src/` (Settings → Advanced, About), `.github/workflows/` release pipeline, `docs/release/*` | A diagnostics bundle can be produced from the UI and contains no credentials/bound values/row data (allowlist-tested); release artifacts build and install on the supported platforms with recorded evidence; telemetry is absent or explicitly opt-in and documented | Redaction tests with fixture secrets; bundle-field allowlist test; per-platform artifact install smoke; version-consistency test |

### 7.3 Milestone counts

| Phase | IDs | Count | Releases |
|---|---|---:|---|
| A — Object CRUD | A01–A08 | 8 | v0.2 (A01–A04), v0.3 (A05–A08) |
| B — Routines | B01–B04 | 4 | v0.2 (B01–B03), v0.3 (B04) |
| C — Transfer | C01–C05 | 5 | v0.3 |
| D — Monitoring | D01–D04 | 4 | v0.3 (D01–D02), v0.4 (D03–D04) |
| E — Security | E01–E03 | 3 | v0.3 (E01–E02), v0.4 (E03) |
| F — Compare/Migration | F01–F04 | 4 | v0.4 |
| G — Productivity | G01–G04 | 4 | v0.2 (G01), v0.4 (G02–G04) |
| H — Advanced AI | H01–H03 | 3 | v0.4 (H01), Later (H02–H03) |
| I — Connections/Environment | I01 | 1 | v0.4 |
| J — Tasks/Jobs | J01 | 1 | Later |
| K — Settings | K01 | 1 | v0.2 |
| L — Release/Diagnostics | L01 | 1 | Later |
| **Total** | — | **39** | v0.2: 9 · v0.3: 14 · v0.4: 12 · Later: 4 |

Independent-implementability rules that keep this list honest:

- No milestone requires more than one new service (A01 introduces one; A06/A07/A08 extend it).
- Every milestone has at least one acceptance criterion that is observable without reading code.
- Where a milestone cannot complete because a provider lacks the capability, the milestone's
  definition of done is "hidden/disabled with reason + test", not "feature implemented".
- Milestones B04, D03, D04, E03, F02–F04, G02–G04, H01, I01 may be re-ordered within their
  phase but not across prerequisites.

---

## 8. Dependency Graph

### 8.1 Graph

```text
A01 Typed Object Mutation Framework
 ├─→ A02 View/MatView CRUD ─→ (F02 diff depth: views)
 ├─→ A03 Index CRUD ────────→ (F03 migration: indexes)
 ├─→ A04 Constraint CRUD ───→ (F03 migration: constraints)
 ├─→ A05 Trigger editor ────→ (F02 diff depth: triggers)
 ├─→ A06 Sequences ─────────→ (F02/F03: sequences)
 ├─→ A07 Types/Enums ───────→ (F03 order: type before table)
 ├─→ A08 Schema/Database ───→ (F03/F04 apply into a schema)
 ├─→ B03 Routine editor ────→ (F02 diff depth: routines)
 ├─→ F04 Migration apply
 └─→ H02/H03 AI mutation output

B01 Routine domain model & gating
 └─→ B02 Routine explorer/source ─→ B03 Routine editor ─→ B04 Execute form
                                       │
                                       └─→ H01 (routine tools, later)

C01 Transfer engine (streaming + job model)
 ├─→ C02 CSV import ─→ C03 Excel import
 │                  └─→ C04 JSON import
 ├─→ C05 Export workbench
 └─→ J01 Task/job scheduler (job state model)

D01 MonitoringService core ─→ D02 Monitoring activity ─→ D03 Locks
                                     │                    └─→ D04 Sizes/stats/maintenance
                                     └─→ H01/H03 admin tools

E01 Security workbench ─→ E02 Privilege matrix ─→ E03 Membership/password

F01 Schema compare UI ─→ F02 Diff depth ─→ F03 Migration generation ─→ F04 Apply + rollback
                                                                          └─→ H02 Migration assistant

G01 Palette expansion ─→ G02 Search activity + unified index ─→ G03 Workspace organization
                                                          └─→ H01 (agent context lookup)

K01 Settings center ─→ G04 Editor/grid/connection customization
                    └─→ I01 Environment labels & scope

I01 Environment labels + connection flags ─→ §10 policy strength ─→ (D destructive actions,
                                                                     F apply, J01 scheduling)

L01 Release/diagnostics: independent; gated by governance decisions (R001/R004/R003/R009)
```

### 8.2 Critical path (longest chain to "full product")

```text
A01 → A02/A03/A04 → F01 → F02 → F03 → F04 → H02
A01 → B01 → B03 → B04                      (parallel, comparable length)
C01 → C02 → (C03/C04/C05)
D01 → D02 → D04 → H03
```

A01 is the single highest-leverage milestone in the roadmap: A02–A08, B03, F04, and every
AI-generated-DDL path depend on it. K01 is the second-highest-leverage *enabler* (settings +
scope for I01/G04), while D01 and C01 are the highest-leverage **new** backends.

### 8.3 Parallel tracks

| Track | Milestones | Can run concurrently with |
|---|---|---|
| Object track | A01→A04, then A05–A08 | Everything after A01 |
| Routine track | B01→B04 | A05–A08, C, D, E |
| Transfer track | C01→C05 | A05–A08, B04, D, E |
| Monitoring track | D01→D04 | A, B, C, E |
| Security track | E01–E03 | A, B, C, D |
| Compare track | F01→F04 | (needs A01 for apply) D, E, G |
| Productivity track | G01, G02→G04 | Everywhere (G01 has no prerequisites) |
| AI track | H01→H03 | Last; needs D and F backends |

Constraint: at most two "backend-new" tracks in flight at once (C01, D01, F03 each introduce a
new port) so reviews stay meaningful.

---

## 9. Provider Capability Matrix (Target)

### 9.1 Legend

| Value | Meaning |
|---|---|
| **NOW** | Implemented and reachable today (source-verified) |
| **PLAN(A01)** etc. | Target state, delivered by the named milestone |
| **GATED** | Intentionally unsupported; UI hides/disables with a surfaced reason and a test covers the boundary |
| **NO** | The provider cannot express the concept at all |

Rules: a `GATED`/`NO` cell must never be emulated by emitting SQL the provider rejects; the
reason string comes from `DatabaseCapabilities`; the UI must never hardcode a driver check.

### 9.2 Schema inspection

| Capability | PostgreSQL | SQLite | Notes |
|---|---|---|---|
| Schemas | NOW | NO (single `main`) | PG schema list is introspected today |
| Tables | NOW | NOW | |
| Views | NOW | NOW | View **columns** missing on both → A02 |
| Materialized views | PLAN(A02) | NO | No `pg_matviews` query today |
| Columns (type/nullable/default/identity/generated/collation) | NOW (identity/generated/collation present) | PARTIAL (no identity/generated/collation) | `IntrospectResult` exists |
| Primary keys (incl. composite) | NOW | NOW | |
| Foreign keys (incl. composite, match/deferrable) | NOW | PARTIAL (shorthand `REFERENCES parent` resolved against parent PK; always enabled triggers) | |
| Unique constraints | PARTIAL (derived from single-column unique indexes) | PARTIAL (same) | A04 adds a real `pg_constraint` `contype='u'` query |
| Check constraints | NOW (`pg_constraint`) | PARTIAL (parsed from `sqlite_master` SQL) | `LIM-011` disposition still open |
| Indexes (method/include/predicate/definition) | NOW | PARTIAL (btree hardcoded; expression/partial lose structure) | |
| Triggers (timing/event/definition/enabled) | NOW (`tgenabled`) | PARTIAL (header parse; `enabled` always true) | |
| Functions / procedures | PARTIAL (list + def; no parameter metadata) | NO (`functions: []` by design) | B01 adds parameter metadata |
| Sequences | PLAN(A06) | NO (AUTOINCREMENT) | Flag exists with no code (C2) |
| Enums / domains / composite types | PLAN(A07) | NO | Flag exists with no code (C2) |
| Dependencies / references | PARTIAL (`pg_depend` `n`/`a` via inherent connector method, not through a port) | GATED | Promote through a port in A08/§4.4 |
| Partitions / tablespaces | PARTIAL (inherent PG methods, Tauri-exposed only) | NO | Out of phase scope; keep as-is or promote later |

### 9.3 Schema mutation

| Capability | PostgreSQL | SQLite | Notes |
|---|---|---|---|
| Create/alter/drop table | NOW (generic raw DDL only) | NOW (generic raw DDL only) | A01 converts this to typed + previewed |
| Add/drop/rename column | NOW (`add_column`, `drop_column`, `rename_column` flags true; builders only for add/drop) | NOW (same) | A01 adds the missing rename/alter builders |
| Alter column type | NOW (flag true; **no builder, no UI**) | NO (`alter_column_type=false`) | A01 (PG) / GATED (SQLite) |
| Create/drop index | PARTIAL (builder + generic DDL) | PARTIAL (same) | A03 |
| PK/FK/Unique/Check create/drop | NO (inspect + re-emit only) | NO | A04 (SQLite FK add may require table rebuild → GATED with warning) |
| Create/drop view | PARTIAL (builder; no ALTER, no `OR REPLACE`) | PARTIAL (same) | A02 |
| Materialized view create/refresh/drop | PLAN(A02) | NO | |
| Trigger create/drop | PARTIAL (builders + generic DDL) | PARTIAL (same) | A05 |
| Trigger enable/disable | PLAN(A05) | NO | |
| Sequence create/alter/drop | PLAN(A06) | NO | |
| Enum/domain create/alter/drop | PLAN(A07) | NO | |
| Schema create/rename/drop | PARTIAL (rename exists as an inherent PG method that **bypasses** the application layer) | NO | A08 promotes it |
| Database create/drop | GATED (PG requires a maintenance connection; only list + guarded create/drop if connected role permits) | NO | A08, low priority |
| Transactional DDL | NOW | NOW | Both support DDL in transactions — use for atomic applies |
| Capability enforcement before SQL | PLAN(A01) — today `execute_ddl` consults only readonly/multi-statement (`schema_service.rs:340-380`) | same | Critical gap |

### 9.4 Routines

| Capability | PostgreSQL | SQLite |
|---|---|---|
| List routines | NOW (`pg_proc`, `prokind f/p`) | GATED (`functions: []`) |
| Parameter metadata / overload signature | PLAN(B01) | NO |
| Source / definition view | PARTIAL (Definition tab, truncated semantics unverified) | NO |
| Create/replace/drop | PLAN(B03) | NO |
| Execute with typed arguments / `CALL` | PLAN(B04) | NO |
| Notices / messages | PLAN(B04) | NO |
| Routine dependencies | PLAN(B02) | NO |

### 9.5 Monitoring / administration

| Capability | PostgreSQL | SQLite |
|---|---|---|
| Server information (version/uptime) | PLAN(D01) | PARTIAL (library version, page/encoding info) |
| Sessions | PLAN(D01) — visibility limited by role privileges; UI must show what is visible | NO |
| Active queries (state/duration/wait) | PLAN(D01) | NO |
| Cancel query (admin) | PLAN(D01); **the per-execution Stop path is GATED today** (`postgres/connector.rs:199-203` returns `Unsupported`; `capabilities.rs:129-132` `cancel=false`) | NOW (interrupt + actor acknowledgement, `sqlite/actor.rs:106-124`) |
| Terminate session | PLAN(D01) | NO |
| Locks / blocking graph | PLAN(D03) | NO |
| Transactions view | PLAN(D03) (derived from `pg_stat_activity` + locks) | NO |
| Database/table/index size | PLAN(D04) | PARTIAL (file size, page count/freelist via PRAGMA) |
| Row estimates vs exact count | PARTIAL (`c.reltuples` approx today) | PARTIAL (`None` today; exact count costs a scan) |
| Scan statistics | PLAN(D04) | GATED |
| Maintenance (`VACUUM`/`ANALYZE`) | PLAN(D04) | PLAN(D04) (`VACUUM`/`ANALYZE`/`PRAGMA optimize`; `VACUUM FULL` equivalent is `VACUUM`) |
| Settings view (`pg_settings`) | GATED (Later) | NO |

**§2.5 C1 is load-bearing here**: the release docs describe the inverse cancel support. Code
is the source of truth; the per-execution Stop gating follows the code flag, and D01 must
resolve whether to implement PG cancellation (`pg_cancel_backend` on the same backend pid
requires tracking the backend pid per pool connection — a real design task, not a flag flip).

### 9.6 Security

| Capability | PostgreSQL | SQLite |
|---|---|---|
| Users/roles list | NOW (backend; PLAN(E01) UI) | GATED (service rejects before provider call) |
| Role create/drop/alter attributes | PARTIAL (create/drop exist; alter PLAN(E01)) | NO |
| Role DDL view | PLAN(E01) | NO |
| Memberships | PLAN(E03) | NO |
| Table grants (allowlisted privileges) | NOW (backend; PLAN(E02) UI) | NO |
| Schema privileges | PLAN(E02) | NO |
| Column/database grants | GATED (Later) | NO |
| Password rotation | PLAN(E03) | NO |
| RLS / policies | GATED (Later) | NO |
| Ownership / default privileges | GATED (Later) | NO |

### 9.7 Transfer

| Capability | PostgreSQL | SQLite |
|---|---|---|
| Export CSV/TSV | NOW (backend; PLAN(C05) UI wiring) | NOW (backend; PLAN(C05) UI wiring) |
| Export XLSX | NOW (`rust_xlsxwriter`, `2^53` guard) | NOW (same) |
| Export JSON | NOW (dup-column + non-finite guards) | NOW (same) |
| Export SQL INSERT | PLAN(C05) | PLAN(C05) |
| Export SQL COPY | PLAN(C05) | NO |
| Import CSV | PLAN(C02) | PLAN(C02) |
| Import Excel (`.xlsx`) | PLAN(C03) | PLAN(C03) |
| Import JSON / NDJSON | PLAN(C04) | PLAN(C04) |
| Streaming + progress + cancel | PLAN(C01) | PLAN(C01) |
| Database-to-database transfer | GATED (Later) | GATED (Later) |
| Backup | PARTIAL (`pg_dump` via PATH; no progress/cancel; PLAN(C01) adds progress) | PARTIAL (`VACUUM INTO` + atomic hard-link publish; not a file copy — C11) |
| Restore | PARTIAL (`psql -f` / `pg_restore`; requires disconnected target on SQLite) | PARTIAL (staged + `PRAGMA quick_check` + rename) |
| Backup format choice | GATED (plain/custom exist in engine; no UI) | NO (single mechanism) |

### 9.8 Cancellation, explain, maintenance (summary)

| Capability | PostgreSQL | SQLite |
|---|---|---|
| Per-execution cancel | GATED today (`Unsupported`) — D01 owns the decision | NOW |
| Cancel / terminate from Monitoring | PLAN(D01) | NO |
| `EXPLAIN` | NOW (JSON plan) | NOW (`EXPLAIN QUERY PLAN`, JSON-wrapped) |
| `EXPLAIN ANALYZE` | NOW (classified as execution → confirmation) | NOT AVAILABLE (SQLite has no ANALYZE-executing explain) |
| Visual plan tree | PLAN(G04/H01) — `components/explain.rs` exists but is gallery-only | PLAN(same, text-derived) |
| Long-running safety (timeout/deadline) | NOW (per-connection `query_timeout_ms`, transaction rollback) | NOW (actor interrupt + ack) |

### 9.9 Unsupported = disabled, never faked

Rules applied to every `GATED`/`NO` cell in this section:

1. The backend rejects the operation before building SQL, returning
   `DbError::Unsupported { .. }` with a stable reason key.
2. The UI renders the affordance disabled with that reason (tooltip + an `Alert` in the
   empty-state position), never with a silently failing button.
3. Tests cover the boundary: the operation is attempted, the rejection is asserted, and no
   SQL reaches the provider (mock connector assertion).
4. Release notes and `docs/release/provider-capability-matrix.md` must match this table.

---

## 10. Safety Model

### 10.1 Operation classes (target)

Extend the existing classifier (`crates/core/src/domain/safety.rs:53-580`) with two classes so
that administration and long-running work are no longer mis-bucketed as `Write` (today the
default arm at `:96` classifies `GRANT`/`REVOKE`/`VACUUM`/`COMMENT` as `Write`):

| Class | Meaning | Examples | Confirmation |
|---|---|---|---|
| `ReadOnly` | No persistent state change | `SELECT`, `SHOW`, `TABLE`, plain `EXPLAIN`, introspection | None (auto-run allowed under the explicit read-only auto-run setting) |
| `Mutation` | Changes data or schema, but is the normal editing path | `INSERT`/`UPDATE`/`DELETE` with `WHERE`, `CREATE`, `ALTER` (non-narrowing) | Preview + single confirmation; grid edits use the staged-change flow |
| `Destructive` | Can lose data, drop objects, or end sessions | `DROP`, `TRUNCATE`, `DELETE` without `WHERE`, `CALL`/`DO`/`EXECUTE`, type narrowing, `DROP ROLE`, `TERMINATE`, `VACUUM FULL`, migration steps that drop | Strong confirmation: target + environment + exact SQL; Production requires typed object name |
| `Administrative` | Operational control, not schema/data | `GRANT`/`REVOKE`, `SET`, session/role attribute changes, `ANALYZE`, `VACUUM` (non-FULL), maintenance actions | Explicit confirmation naming the target and effect; audit entry required |
| `LongRunning` | Orthogonal flag, not a class | imports, exports, backups, restores, migration apply, large introspection | Progress + cancellation + timeout policy; blocked entirely on read-only connections when it mutates |

`LongRunning` is a modifier: an import is `Mutation + LongRunning`; a restore is
`Destructive + LongRunning`. It drives progress/cancel/timeout behavior, not permission.

### 10.2 Policy matrix

| Class | Read-only connection | Full connection | Production connection | Agent (any mode) |
|---|---|---|---|---|
| `ReadOnly` | Allowed | Allowed | Allowed | Auto-run only with the explicit read-only auto-run setting |
| `Mutation` | **Blocked** (`ReadOnlyViolation`) | Preview + confirm | Preview + confirm + environment banner | Preview + explicit confirmation (no auto-run) |
| `Destructive` | **Blocked** | Strong confirm (target + SQL) | Strong confirm + typed object name + `Production` banner | Explicit confirmation with execution-time re-classification (retained anti-TOCTOU) |
| `Administrative` | **Blocked** | Confirm + audit | Confirm + audit + banner | Explicit confirmation |
| `LongRunning` mutating | **Blocked** | Progress + cancel + timeout | Progress + cancel + timeout + banner | Explicit confirmation before start |

Policy producers (this is the C5 fix): the policy is derived from
`(ConnectionConfig.readonly, ConnectionConfig.allow_ddl, ConnectionConfig.allow_destructive,
ConnectionConfig.environment, capability flags)` and evaluated **inside the service** before
SQL construction. Existing behavior to preserve: read-only enforcement in query, DDL, export,
table data, user management, and restore.

### 10.3 Environment model

| Environment | Label | Required behavior |
|---|---|---|
| `Development` | Default | Normal confirmations |
| `Staging` | Badge | Confirmations show the environment; destructive operations get the banner |
| `Production` | High-visibility badge (danger-tinted) | Destructive/Administrative require typed confirmation (object/session name); agent mutating tools cannot auto-run; a persistent badge appears on every workspace `ObjectHeader` and in every confirmation; status bar shows the active environment |

Environment is a first-class field on `ConnectionConfig` (I01), is never inferred from the
host name, and is exported in connection profiles (it is not a secret).

### 10.4 Confirmation contract

Every mutating operation renders the same confirmation payload:

```text
What:      <operation class> on <object kind> <qualified name>
Where:     <connection name> · <environment> · <database/schema>
SQL:       <exact statement(s) that will run, scrollable, copyable>
Effect:    <rows/tables affected, data-loss warning, lock warning>
Rollback:  <"transactional" | "no rollback available">
Buttons:   Cancel | Generate SQL (no execution) | <Confirm action>
```

Rules: the SQL shown is the SQL executed (no regeneration between preview and apply —
resolve any version drift before executing); confirmation is per-operation-class, never a
blanket "don't ask again" for `Destructive`.

### 10.5 Cancellation, timeout, and audit

- Every `LongRunning` operation is request-scoped and cancellable; the UI reflects the stop
  within the interactive budget (target ≤250 ms per `docs/10-egui-native-migration-plan.md`
  §8) and always receives a terminal event.
- Timeouts come from the per-connection `query_timeout_ms` (exists) and are surfaced as
  `QueryTimeout`, not as a generic failure. Transaction paths must roll back on timeout
  (already implemented for both providers).
- Audit: every `Mutation`/`Destructive`/`Administrative` execution writes an audit record
  (timestamp, connection, environment, class, object, SQL digest, outcome) rendered in
  `ActivityLog`. Records never contain bound values or credentials.

### 10.6 Agent-specific rules (binding)

1. No tool opens its own connection; all tools call the canonical APIs (retained).
2. Destructive and mutating tools require explicit confirmation; approval re-classifies SQL
   at execution time and can escalate the required confirmation (retained).
3. New tools declare a risk class and inherit the table in §10.2 — no exceptions for
   "convenience".
4. Agent-generated DDL never executes directly; it flows through the A01 preview/apply path.
5. No autonomous index creation, no auto-vacuum policy, no scheduled agent runs.

---

## 11. Testing & Verification Strategy

### 11.1 Evidence levels (mandatory, per `docs/plans/FEATURE_LIFECYCLE.md`)

1. Source evidence — code proves a path exists.
2. Automated evidence — a command was executed and its result recorded.
3. Provider runtime evidence — behavior observed against real PostgreSQL or SQLite.
4. UI runtime evidence — the lifecycle observed end-to-end in the native UI.

Source evidence must never be written as runtime evidence. A plan with pending
provider/runtime evidence stays under `docs/plans/active/`.

### 11.2 Test layers and ownership

| Layer | Location | Must cover |
|---|---|---|
| Domain unit | `crates/core/src/domain/**` inline tests | Classifier (all classes + `Administrative`/`LongRunning` additions), value objects, identifier/quoting rules, capability/flag consistency |
| Application service unit | `crates/core/src/application/**` with `MockDbConnector`/mock ports | Policy rejection, capability rejection, SQL generation exactness, cache invalidation, transaction policy |
| Provider integration | `crates/infrastructure/tests/**` | SQLite always-run fixtures; PostgreSQL live suites (currently `#[ignore]`, `tests/pg_integration.rs`) — **new PG features must extend the ignored suite and cannot close without a live run** |
| Runtime/worker | `crates/runtime/src/**` inline tests | Command → service routing, cancellation, progress/terminal events, replay idempotency, dead-variant removal |
| UI logic | `crates/ui/src/**` (`app_tests.rs`, per-module tests) | Reducer correctness, polling lifecycle, stale-response guards, confirmation variants, disabled-with-reason states |
| Manual/runtime | `docs/release/0.1.0-manual-smoke.md` + per-milestone `VERIFICATION.md` | Provider walkthroughs, screenshots at 1280×800 / 1440×900 / 1920×1080, light + dark |

### 11.3 Provider independence rule

PostgreSQL evidence never proves SQLite and vice versa. Every milestone records:

| Provider | Supported | Automated | Live/runtime | Capability gate |
|---|---|---|---|---|
| PostgreSQL | yes/no/partial | PASS/NOT VERIFIED/N/A | PASS/NOT VERIFIED/N/A | reason if unsupported |
| SQLite | yes/no/partial | PASS/NOT VERIFIED/N/A | PASS/NOT VERIFIED/N/A | reason if unsupported |

An unsupported provider operation must have: deterministic capability detection, no
unsupported SQL emitted, a surfaced reason, and a test covering the boundary.

### 11.4 Milestone verification recipe (per milestone PR)

```text
1. Plan folder exists (PLAN/CHECKLIST/FINDINGS/VERIFICATION) under docs/plans/active/<slug>/
2. Quality gates executed and recorded:
     cargo fmt --all -- --check
     cargo clippy --workspace --all-targets -- -D warnings   (scope out the legacy host if it still blocks; state the scope)
     cargo test --workspace
     SQLite provider tests (default run)
     PostgreSQL live suite (DATABASE_URL, --ignored) for any PG-touching change
3. Capability boundary tests for every GATED cell touched by the milestone
4. Native UI evidence for every new/changed surface (screenshots per gate size, both themes
   for visual work)
5. VERIFICATION.md records exact commands + observed output; no "works perfectly"
6. STATUS.md updated to match the plan folder and state
```

Rules: never mark `COMPLETED` on source evidence alone; never claim workspace-wide gates are
green while `crates/tauri-app` still fails to build without stating the scope (C10); a
milestone that changes a rail surface must record the placeholder-removal evidence.

### 11.5 Performance test harness

Existing: `crates/ui/benches/result_grid_benchmarks.rs` (criterion), diagram tests at
20/100/500/1000 tables, `crates/infrastructure/benches/sqlite_benchmarks.rs`,
`docs/architecture/performance-baseline.md`, `docs/plans/performance-baseline-audit-2026-08-13.md`.

Targets to add with the milestones that create the risk:

| Scenario | Budget | Owner |
|---|---|---|
| 1000 tables / 10k columns introspection | Explorer stays interactive; no full re-introspection on object-level change | A01–A08 |
| Virtualized rendering (grid, explorer, property grid, monitoring lists) | No per-row allocation for the whole dataset; bounded frame time while scrolling | All UI milestones |
| Large query result | Unbounded query results in memory are a known gap — introduce a cap/progressive fetch before claiming large-result support | G04 / Later |
| CSV/JSON import & export | Bounded memory at 1M rows; progress events ≤ ~10/s | C01–C05 |
| Backup/restore | Progress + cancel; no unbounded buffering of `pg_dump` output | C01/D04 |
| Large schema compare (1000 tables) | Bounded comparison time; virtualized diff tree | F01–F04 |
| Monitoring polling | Idle CPU budget maintained; no UI-thread blocking | D01–D04 |
| Background work | Every long operation runs off the UI thread with cancellation | All |

---

## 12. Exit Criteria

### 12.1 Per-area exit criteria (summary of §5)

| Area | Exit criteria |
|---|---|
| A Object CRUD | Every in-scope object kind: inspect → generate DDL → preview → create/alter/drop → refreshed metadata, with per-provider evidence; no capability flag without code; no SQL built in the UI |
| B Routines | Browse/source/execute/manage end-to-end on PostgreSQL with typed arguments and output; SQLite gated with reason + test |
| C Transfer | Import (CSV/Excel/JSON) with mapping/preview/progress/cancel on both providers; export through the backend engine; SQL INSERT/COPY where supported; Transfers placeholder removed; 100k-row streaming evidence |
| D Monitoring | Sessions/queries/cancel live on PostgreSQL and correctly gated on SQLite; locks/sizes/stats/maintenance with confirmations; Monitor placeholder removed |
| E Security | Roles/users/grants/memberships manageable on PostgreSQL; SQLite gated; no password ever rendered or logged |
| F Compare/Migration | Diff depth covers the object matrix; ordered dialect-correct migration plan; gated transactional apply with rollback plan; lossy cross-provider operations refused, never silently dropped |
| G Productivity | Search finds objects, columns, queries, and commands; palette expanded; favorites/snippets/scratch persist; keybindings remappable; index provably secret-free |
| H AI | Every shipped assistant is a typed tool over canonical APIs, with declared risk class, bounded output, and a confirmed mutation path; zero autonomous destructive behavior |
| I Connections | Folders/tags/favorites work; environment labels visible on every target; profile export secret-free; `allow_ddl`/`allow_destructive` real |
| J Tasks | Only if scheduled: tasks run/record/cancel and cannot bypass policy |
| K Settings | All ten sections persist; no dead settings port; no persisted-but-unread keys |
| L Release | Artifacts install per platform; diagnostics bundle redacted by construction; telemetry default-off or absent |

### 12.2 Product-level exit criteria

1. All 39 milestones are `COMPLETED` (or explicitly re-scoped with a recorded decision) and
   `docs/plans/STATUS.md` matches the plan folders.
2. Every `GATED`/`NO` cell in §9 has a boundary test and a matching reason string surfaced in
   the UI.
3. No rail placeholder exists anywhere; every activity in §3.1 is real.
4. `crates/ui` contains zero SQL string construction for mutating operations and zero direct
   provider imports.
5. Every mutating operation path renders the §10.4 confirmation payload and writes an audit
   record.
6. PostgreSQL and SQLite evidence are recorded independently for every database-facing
   feature; no release note claims a capability the code does not have (§2.5 is empty of
   open conflicts, or each remaining conflict has a recorded owner and deadline).
7. Performance budgets in §11.5 are measured and recorded, with exceptions documented rather
   than waived silently.
8. The v0.1 governance decisions (brand R001, license R004, signing R003/R009) are resolved or
   explicitly accepted in `docs/release/risk-register.md`.

### 12.3 Explicit instruction to implementers

- Do not start Phase A work from this document alone. Each phase has its own execution goal
  (`docs/goals/goal-phase-*.md`) containing the UX, domain model, service/port surface,
  provider behavior, tests, and ordered milestone list required to implement it.
- Do not widen a milestone to "finish the area". A milestone is one coherent deliverable with
  its own evidence.
- Do not implement a capability that contradicts §9. If the provider cannot express it, gate
  it, test the boundary, and surface the reason.
