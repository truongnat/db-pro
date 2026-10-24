---
schema_version: 1
id: "MEM-PLAN-native-ui-foundation-7fbe2b711a90"
title: "Plan archive: Native Ui Foundation"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, native-ui-foundation]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "7fbe2b711a90956aa0cfa508562918b8ebc7623db6a96767fa3b9a575b3c2d48"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/native-ui-foundation/PLAN.md"
  - "docs/plans/completed/native-ui-foundation/FINDINGS.md"
  - "docs/plans/completed/native-ui-foundation/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/native-ui-foundation/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Native UI Foundation

## Status

`COMPLETED`

## Goal

Create the first runnable egui/eframe native base and the first real database workspace slice without changing the existing React/Tauri product surface.

## Scope

- Add `db-pro-ui` library crate.
- Add `db-pro-native` binary crate.
- Establish DB Pro visual tokens and a light-first Codex-inspired theme.
- Implement the first shell: activity bar, sidebar, topbar, statusbar, workspace tabs, welcome view, query workspace and Agent preview panel.
- Connect the native shell to the shared Rust runtime through a typed task bridge.
- Define a provider boundary for the native Agent preview, with offline drafts as the safe fallback.
- Add an optional Rust Codex Responses provider with an async worker path and redacted credential handling.
- Add Codex-style Quick Open and Command Palette navigation for the native workspace.
- Provide the first DBeaver-shaped table workspace with Structure, Data and DDL views.
- Add a bounded native ER diagram canvas for the active schema with table columns and foreign-key relationships.

## Out of scope

- Native SQL editor parity with Monaco.
- Large-schema ER rendering and packaging.
- MCP/tool execution, autonomous database mutations, and persistent API-key management UI.
- Removing React/Tauri.

## Next slice

The native database workspace and shared runtime boundary are implemented and verified. Native Data editing preserves composite primary-key identity and typed key values across the UI/runtime bridge, NUMERIC/DECIMAL fields use exact precision-aware validation, ER opens large schemas in a search-first bounded view, views expose definition and bounded Data browsing, and PostgreSQL functions/procedures have a definition workspace. Tauri commands consume `DbProRuntime` facades. Monaco parity, MCP/autonomous writes, large-schema packaging and deeper Tauri lifecycle work remain explicitly out of scope for this plan.


## Source: `FINDINGS.md`

# Findings

