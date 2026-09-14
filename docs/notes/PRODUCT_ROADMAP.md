# DB Pro — Product Roadmap (Audit-Grounded)

- Date: 2026-09-14
- Branch: `feature/product-audit-roadmap`
- Inputs: `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` (code-evidence audit), `docs/notes/product-roadmap-dbeaver-parity.md` (vision, retained), `docs/plans/STATUS.md`, `docs/release/*`, `docs/architecture/*`.
- Direction (unchanged): **DBeaver-class database client + Codex/VS Code-style native UX + AI-native workflow.**
- This doc adjusts the phase plan to what the audit actually found. No implementation in this task.

## 0. Where we are

Current (do not expand scope until closed):

- Native Visual Redesign — IMPLEMENTING (Waves 1–14 code done; all-surface light/dark traversal, provider matrix refresh, independent review remain)
- Core Safety / Table Editor / Query Intelligence / Agent Workflow / Large-Schema ER / RC1 QA — all RUNTIME_VERIFY (P0=P1=0; native + live-provider evidence pending)
- v0.1 blockers per `docs/release/`: manual desktop smoke sign-off, native packaging/installers, governance (brand collision R001, LICENSE R004, signing R003/R009)

Release boundaries (firm):

| Release | Content | Rule |
|---|---|---|
| v0.1 | RC closure only: runtime verification, packaging, smoke, governance accepts | No new product/agent features during remediation (STATUS.md rule) |
| v0.2 | Phase A (object CRUD) + Phase B (functions/procedures) + palette search expansion (G1) | Engine pays off: builders/services already half-exist |
| v0.3 | Phase C (transfer) + Phase D (monitoring core) + Phase E (users/roles UI) | Admin surface becomes real |
| Later | Phase F depth (migration apply), Phase G remainder, Phase H (advanced AI), ER design mode, DB-to-DB transfer | Only on stable typed tools |

No DBeaver parity in a single release. Each phase below lists exit criteria; a phase is not done until its runtime verification + safety + tests are recorded.

Architecture rules (binding for every phase):

1. Reuse `domain → application → ports → provider adapters → runtime worker → native workbench`. New `UiCommand`/`RuntimeCommand` pairs only; UI never calls provider logic directly.
2. Unsupported operations are capability-gated with a surfaced reason; never emit unsupported SQL.
3. Reuse workbench patterns (`ObjectList`, `ObjectDetails`, `PropertyGrid`, `DDLViewer`, `FormEditor`, `Confirmation`, `CommandPalette`, `SearchResults`, `Tree`, `DataGrid`, `ResultGrid`, `ActivityPanel`, `EmptyState`, `Loading/Error`) — no one-off UI per object.
4. Agent gets no parallel DB implementation; new capabilities arrive as typed tools over canonical actions; destructive ops always explicit confirmation with execution-time re-classification.
5. Fix the §16 matrix conflicts (cancel inversion, flag-without-code) before building on those flags.

---

## Phase A — Database Object CRUD (P1, v0.2)

Goal: every introspected object becomes manageable: inspect → DDL → create/alter/drop with preview + confirmation.
User value: the single largest DBeaver gap; turns DB Pro from viewer into workbench.
Existing reusable: `ddl_builder.rs` (table/view/index/trigger builders), `execute_ddl/batch` + policy, reconstructed DDL views, table sub-tab patterns, `PostgresApi::rename_schema_object` (promote into core service).
Missing architecture: typed `create/alter/drop` application services per object; capability-check enforcement inside `SchemaService`; view-column introspection; sequence/type/matview catalog + domain.

