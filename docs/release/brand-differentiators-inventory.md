# Source-backed product differentiators and parity evidence (#120)

- **Audited baseline:** `main @ 11854b3` (issue-queue pass 3, 2026-09-15)
- **Issue:** #120 ([RC1][Brand Prep] Inventory source-backed product differentiators and parity evidence) —
  parent brand workstream **#30**, supports #96/#97/#98/#105
- **Purpose:** stop positioning from marketing roadmap as shipped capability. Every row below is decided
  from the current tree (source, tests, or a named registry/audit entry) and carries safe wording plus, where
  it matters, the wording that must **not** be used.
- **Companion evidence:** the capability→tests→CI→smoke map is `docs/release/audit-release-traceability.md`
  (#131); the provider support facts are `docs/release/provider-capability-matrix.md` (#132, corrected by
  #128); the limitations registry is `docs/release/known-limitations.md` (#135).

**Status classes** (the issue's five, used strictly): `SHIPPED/VERIFIED IN v0.1 PATH` ·
`IMPLEMENTED BUT NOT RELEASE-QUALIFIED` · `PREVIEW/EXPERIMENTAL` · `ARCHITECTURE FOUNDATION ONLY` ·
`BACKLOG`.

## 1. Capability state

| Capability | Status | Source / test / doc evidence | User-visible value | Parity or wedge |
|---|---|---|---|---|
| PostgreSQL + SQLite connection lifecycle (create, test, connect, disconnect, reconnect) | `SHIPPED/VERIFIED IN v0.1 PATH` | `connection_service/tests.rs` (24 tests), live `pg_integration` (25 cases), `integration.rs` (32) | two engines, no server-side agent to install | **commodity parity** |
| Query editor: current statement, selection, run-all | `SHIPPED/VERIFIED IN v0.1 PATH` | `query_document.rs`/`renderer.rs` tests, `pg_execute_batch_*`; matrix rows in `audit-execution-safety.md` | run one statement or a script | commodity parity |
| Multi-statement semantics (worst-statement classification, atomic batch) | `SHIPPED/VERIFIED IN v0.1 PATH` | `classify_script_safety` (`safety.rs:123`), `execute_transaction`, `audit-execution-safety.md` §2 rows 6–8 | a failing statement does not leave a half-applied script | **wedge (rarely documented this explicitly)** |
| Destructive-statement confirmation | `SHIPPED/VERIFIED IN v0.1 PATH` | `hold_destructive_run` + `draw_destructive_run_dialog` (#129), 4 falsified UI tests | DROP/TRUNCATE/unbounded DELETE are held until confirmed | wedge over "just a SQL box" |
| Query cancellation | `IMPLEMENTED BUT NOT RELEASE-QUALIFIED` | capability gating `providers/15`; SQLite cancels, PostgreSQL is gated **off** (LIM-014) | honest per-provider behaviour | parity, with a stated asymmetry |
| Query history | `SHIPPED/VERIFIED IN v0.1 PATH` (in-session) | `QueryHistoryRecord` in `events.rs`; history persistence is not shipped | re-run recent statements | parity |
| Explain | `SHIPPED/VERIFIED IN v0.1 PATH` | `explain_query_uses_selected_connection_and_switches_output`, `test_multi_tab_explain_plan_routing` | plan inspection | parity |
| Schema explorer + introspection (tables, columns, indexes, FKs, triggers, views, DDL text) | `SHIPPED/VERIFIED IN v0.1 PATH` | `pg_introspect_*` (6 live tests), `integration.rs`, `introspect.rs` parser tests | full structural browsing | parity |
| CHECK-constraint exposure | `IMPLEMENTED BUT NOT RELEASE-QUALIFIED` | LIM-011, disposition pending #68 | some constraints are not shown | gap to disclose |
| Data Grid virtualization + filter + sort + paging | `SHIPPED/VERIFIED IN v0.1 PATH` with one known defect | `filter_returns_original_row_indexes`, `sort_is_stable_over_filtered_indexes`, `test_table_data_limit_and_paging_offset`; **#238** = sorted large results recomputed per frame (measured 3,414 ms at 200k rows) | works at scale, with a documented responsiveness defect | parity + a fix in flight |
| Data Grid staged update/delete/insert, no-PK and read-only gating | `SHIPPED/VERIFIED IN v0.1 PATH` | `table_edits_stage_until_explicit_apply`, `no_primary_key_table_blocks_safe_row_mutations`, #62 policy tests, backend re-checks | edits are reviewed and applied atomically; unsafe tables are locked out | **wedge (safety by construction)** |
| Workspace / tabs lifecycle | `SHIPPED/VERIFIED IN v0.1 PATH` | `dirty_query_close_is_deferred_until_user_decision`, `test_open_table_blocked_with_unapplied_staged_changes` | multiple query/table tabs with close guards | parity |
| Workspace/tab **persistence** | `BACKLOG` (registry: not implemented) | LIM-016; runbook Step 18 tells the verifier to expect it | — | must be disclosed |
| Quick Open / Command Palette | `SHIPPED/VERIFIED IN v0.1 PATH` | `quick_open_filters_workspaces_by_title_and_description`, `command_palette_*` (4 tests) | keyboard-first navigation | parity |
| ER diagram (native painter, LOD, spatial index, search/neighbourhood) | `SHIPPED/VERIFIED IN v0.1 PATH` | `diagram/tests.rs` incl. 500/1000-table timing + `search_subset_reuses_existing_graph` | schema map that stays usable on large schemas | **wedge (bounded large-schema behaviour is *tested*, not claimed)** |
| ER "React Flow / Cytoscape architecture" | `BACKLOG` (retired) | React frontend archived 2026-09-11; the native renderer is `crates/ui/src/diagram/` | — | **do not market** (no such component ships) |
| Export CSV/TSV from the query view | `IMPLEMENTED BUT NOT RELEASE-QUALIFIED` | `test_export_result_writes_escaped_delimited_text`; not atomic, not cancellable (#244) | export a result without precision loss | parity, with a disclosed weakness |
| Export JSON / XLSX | `BACKLOG` (library code, no shipped surface) | core `ExportService` unreachable from the app (#128 E-4, matrix row corrected) | — | **do not market** |
| Import | `BACKLOG` (deferred) | LIM-012, `audit-data-integrity.md` | — | **do not market** |
| Agent: typed tool surface, approval gate, cancellation, Preview badge | `PREVIEW/EXPERIMENTAL` | `agent_workflow.rs` (mode + `execution_decision`, approval bound to run id + document version), `agent_executor.rs`, `agent_view.rs:224` badge; LIM-007 | ask for schema info, propose SQL, run approved statements | **current wedge *if* framed as Preview** |
| Local-first / zero-telemetry | `SHIPPED/VERIFIED IN v0.1 PATH` | `audit-security-boundaries.md` §1 (telemetry/update/licence/remote-asset greps → 0; only two user-key-gated AI endpoints; no socket on the startup path) | nothing leaves the machine unless the user configures a connection or an AI key | **wedge — and the strongest verifiable one** |
| Type-aware, lossless value fidelity (exact int64/numeric, dedicated temporal classes, byte-exact fallback, one escaping path) | `SHIPPED/VERIFIED IN v0.1 PATH` | `provider-value-contract.md` (20-row class matrix), `providers/37`–`46`, `live_fixture_query_survives_the_provider_to_dto_path`, `int64_serializes_as_string_for_lossless_ipc` | no silent precision, timezone or byte loss when reading, copying or exporting | **wedge (documented + fixture-pinned)** |
| Backups/restore SQLite | `SHIPPED/VERIFIED IN v0.1 PATH` (packaged leg pending) | 8 regression tests (#145), `sqlite_backup.rs` | snapshot-safe, atomic backup/restore | wedge vs tools that copy files |
| Backups/restore PostgreSQL | `IMPLEMENTED BUT NOT RELEASE-QUALIFIED` | `pg_dump.rs`, `audit-data-integrity.md` (needs tools on PATH, LIM-015; restore not transactional, #244) | dump/restore through local tools | parity, with disclosed limits |
| SSH tunnel plumbing | `IMPLEMENTED BUT NOT RELEASE-QUALIFIED` | `tunnel.rs` (+ argv/secret tests), LIM-006, #239 (in-UI caveat filed) | reach a database through a bastion | **must be framed as unqualified** |
| Schema/admin/users/roles mutation, index/trigger editors | `BACKLOG` | goals `goal-phase-a-*`, issues #183–#190, #198; the native app has no such surface (#128) | — | **do not market** |
| Row insertion completeness | `IMPLEMENTED BUT NOT RELEASE-QUALIFIED` | LIM-003: staged insert ships, full dialog set does not | insert a row into a PK table | disclose |
| MCP / external agent ecosystem | `BACKLOG` | LIM-008, #32 | — | **do not market** |
| Packaging: macOS ARM64, unsigned, no installer, no auto-update | `IMPLEMENTED BUT NOT RELEASE-QUALIFIED` | LIM-017/018, `0.1.0-packaging.md`, runbook Steps 1/18/19 | a single `.app` archive that launches and persists state | disclose |
| Windows / Linux runtime | `BACKLOG` | `R-WINLINUX`, #92/#93, `platform-prerequisites.md` (build vs run prerequisites) | — | **do not claim support** |
| Public name / licence | unresolved **owner decision** | LIM-001, LIM-010, #30/#101/#119 | — | **do not state a product name as final** |

## 2. Positioning table for #97 — can say / cannot say yet

| Can say (source-backed today) | Cannot say yet |
|---|---|
| "A local-first PostgreSQL + SQLite desktop client: no telemetry, no update pings, no remote assets — verified by source audit, and the only network calls are the AI features you enable with your own key." | anything implying "enterprise-grade security/compliance", "certified", "audited by a third party" |
| "Built so values survive the round trip: exact integers and decimals, distinct timestamp/timezone classes, byte-exact binary — pinned by a per-class contract and a checked-in fixture." | "lossless for every PostgreSQL type" (arrays/range/composite deliberately fall back to bytes in v0.1) |
| "Editing is staged and applied in one transaction; tables without a primary key are locked out of row editing at both the UI and the core." | "safe for arbitrary production writes" or "guaranteed undo" (staging is not a backup) |
| "A destructive statement is held for confirmation, and a script is classified by its most dangerous statement before it runs." | "prevents destructive mistakes" (it confirms; it does not block by policy) |
| "The ER map stays usable on large schemas: 500- and 1000-table timing and invariants run in the test suite." | "handles any schema size" (tested at 1000 tables; the explorer at 250 is an accepted, unmeasured risk — §A `QA-P2-14`) |
| "Ships as one native binary with an embedded SQLite and bundled fonts — nothing to download at runtime." | "available for Windows and Linux" or "auto-updating" |
| "An AI agent that proposes and only executes with your approval, on a typed tool surface — marked Preview." | "production-ready AI", "autonomous database agent", "MCP support" |
| "You can restore a SQLite database from a snapshot-safe backup, and inspect a PostgreSQL dump/restore flow that uses your local tools." | "one-click backup for every engine" or "import/export in any format" |

**Prohibited overclaim wording** (no shipped surface backs any of these): "database administration platform",
"team collaboration", "supports MySQL / MariaDB / Oracle / SQL Server / MongoDB", "fully qualified SSH
tunnels", "schema migration and DDL generation", "user and role management", "data import", "Excel/JSON
export", "cross-database transfer", "monitoring and locks dashboard", "query plan visualiser", "scheduled
tasks", "enterprise SSO/audit logging", "available on Windows/Linux".

## 3. Acceptance check

- **No long-term Agent/MCP capability is mislabeled:** the agent is `PREVIEW/EXPERIMENTAL` with LIM-007 and
  the in-UI `Preview` badge; MCP is `BACKLOG` with LIM-008 — both stated above and in the registry.
- **Actual v0.1 strengths have source/test evidence:** the four wedge rows each name a document *and* the
  tests or audits behind it (type matrix, local-first audit, staged-mutation + read-only tests, the
  destructive hold, the large-schema timing tests).
- **#96 can compare like-for-like:** §1 uses the same five status classes the issue defines, so a competitor
  matrix can be built on identical terms; commodity parity rows are marked so the comparison does not
  mistake table stakes for advantage.
- **#97 gets an evidence-backed can-say / cannot-say table** (§2) plus the explicit prohibited list.
- **Baseline SHA recorded** (`11854b3`); unresolved evidence gaps are the open defects #238, #243, #244 and
  the host-blocked runtime rows (#91–#95, `R-GUI-SMOKE`, `R-WINLINUX`), not silent unknowns.

**Unresolved evidence gaps that positioning must not fill in by itself:** the public name (#30/#101), the
licence (#119), whether the candidate is publishable at all (`0.1.0-readiness.md`: `PUBLIC_RELEASE_READY =
NO`), and the AI-egress disclosure (#242) — until that lands, no claim should imply that the AI features
are "fully local".