- Full Rust quality gates are available and passing in the current worktree. Native UI screenshot verification was completed through the visible debug binary at 1280×800, 1440×900 and 1920×1080.
- The existing `crates/tauri-app/src/lib.rs` owns Tauri command registration; service construction now lives in the shared `DbProRuntime` so native egui and Tauri consume one graph.
- The current base intentionally uses mock connection/query state so its visual direction can be reviewed independently from backend migration.
- `crates/runtime` owns a Tauri/egui-independent service graph for connection, query, schema, export, table-data, backup, user, diff and PostgreSQL adapter operations. Tauri commands now receive one runtime-owned `Arc` and call typed runtime facades; framework-specific lifecycle wiring remains in the Tauri adapter.
- The native binary now boots the runtime, forwards typed UI commands through a tokio worker, loads persisted connection summaries into the explorer, and supports selecting/connecting a saved connection.
- Query results now cross the runtime/UI boundary as typed columns and cells, with NULL/boolean/number/text/JSON/bytes rendering.
- The native grid now supports filter, click-to-sort, column divider resize, visible-row virtualization through egui `show_rows`, scoped cell/row selection and clipboard copy. Column widths persist through eframe storage and are clamped to safe bounds when restored. Pure tests cover filtered index mapping, sorting and typed cell text; large-result benchmarks remain.
- Connection list/connect/query worker paths now call `ConnectionApi`/`QueryApi` instead of reaching through service internals. `TableDataApi`, `ExportApi`, `BackupApi` and `UserApi` now expose typed native-facing operations. Connection create/update/delete/test commands are available in the shared runtime worker, and native UI now has create/test/edit form validation plus delete confirmation. SSL mode and SSH tunnel fields now map into domain config with validation. Native rfd file pickers now populate SQLite and SSH private-key paths. Runtime backup/restore commands now emit started/completed/failed/cancelled progress events; native backup/restore controls now dispatch typed operations with path fields; native file dialogs and restore confirmation remain. The query surface now has line numbers, find toolbar, zoom controls, F5/Cmd+Enter dispatch, lightweight keyword/string/comment highlighting, basic formatting, current-statement execution, keyword completion, snippets, heuristic diagnostics, hover hints and local query history. Persistent multi-query tabs and local query document metadata are now available through eframe storage. QueryApi now exposes saved-query and folder persistence methods; native list/save query UI and worker commands are now wired. Folder creation and assignment UI are now wired. Saved queries are grouped by folder with delete confirmation. Selection execution and dialect-aware completion/diagnostics are wired. Diagnostics now run through a lightweight tokenizer/parser that tracks strings, parentheses, statement keywords and dialect restrictions; the UI now uses sqlparser with PostgreSQL, SQLite and generic dialect grammars before applying safety diagnostics.
- `egui` 0.29 / `eframe` 0.29 were selected to stay compatible with the workspace's current Rust 1.77 policy; dependency compatibility must be confirmed by CI.
- The common UI audit found two concrete polish issues: ghost buttons were visually filled at rest, and fixed-width topbar/query/results rows could overflow the minimum window. Shared compact controls, transparent ghost states and wrapped toolbars now address those cases. Remaining raw egui controls are intentionally specialized inputs, table headers, checkboxes and selectors.
- The native UX alignment pass now keeps cards border-defined instead of shadow-heavy, shows the active connection in the topbar, exposes runtime error text in the statusbar even while the database remains connected, uses platform-appropriate shortcut labels, and gives the empty Explorer a direct New connection action.
- The layout pass now gives the workspace a stable rail/sidebar/content hierarchy, a framed tab strip with active state, a light welcome card with primary actions, and a two-level topbar for backup/restore. Runtime screenshot review remains the final visual gate.
- The native theme is no longer hard-coded to light: Settings switches between off-white light and token-matched dark surfaces, and eframe persists the choice locally for the next launch.
- Native Settings now exposes a persisted Reduce motion toggle; Agent loading keeps an explicit static icon when enabled, so progress is not communicated by motion alone.
- The Welcome surface now has a centered prompt composer with an explicit Open in Query action and Quick Open fallback, keeping the Codex-style first-run path useful without adding an Agent execution affordance.
- The polish pass showed that backup/restore made the global header too dense. They now live under Settings in dedicated database-file cards; the global topbar is back to a compact 48px workspace header. This keeps the primary task, querying, visually dominant.
- The responsive visual pass found that the SQL editor was sizing to its content instead of the workspace. The editor and result surfaces now reserve the full available content width, and the query action row is one wrapped layout so the Agent panel does not clip actions at narrower widths. The Agent panel is capped at 380px for a better 1440px balance.
- The minimum-size stress pass found that an Agent side panel and Explorer sidebar cannot both remain visible at 1024×640 without making the query surface unusable. Opening Agent below 1180px now temporarily hides Explorer and restores its previous state when Agent closes, including the Escape close path. The side panel is laid out before CentralPanel so egui computes the real remaining width and wraps query actions instead of painting over them.
- The Agent panel close icon had bypassed that shared close transition, so closing from inside the panel could leave Explorer hidden after a narrow-window session. The icon now uses the same restoration path as the topbar and Escape handlers, with a regression test covering a 1024px viewport.
- The repository perf-audit script was rerun after installing the locked frontend dependencies. The 10 performance-budget tests pass and ER code-splitting is intact, but the raw production asset budget still fails: JavaScript totals 2.35 MB (critical threshold 2 MB), the largest chunk is 594 KB, and CSS totals 122 KB (warning threshold 100 KB). This is a P2 startup/bundle concern, not a database correctness issue; the current UI already keeps Query/Schema/ER feature payloads in lazy chunks, so a larger bundling refactor stays out of this UI slice.
- The SQLite result-grid review found column headers inheriting an unreadable light widget color. Header labels now explicitly use the primary text token, matching the light theme while keeping sort affordances visible.
- The post-rebuild runtime review found the Explorer connection indicator conflating selection with a live session. A selected-but-disconnected connection now uses a neutral outlined indicator and accent selection, while the check/green state is reserved for an active connection; a unit regression test locks the distinction.
- Native result-grid benchmark coverage now exercises the production row-index projection with 1,000,000 rows and a 100-row visible scroll window. The optimized build measured about 3.15 ms for the full no-filter/no-sort projection (~318M rows/s) and about 38 ns for visible-window materialization; these numbers cover metadata/index work, not GPU paint time.
- Query tabs now expose a close control once more than one document exists. Closing a non-active or active document clamps the active index, restores the remaining document content, clears stale results, and keeps one query document available as the workspace entry point.
- The Codex-light base-component pass now centralizes navigation frames, painted activity/sidebar items, single-line/password inputs and consistent button sizing/radii. Native runtime review confirmed the light hierarchy, idle/hover/active sidebar states, violet input focus ring, query toolbar, editor and mutation fields at 1280×800. The remaining raw controls are intentionally specialized editor, selector, checkbox and result-grid widgets.
- Table selection now persists in the Explorer after the pointer leaves the row, keeping the selected table and generated query visually tied to the active sidebar state.
- The first native table workspace now loads real metadata through `SchemaService::get_table_info` and `get_table_ddl`, and loads bounded table data through `TableDataApi`; SQLite schema resolution uses `main` while PostgreSQL uses `public`.
- Structure, Data and DDL are separate workspace views so inspecting an object does not destroy the prepared query. Data reuses the result-grid filter, sorting, selection and copy controls; DDL uses the editor token surface with an editable, confirmation-gated single-statement apply path.
- The Explorer previously rendered `public` for every provider and kept a disconnected global column list after selecting a table. It now renders the active provider schema (`main` for SQLite), nests selected-table columns/indexes/foreign keys below the table row, and keeps the global column list only as a pre-selection fallback.
- The native Explorer now has a bounded table-name search and reports filtered/total counts before rendering at most 100 rows, preventing long schema lists from dominating the sidebar first paint.
- Table Data now carries an explicit page offset through the UI command, native translation and runtime worker. The Codex-light toolbar exposes previous/next controls and a visible range such as `101–150 of 150`, while the existing virtualized result grid remains the row renderer.
- PostgreSQL and SQLite connector dialect tests now lock their real placeholder/pagination contracts (`$n` versus `?`) at the infrastructure boundary; the isolated PostgreSQL fixture runtime matrix passes 10/10, while the user's BSN database remains read-only verification-only.
- Table Data contains filters and `ORDER BY` controls now travel as typed UI/runtime DTOs into the shared parameterized SQL builder. The filter uses a bound `LIKE` value and the sort identifier is quoted by the builder; the existing “Search visible rows” field remains explicitly local to the current page.
- The core introspection layer already collected views and triggers, but the native summary DTO discarded them. The runtime now preserves view definitions and trigger attachment/timing/event/definition metadata; Explorer renders separate Views and Triggers groups, and selection opens a definition workspace with an Open in Query action.
- A view is a read-only relation for the native workspace: its Data tab reuses `TableDataApi` with the view name as the quoted relation, so pagination, bound contains filters, ORDER BY and the virtualized grid stay consistent with tables without exposing mutation controls.
- PostgreSQL function introspection now reads `pg_proc`/`pg_namespace` and `pg_get_functiondef`, preserving overloaded routines as separate rows with schema, routine type and return type. The native DTO and Explorer expose them as definition-only schema objects; SQLite reports an explicit empty Functions group because its provider capability is false.
- Runtime review with Agent open exposed a wrapped “Row mutation” field becoming visually detached from its group. The mutation controls now sit in a dedicated toolbar frame with a compact header and the same wrapped input row, so narrow workspace widths preserve grouping instead of producing an orphan field.
- Tauri startup previously duplicated the full connector, registry, metadata, secret and application-service graph that 