Features: A1 typed table create/alter/drop service + form UI; A2 column add/drop/rename UI (+PG alter-type); A3 view create/drop/replace UI; A4 materialized views PG (introspect/refresh/browse/drop); A5 index create/drop wizard UI; A6 FK create/drop UI; A7 trigger editor + PG enable/disable; A8 sequence browser + CRUD (new domain/introspection); A9 PG types/enums/domains browser + CRUD; A10 schema create/drop/rename (promote PG rename; SQLite n/a-gated).
Dependencies: capability flags corrected (§16.2); safety policy for DDL; introspection cache invalidation (exists).
Safety: DDL preview ("Generate SQL") before execute everywhere; destructive (drop) confirmation; readonly-gate; transactional batch where supported.
Tests: builder unit + service integration per provider (PG live + SQLite), UI regression for dialogs.
Runtime verification: PG + SQLite CRUD walkthroughs per object, native screenshots, error-path (e.g. drop-restricted) evidence.
Exit criteria: matrix §2/§3 rows for tables/columns/views/matviews/indexes/FKs/triggers/sequences/types read PARTIAL-or-better with UI wired; zero "flag-without-code" left in this scope.

## Phase B — Functions / Procedures (P1, v0.2)

Goal: first-class routine surface: browse → source → execute → manage.
User value: closes the other major DBeaver-class gap; routines are where app logic lives.
Existing reusable: PG `pg_proc` introspection + `Function` domain + Explorer rows + read-only Definition view; `QueryService::execute` for CALL/SELECT; agent `patch_query` for edit flow.
Missing architecture: routine source/args metadata service; execute-with-arguments form + result rendering; create/replace/drop service; procedure CALL helper; dependency listing for routines.

Features: B1 routine explorer polish (signatures, grouping, search); B2 source/DDL view + overloads; B3 execute with argument form (typed inputs from `QueryParam`) + output rendering; B4 create/replace/drop with preview; B5 routine dependencies; B6 agent explain/edit flow for routines.
Dependencies: Phase A confirmation/preview patterns; QueryParam UI groundwork (shared with G snippets/params).
Safety: `CREATE OR REPLACE` preview; DROP confirmation; execution through policy (CALL is Destructive-class → confirm); no debug-server ambitions (deferred).
Tests: PG live (routines are PG-only; SQLite gated), UI arg-form tests.
Runtime verification: PG routine walkthrough end-to-end on native UI.
Exit criteria: matrix Functions/Procedures rows PARTIAL-or-better with execute path; SQLite correctly gated everywhere.

## Phase C — Data Transfer (P1 engine+import, v0.3)

Goal: move data in and out safely: import wizards + wired export + reliable backup/restore UX.
User value: import is the most common non-query task; export engine already exists but unreachable.
Existing reusable: `ExportService` (CSV/JSON/XLSX + guards), `BackupService` + engines, result-grid copy matrix, Settings backup/restore minimal UI.
Missing architecture: entire import pipeline (readers, preview, column/type mapping, conflict strategy, transactional apply, progress, cancellation, streaming); native export triggers; transfer job model for the Transfers activity.

Features: C1 CSV import wizard (preview/mapping/transaction/progress/cancel); C2 JSON import; C3 Excel import; C4 wire backend exporters to native UI (CSV/JSON/XLSX) + fix native CSV quoting; C5 SQL INSERT/COPY export generation; C6 backup/restore UX completion (progress, schedule-later, format choice); C7 Transfers activity job list (unify import/export/backup jobs).
Dependencies: grid + dialog patterns; cancellation contract (`query-runtime.md`); large-file streaming design (no full-materialize).
Safety: preview-before-write; transaction/rollback policy surfaced; readonly-gate; bounded memory; destructive-restore confirmation.
Tests: golden-file import/export per provider; cancellation mid-import; malformed-file matrices.
Runtime verification: 100k-row CSV round-trip evidence per provider; backup/restore live runs.
Exit criteria: matrix §9 Import rows PARTIAL-or-better; export BACKEND_ONLY eliminated; Transfers placeholder removed.

## Phase D — Monitoring / Administration (P1 core, P2 depth, v0.3)

Goal: see what the database is doing and act: sessions → queries → locks → sizes → maintenance.
User value: the difference between SQL client and database IDE/admin tool.
Existing reusable: almost nothing — only capability flags + `reltuples` estimates + cancel primitives. Greenfield behind new `AdminService` + port.
Missing architecture: `AdminService` + `AdminPort` (sessions, queries, locks, sizes, stats, vacuum/analyze, server info, terminate), Monitor activity UI, agent read-only tools for sessions/plans.

Features: D1 active sessions + terminate (PG); D2 running queries + cancel wiring (resolve §16.1 first); D3 locks + blocking graph (later: visual); D4 DB/table sizes + statistics; D5 VACUUM/ANALYZE actions with confirmation; D6 server information; D7 Monitor activity (Overview/Sessions/Queries/Locks/Sizes).
Dependencies: §16.1 cancel decision; worker cancellation maps; confirmation patterns.
Safety: terminate/cancel confirmations; read-only default views; no auto-kill anything; agent tools read-only.
Tests: PG live (Docker fixture like safety-hardening VERIFICATION); UI polling/refresh tests with fake port.
Runtime verification: live PG sessions/locks walkthrough; long-query cancel evidence.
Exit criteria: matrix §8 core rows (sessions/queries/cancel/sizes) PARTIAL-or-better on PG, correctly gated on SQLite; Monitor placeholder removed.

## Phase E — Users / Roles / Permissions (P2, v0.3)

Goal: expose the existing PG backend as a safe workbench.
User value: completes the admin story; backend is DONE and waiting.
Existing reusable: `UserService` + `PostgresUserManager` + allowlist quoting + writable/server gates.
Missing architecture: native UI only (users/roles/grants pages, membership editor, privilege matrix) + `UiCommand` set.

Features: E1 users/roles browser; E2 role editor (login/superuser/createdb); E3 membership management (missing backend — add service first); E4 grant matrix (table-level; schema-level new); E5 role DDL view.
Dependencies: none heavy; Phase A dialog patterns.
Safety: privilege allowlist (exists) + destructive confirmations + readonly-gate; never render passwords.
Tests: PG live; SQLite-gate tests.
Runtime verification: PG user/grant walkthrough on native UI.
Exit criteria: matrix §11 BACKEND_ONLY eliminated for scoped rows; membership gap closed or explicitly deferred.

## Phase F — Schema Compare / Migration (P2, v0.3 core / Later depth)