… [truncated for memory entry; see source file]


## Source: `VERIFICATION.md`

# Verification

## Native UI module refactor — 2026-09-11

- `cargo fmt --all -- --check` PASS.
- `cargo test --workspace --offline` PASS: 305 tests passed; the 10 PostgreSQL fixture tests are intentionally ignored by the default workspace command and were run separately against an isolated fixture container.
- `cargo clippy --workspace --offline --all-targets -- -D warnings` PASS.
- `cargo build -p db-pro-native --offline` PASS.
- Native launch smoke PASS: `target/debug/db-pro-native` remained alive for 8 seconds with no stdout/stderr or crash; the command was then stopped by the timeout.
- Native UI ownership is split into bounded modules; `crates/ui/src/app.rs` is 677 lines and contains root state/update orchestration, while state initialization/persistence constructors live in `app_state.rs` and tests live in `app_tests.rs`.
- Clean-code hardening: SQL `WITH`/quote scanners and SQLite foreign-key grouping no longer use panic-prone `unwrap()` paths; `cargo test -p db-pro-core --offline` (186 tests), `cargo test -p db-pro-infrastructure --offline` (37 unit + 25 SQLite integration tests) and targeted core/infrastructure clippy all PASS.
- Explorer clean-code refactor: the 366-line `draw_explorer` render block is now a coordinator over connection, schema feedback, table, view, trigger, function and footer sections; UI behavior is unchanged and the 36-test UI suite plus UI clippy remain PASS. The diagram zoom label no longer uses a numeric cast, and the test-only large-error lint exception now records its reason.
- Native composition-root refactor: `crates/native-app/src/main.rs` now contains startup, worker lifecycle and window setup (158 lines); command/event translation and DTO mapping live in `translate.rs`. `cargo test -p db-pro-native --offline` (1 test) and native clippy PASS.
- Final follow-up: PostgreSQL NUMERIC wire-scale decoding and native event translation were covered by focused tests; workspace test/clippy/build gates remained PASS, and the native binary stayed alive for an 8-second launch smoke with no output or crash.

## Tauri DTO facade migration — 2026-09-11

- Tauri commands now receive one `State<Arc<DbProRuntime>>`; no command module imports or stores `ConnectionService`, `QueryService`, `SchemaService`, `TableDataService`, `ExportService`, `BackupService`, `UserService` or `DataDiffService` directly.
- Runtime facades cover connection details/secret testing, query multi/explain/history/run-config operations, schema batch/cache operations, cross-connection diff, PostgreSQL-specific tunnel/catalog operations, table data, export, backup and user management.
- `cargo check -p db-pro-runtime -p db-pro-tauri --offline` PASS.
- `cargo test -p db-pro-runtime --offline` PASS (4 tests); `cargo test -p db-pro-tauri --offline` PASS (21 tests); `cargo clippy -p db-pro-runtime -p db-pro-tauri --offline --all-targets -- -D warnings` PASS.
- Direct-service boundary audit: `rg 'State<.*Arc<.*Service>|use db_pro_core::application::.*Service' crates/tauri-app/src` returns no matches.
- Local provider check: `127.0.0.1:15433` accepts `postgres/postgres` and exposes the BSN schemas (`master`, `tenant1`, `tenant2`, etc.); it was left read-only and untouched.
- Isolated PostgreSQL fixture: a temporary `postgres:18` container on `127.0.0.1:15434` loaded `fixtures/postgres/001_schema.sql` and `002_seed.sql`; `DATABASE_URL=postgres://dbpro:dbpro_test@127.0.0.1:15434/dbpro_fixture cargo test -p db-pro-infrastructure --test pg_integration --offline -- --ignored` passed **10/10**. The temporary container was removed afterward.
- Numeric mapper fix: SQLx's `BigDecimal` decode did not apply PostgreSQL wire `dscale`, producing `12345678901234567890.12345000`; the mapper now decodes PostgreSQL binary NUMERIC display scale directly and preserves exact text. Unit coverage and the live PostgreSQL numeric/enum test pass.