Goal: diff → migration SQL → safe apply.
User value: team workflows (dev→prod) without leaving the IDE.
Existing reusable: narrow `schema_diff` (tables/columns/indexes) + count-only `data_diff` + Tauri DTOs (promote, don't duplicate).
Missing architecture: diff depth (views/triggers/functions/constraints), DDL-diff generator (diff → ALTER/CREATE statements), migration preview/apply flow, Transfer sidebar entries.

Features: F1 schema-compare UI on existing backend; F2 diff depth expansion; F3 DDL-diff generation; F4 migration preview + SQL export; F5 safe apply via `execute_ddl_batch` + confirmation; F6 data compare depth (checksums/paging, Later).
Dependencies: Phase A DDL services (apply path); capability gating cross-provider (PG↔SQLite diffs must not emit invalid SQL).
Safety: preview-before-apply; transactional apply; dry-run; confirmation for destructive migrations.
Tests: fixture-pair diffs (PG→PG, SQLite→SQLite, PG→SQLite structural only).
Runtime verification: real migration preview→apply walkthrough.
Exit criteria: matrix §12 Schema Compare PARTIAL-or-better with UI; DDL-diff exists; apply gated and evidenced.

## Phase G — Productivity / Global Search (P2, v0.2–v0.3)

Goal: make daily work fast: find anything, reuse SQL, tune the editor.
User value: where DB Pro can feel better than DBeaver, not just equal.
Existing reusable: palette scaffold, saved queries + folders, local history, completion infra, grid-persist patterns, `RunConfig` repo (unwired).
Missing architecture: unified search index (connection/schema/table/column/view/function/query/command/agent-action); snippet library domain; favorites/recent/pinned models; editor settings page + persistence; connection folders/tags UI (domain already supports).

Features: G1 palette search expansion (columns/views/functions/saved-query SQL/history SQL/settings/commands); G2 dedicated Search activity on the unified index; G3 snippet library (CRUD, variables, sharing via export); G4 scratch SQL docs; G5 favorites/recent/pinned (objects + queries + connections); G6 connection folders/tags/favorites UI (wire existing domain); G7 editor settings page (font/completion/format/diagnostics) + provider settings (timeout/max rows) via `RunConfig`; G8 query parameters dialog (unblocks typed execution + binary-cell hint); G9 staged-copy fix + JSON tree viewer + DEFAULT handling.
Dependencies: small, mostly UI + local/ meta-store work. G8 shared with Phase B arg forms.
Safety: search index must never include secrets (see `security-boundaries.md`); params always parameterized.
Tests: UI-heavy; search-index unit tests; no provider matrix needed except where DB-backed.
Runtime verification: native walkthroughs + screenshots; keyboard-only pass.
Exit criteria: matrix §13/§14 MISSING rows for scoped items eliminated; Search activity real; §16.5–16.7 paper cuts fixed.

## Phase H — Advanced AI (P3, Later)

Goal: assistant workflows on top of stable tools — never autonomous destruction.
User value: optimize/migrate/administer with the agent, safely.
Existing reusable: 9 typed tools, Ask/Edit/Agent modes, confirmation + anti-TOCTOU pipeline, context chips.
Missing architecture: new tools (`compare_schema`, `generate_migration`, `analyze_slow_query`, `inspect_sampled_data`, sessions/plan tools) + their backends (Phases D/F first) + data-analysis rendering.

Features: H1 slow-query analysis (needs D); H2 index suggestions with explicit apply confirmation; H3 migration assistant (needs F); H4 schema-compare assistant (needs F); H5 admin assistant (needs D, read-only tools first); H6 data-analysis assistant (sampling + charts-later).
Dependencies: Phases D and F backends; monitoring + diff depth.
Safety: new tools start read-only; any generated DDL goes through preview + confirmation; no autonomous destructive operations (parity-note rule retained).
Tests: tool-level + workflow-level (idempotency, replay, stale-version).
Runtime verification: live provider E2E per assistant.
Exit criteria: each assistant independently evidenced; P0/P1 zero.

---

## Sidebar proposal (FINAL)

Keep the compact 9-icon rail from the vision. Adjustments from audit, with reasons:

```text
▣ Explorer   — as today (WIRED)
⌕ Search     — NEW in Phase G (palette graduates to full activity; reason: palette already exists, tables-only limit is the #1 findability gap)
⌘ Queries    — as today, absorbs History as a section (reason: History is already a Queries-adjacent sidebar; separate rail icon wastes top-level space)
◫ Data       — NEW icon in Phase C (reason: table/query tabs exist but transfer/data tools need a home; until then, no icon — do not add empty icons)
◇ ER         — as today
✦ Agent      — as today (Preview until H)
▥ Monitoring — icon stays, becomes real in Phase D (placeholder removed then, not before)
⇄ Transfer   — icon stays, becomes real in Phase C (placeholder removed then, not before)
⚙ Settings   — as today, grows into sectioned pages in Phase G (Connections/Editor/Appearance/AI/Keybindings/Database/Advanced/Privacy)
```

Deferred top-level: Security (lives under Explorer → database → Security + Phase E pages), Tasks/Jobs (no scheduler exists; Later), Favorites (a Search/Queries filter, not an icon).

Rule: no new rail icon ships as placeholder. Icons appear with the phase that wires them.

---

## DB Pro vs mature database client capability

Already Strong: connection lifecycle + safety policy; SQL editor intelligence; staged data editing with 3-way conflicts; provider capability gating; typed agent workflow with confirmations; ER large-schema architecture; reconstructed dialect-aware DDL; backup engines (both providers); export engine breadth (CSV/JSON/XLSX + precision guards).

Competitive: explorer introspection depth (tables/views/columns/constraints/indexes/triggers/functions); multi-result + explain plumbing; saved queries + history + drafts; composite PK/FK handling.

Partial: views/triggers/functions management; query params/snippets/scratch; export wiring; backup/restore UX; search/palette; editor settings; conflict-free provider matrix wording.

Major Gap: typed object-CRUD workbench; sequences/types/enums/matviews; routine execution; import (all formats); monitoring/admin (all); users/roles UI; schema-compare depth + migration flow; global search; keybindings; connection organization UI.

Intentionally Deferred: ER design/edit mode; DB-to-DB transfer; job scheduler; MCP; additional drivers (PG + SQLite only per LIM-002); server-side extras (replication, pools, extensions) — revisit post-v0.3.

---

## Technical debt (new + pointers)

Pointers: `docs/architecture/backend-debt-register.md` (P1-01/P1-02 cross-connection bypass, P1-04 `parse_connection_id` ×6, P2-05 `CellValue`/`QueryParam`, P3-0x cosmetics), `docs/release/risk-register.md` (R001 brand, R004 license, R003/R009 signing, R006 pg_dump PATH, R009 ssh binary).

New from this audit (not fixes in this task):

- Architecture: `execute_ddl` ignores capability flags (only user-manager/cancel/backup/functions gate); cross-connection `PostgresApi` bypasses application layer; DTO triplication (tauri/runtime/native); two cancellation registries (tauri `ExecutionRegistry` vs worker maps); three `AgentContext` namings; `result_grid` vs `components/table` overlap.
- Provider: cancel flag/matrix inversion; flag-without-code (sequences/enums/sessions/partitions); rename only via PG inherent method; SQLite index/trigger/check fidelity loss; no PG benches.
- Native UI: export-backend unwired + separate unquoted CSV writer; server history unwired; `RunConfig` unwired; connection tags dropped; Ctrl+N hint unwired; tab-close doesn't cancel; staged-copy gap; binary/default placeholders; query results unbounded in memory; diagram positions not persisted.
- Tests: PG live suites `--ignored` without fixture; SSH E2E pending; coverage % unmeasured; visual gate (3 resolutions × states) pending.
- Runtime: every active plan in RUNTIME_VERIFY; native packaging/installers unproven; tauri-app blocks workspace check.
- Performance: no native paint/scroll harness; no ER large-schema Criterion bench; external processes (pg_dump/ssh/keyring) unbudgeted.

---

## Implementation order (20 milestones, each independently implementable + verifiable)

1. M1 View CRUD (builders exist; add replace/alter + native UI + PG/SQLite evidence)
2. M2 Index create/drop wizard UI (builders exist)
3. M3 FK create/drop UI (new service; DDL preview + confirm)
4. M4 Table create/alter form UI (column/constraint composition over builders)
5. M5 Trigger editor + PG enable/disable (live PG evidence)
6. M6 Sequences PG (catalog + domain + service + UI)
7. M7 Types/Enums/Domains PG (catalog + domain + service + UI)
8. M8 Materialized views PG (catalog + refresh + UI)
9. M9 Routine explorer + source view polish (signatures/overloads/search)
10. M10 Routine execute form (typed args + results; PG)
11. M11 Routine create/replace/drop + dependencies
12. M12 CSV import wizard (preview/mapping/txn/progress/cancel)
13. M13 JSON/Excel import + wire backend exporters + SQL INSERT/COPY export
14. M14 Backup/restore UX completion + Transfers job list
15. M15 Monitoring core (sessions/queries/cancel/terminate; PG) + Monitor activity
16. M16 Monitoring depth (locks/sizes/stats/vacuum/analyze/server info)
17. M17 Users/roles/grants UI (+ memberships backend; PG)
18. M18 Schema-compare UI + diff-depth expansion + migration preview/SQL/apply
19. M19 Unified global search (index + Search activity + palette expansion)
20. M20 Productivity completion (snippets, scratch, favorites/recent/pinned, params dialog, editor/provider settings, connection folders/tags, staged-copy + JSON tree + keybinding docs)

Suggested release packing: v0.2 = M1–M5, M9–M11, M19-search-part; v0.3 = M6–M8, M12–M17; Later = M18-depth, M20-remainder, H-assistants, ER design, DB-to-DB transfer.

Each milestone PR must reference its plan dir (`PLAN/CHECKLIST/FINDINGS/VERIFICATION`), record PG+SQLite disposition separately, and cannot close on source evidence alone.