## Native UX alignment follow-up — 2026-09-11

- Native UI refactor gate: `crates/ui/src/app.rs` reduced to 677 lines of root state/update orchestration; initialization/persistence state lives in `app_state.rs`, and query, table editor, runtime events, palette, schema object and workspace rendering live in bounded modules. `cargo test -p db-pro-ui --offline` PASS (36 tests); `cargo clippy -p db-pro-ui --offline --all-targets -- -D warnings` PASS.
- `cargo fmt --all -- --check` PASS.
- `git diff --check` PASS.
- `cargo test -p db-pro-ui --offline` PASS (36 tests, including light/dark token switching, workspace-tab close/orphan recovery, connection/schema stale-event, connection-draft invalidation, Explorer search, ER search and DDL impact coverage).
- `cargo test -p db-pro-infrastructure --offline` PASS (37 unit/integration tests plus 25 SQLite integration tests; the 10 PostgreSQL fixture tests pass in the separate fixture run above).
- `cargo clippy -p db-pro-infrastructure --offline --all-targets -- -D warnings` PASS.
- `cargo test -p db-pro-native --offline` PASS (1 SQLite credential-boundary test).
- `cargo clippy -p db-pro-ui -p db-pro-native --offline --all-targets -- -D warnings` PASS.
- `cargo build -p db-pro-native --offline` PASS; the native binary stayed alive for an 8-second launch smoke with no stdout/stderr or crash.
- `cargo test --workspace --offline` PASS; all default workspace tests passed, with PostgreSQL fixture coverage recorded separately above.
- `cargo clippy --workspace --offline --all-targets -- -D warnings` PASS.
- Clean-code diff scan: no frontend production findings and no blocking translation-function finding; remaining warnings are existing egui render boundaries/Tauri bootstrap and intentionally documented in `docs/quality/CLEAN_CODE_AUDIT_REPORT.md`.

## Automated

Verified on 2026-09-11 from the current worktree:

```text
cargo fmt --all -- --check                                      PASS
cargo check --locked -p db-pro-runtime -p db-pro-ui -p db-pro-native PASS
cargo test --locked -p db-pro-runtime -p db-pro-ui                 PASS (26 UI tests; runtime has 4 tests)
cargo clippy --locked -p db-pro-runtime -p db-pro-ui -p db-pro-native --all-targets -- -D warnings PASS
cargo check --locked --workspace                                  PASS
cargo test --locked --workspace                                   PASS (305 passed; 10 PostgreSQL tests ignored)
cargo clippy --locked --workspace --all-targets -- -D warnings    PASS
cargo fmt --all -- --check                                      PASS (after Lucide icon integration)
cargo check --locked -p db-pro-ui -p db-pro-native                  PASS
cargo test --locked -p db-pro-ui -p db-pro-native                   PASS (26 UI tests)
cargo clippy --locked -p db-pro-ui -p db-pro-native --all-targets -- -D warnings PASS
native composite-PK identity unit coverage                             PASS (26 db-pro-ui tests; each key column and typed UiCell preserved)
native exact-decimal input unit coverage                                PASS (NUMERIC/DECIMAL exponent, scale and precision checks preserve the original decimal text)
common UI audit                                                     PASS (shared action controls, ghost state, wrapped toolbars)
workspace layout pass                                               PASS (tab strip, welcome card, topbar hierarchy)
settings surface pass                                               PASS (backup/restore moved out of global header)
responsive native UI pass                                           PASS (1024×640, 1280×800, 1440×900, 1920×1080)
native SQLite result-grid pass                                      PASS (2-row fixture, visible headers/filter/copy controls)
Agent close restoration regression                                     PASS (11 db-pro-ui tests; narrow viewport state restored)
Connection status indicator regression                                  PASS (12 db-pro-ui tests; selected/disconnected differs from connected)
native result-grid benchmark                                            PASS (1,000,000-row projection ~3.15 ms; 100-row visible window ~38 ns; Criterion --quick)
query-tab close state regression                                         PASS (15 db-pro-ui tests; active SQL document restored after close)
Codex-light base component/runtime review                                PASS (1280×800 welcome, Settings inputs, Query/editor/mutation controls; focus ring and sidebar states visually checked)
Agent-open mutation toolbar review                                        PASS (row mutation controls remain grouped in a framed header/action surface when the 380px Agent panel is open)
SQLite native end-to-end smoke                                            PASS (connected Native Test fixture, selected customers table, ran SELECT, rendered 2 result rows, opened Agent and generated a schema draft)
Explorer table selection persistence review                                PASS (selected `customers` keeps the active sidebar treatment after query navigation)
Native table workspace review                                               PASS (SQLite `main.customers` opened into Structure, Data and DDL views; 3 columns and generated CREATE script rendered)
Native table data runtime smoke                                             PASS (Data tab fetched 2 rows through `TableDataApi`; shared filter/copy grid rendered)
Provider-aware Explorer review                                             PASS (SQLite renders `main`; selected `customers` expands Columns/Indexes/Foreign keys in the sidebar)
Native table pagination runtime smoke                                       PASS (150-row SQLite fixture loaded `1–100 of 150`, then `101–150 of 150` through the next-page command)
Server-side table filter runtime smoke                                      PASS (SQLite `id contains 101` returned `1–1 of 1` through `TableDataApi`)
Server-side table sort runtime smoke                                        PASS (SQLite `ORDER BY id DESC` returned the first page beginning at id `150`; UI restored to `ASC`)
Native view definition runtime smoke                                        PASS (SQLite `active_customers` appeared under Views and opened its `CREATE VIEW` definition)
Native trigger definition runtime smoke                                     PASS (SQLite `customers_after_insert` appeared under Triggers with `customers · AFTER · INSERT` metadata and opened its definition)
Native view data runtime smoke                                               PASS (SQLite `active_customers` loaded `1–75 of 75` through `TableDataApi`; shared filter/sort/pagination grid rendered active rows)
Native function DTO/UI review                                                PASS (PostgreSQL catalog mapping preserves schema, routine type, return type and definition; restarted SQLite app renders `Functions (0)` for the unsupported provider)
Shared-runtime wiring review                                              PASS (Tauri setup constructs `DbProRuntime`; Arc service/connector/registry states are managed for command adapters; workspace compiles and tests pass)
Native ER diagram runtime smoke                                            PASS (restarted SQLite fixture rendered `main.customers` and `main.orders`, showed `2 tables · 1 relationships`, drew the `customer_id → id` orthogonal connector, and clicking a node opened its Table/Structure workspace at 1280×832)
Native table Data inline-edit UI smoke                                      PASS (SQLite `main.customers` loaded `1–100 of 150`; selecting a cell and pressing Edit cell opened an in-grid editor with the violet focus ring, and Escape cancelled without submitting a mutation)
Native table Data New row UI smoke                                          PASS (SQLite `main.customers` opened a Codex-light metadata-driven Insert row dialog with `id · INTEGER`, `name · TEXT` and `status · TEXT` fields; the dialog was cancelled without changing the fixture)
Native DBeaver-style table Data UI smoke                     

… [truncated for memory entry; see source file]

