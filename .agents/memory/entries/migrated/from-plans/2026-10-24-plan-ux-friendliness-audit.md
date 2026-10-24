---
schema_version: 1
id: "MEM-PLAN-ux-friendliness-audit-90f970b6232d"
title: "Plan archive: Ux Friendliness Audit"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, ux-friendliness-audit]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "90f970b6232dad136943c90fa809ca32504e09fa3e9671d1615d144f141901a3"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/ux-friendliness-audit/PLAN.md"
  - "docs/plans/active/ux-friendliness-audit/FINDINGS.md"
  - "docs/plans/active/ux-friendliness-audit/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/ux-friendliness-audit/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# UX Friendliness Audit — Plan

Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`. Current: `RUNTIME_VERIFY`.
Branch: `main` (user-directed override; repository default is feature branch + PR). Baseline: `main` + RC1 audit `qa/rc1-static-audit` (P1: 14, P2: 25).

## Goal
Audit every user-facing feature for UX friendliness (clarity, forgiveness, feedback, discoverability, consistency) and produce a prioritized, implementable remediation plan. No behavior change in this planning step — source evidence only.

## Scope (feature areas)
1. Welcome / onboarding 2. Connections (list, dialog, test, SSH, SQLite) 3. Explorer / sidebar search 4. Workspace tabs (preview, pinned, close guard, orphan recovery) 5. Query editor (editing, history, saved, snippets, run configs) 6. Results grid + EXPLAIN 7. Data grid editing (staging, commit, delete) 8. Schema inspector (columns, FK, indexes, triggers, DDL, CRUD gen) 9. ER diagram (search-first, LOD, minimap, overview) 10. Export/Import/Backup 11. User management 12. Command palette / Quick Open / Agent panel 13. Shell chrome (topbar, statusbar, docks, shortcuts, i18n, tokens)

## Non-goals
- No provider runtime claims until runtime evidence is collected; no new Agent/MCP features (RC1 freeze respected).

## Method (per feature)
Heuristics: visibility of status, match with mental model, error prevention/recovery, feedback within 1s, empty/loading/error states, destructive-action guard, keyboard + a11y, consistency (tokens, shortcuts, dialogs), provider-appropriate language (PG vs SQLite).

## Acceptance
- `PLAN/CHECKLIST/FINDINGS/VERIFICATION.md` exist under `docs/plans/active/ux-friendliness-audit/`
- Every feature has ≥1 friendliness verdict + evidence file path
- Findings ranked UX-P1 (blocks/confuses/misles primary flow) / UX-P2 (friction/polish)
- No COMPLETED claim; runtime verification explicitly pending.


## Source: `FINDINGS.md`

# UX Friendliness Audit — Findings (source and runtime evidence)

## Per-feature verdicts

### 1. Welcome / onboarding — Friendly with gaps
- Good: recent connections + shortcuts give fast resume (`commons/components/welcome-view.tsx`).
- Resolved (2026-09-09, RC1 QA-P2-21): SQLite recent connections show the database path instead of a meaningless host/port pair.
- Resolved (2026-09-09): empty first-run state explains that there are no connections and guides the user to create the first one.

### 2. Connections — Least friendly, most fixes needed
- Resolved (2026-09-09, QA-P1-05..08): SQLite no longer requires a password, SQLite strips stale SSH state, SSH defaults port 22 and preserves omitted passphrase as `undefined`, and connection-dialog callbacks are guarded by a per-open session generation.
- Resolved (2026-09-09, QA-P2-23): connection deletion from both the row action and context menu uses the shared app `AlertDialog`, with localized confirm/cancel labels.
- Resolved (2026-09-09, QA-P2-15..19): test results reset after edits, backend `userMessage` is surfaced, SQLite Browse reports picker failures, driver-change password requirements follow the current driver, and duplicate now explains that credentials are intentionally not copied. The connection-list context menu also has a real table-row trigger instead of attaching Radix handlers to a fragment.
- Resolved (2026-09-09, QA-P2-20): favorite state rolls back when persistence fails.
- Runtime evidence (2026-09-09): native Tauri walkthroughs completed for PostgreSQL (`UX Fixture` → `public` → `orders`) and SQLite (`A4 SQLite` → `main` → `typed_values`), including connection/introspection, table data, column metadata, and DDL views.

### 3. Explorer / sidebar search — Functional, not large-schema friendly
- Resolved (2026-09-09, QA-P2-13): sidebar search builds a memoized table/view catalog, debounces input, virtualizes matching rows, and reports match counts.
- Resolved (2026-09-09, QA-P2-14): expanded schema table/view groups render through a bounded virtualized child list while preserving row context menus.
- Resolved (2026-09-09): selecting a connection now collapses unrelated connection/schema groups while preserving the selected tree.
- Plan: collect runtime evidence on very large schemas.

### 4. Workspace tabs — Powerful but unpredictable
- Fixed in W1: preview-carries-staged-changes, staged bypasses close guard (QA-P1-02/03) — verify by runtime.
- Resolved (2026-09-09, QA-P1-10/11): tab close commands/actions now share the guarded close service; orphan query reassignment resets connection context, while orphan DB-object/ER tabs close and select the target connection for a fresh resource pick.
- Resolved (2026-09-09, QA-P2-01): keyboard navigation and context-menu relative ordering now use the same pinned-first order rendered by the tab bar.
- Resolved (2026-09-09, QA-P2-02): shortcut labels use the platform-aware formatter; macOS shows symbols and other platforms show Ctrl text.
- Plan: collect runtime evidence for preview/staged close and orphan recovery flows.

### 5. Query editor — Strong core, weak forgiveness
- Good: Monaco + dialects + risk classifier + timer + zoom.
- Resolved (2026-09-09, QA-P2-23): query history/import overwrite uses the shared app `AlertDialog` confirmation with localized action labels; no native `confirm()` remains in the query editor.
- Resolved (2026-09-09, QA-P2-22): export is gated by result state rather than SQL text (`9dcb5b7`). Query empty/error/success/cancelled/messages copy now uses localized `query.*` keys, and a known disconnected query connection exposes a reconnect affordance.
- UX-P2 remains: runtime/provider evidence.
- Plan: picker shows reconnect affordance; finish result/status i18n.

### 6. Results grid — Readable, needs scale honesty
- Resolved (2026-09-09, QA-P2-12): sorted large result sets now show an explicit localized warning at the existing 10,000-row threshold; the existing status bar also surfaces large-result state.
- Resolved (2026-09-09, QA-P1-04/14): native PG and SQLite result grids preserve provider metadata and typed values; PostgreSQL numeric/enum mapping was fixed in `5958698` and covered by ignored PG integration coverage.
- Plan: retain provider/runtime evidence for future regression coverage.

### 7. Data grid editing — Dangerous affordances
- Resolved (2026-09-09, QA-P2-08): read-only connections show a clear banner and prevent cell editing before staging, rather than failing only at Apply.
- Resolved (2026-09-09, QA-P2-07/09): column visibility now has one event owner, and the custom grid context menu clamps to the viewport, receives focus, and closes on Escape.
- Resolved (2026-09-09, QA-P2-10/11): grid resize handles now expose separator semantics and Arrow-key resizing; unmount cleanup removes active document listeners and restores body styles.
- Plan: collect runtime evidence for the source-complete column toggle, Radix menu, keyboard resize, and cleanup flow.

### 8. Schema inspector (columns/FK/indexes/triggers/DDL/CRUD) — Complete but intimidating
- Good: per-object tabs + DDL viewer + capabilities gating.
- Resolved (2026-09-09): column mutations show operation/risk/warning/SQL before confirmation, and the DDL editor now requires a confirmation showing the operation, target, and SQL preview before every execution.
- Resolved (2026-09-09, QA-P2-06): ColumnEditDialog action/confirmation labels and classifier risk/warning copy now use localized EN/JA schema keys; mounted CRUD preview now labels its copy action as SQL. SQLite and PostgreSQL trigger lifecycle/introspection are covered by Rust runtime tests; packaged UI proof is complete.
- Plan: collect live PostgreSQL trigger and provider/runtime evidence.

### 9. ER diagram — Impressive, unfriendly at scale
- Resolved (2026-09-09, QA-P1-12/13; implementation in `59018df`): >200-table and L/XL schemas open in search-first mode with no renderer/layout; selecting a table materializes a bounded neighborhood and safe compact first paint; full overview requires explicit Show All.
- Resolved (2026-09-09, QA-P2-24/25): ER search renders schema-qualified candidates for disambiguation, Enter/click selects the highlighted candidate, and Fit calls the renderer API directly.
- Resolved (2026-09-09): when the React Flow MiniMap is omitted, the canvas explains why and points to overview controls.
- Runtime evidence (2026-09-09): Tauri dev opened a PostgreSQL fixture with 500 temporary tables (510 tables in the ER model) directly into the search-first state; the accessibility tree exposed the search entry, table count, and schema-qualified suggestions without mounting the graph renderer. The theme control also cycled through Dark and System.
- Resolved (2026-09-09): packaged desktop smoke now renders the full application after the production Vite chunk-cycle fix (`f28f6dc`); release UI showed Explorer, workspace/table tabs, and table data.
- Runtime evidence (2026-09-09): the schema row now exposes a direct `Open ER Diagram: <schema>` action; native Tauri opened `ER: public` and exposed the diagram/overview tabs, Fit View control, search entry, MiniMap, and the 10-table graph.
- Resolved (2026-09-09): React Flow table nodes expose schema-qualified accessible names and a keyboard focus action; Enter/Space now follows the same neighborhood-focus or open-table path as pointer selection.
- Runtime evidence (2026-09-09): on a 510-table PostgreSQL fixture, activating the schema-qualified `public.orders` search result opened a bounded 3-table neighborhood. The native tree exposed and activated `Show all 510 tables`, which transitioned to the full overview; the neighborhood tree exposed and activated `Fit View`. The remaining runtime gap is keyboard-only input because the Wayland desktop provider cannot focus the Tauri window for key injection.

### 10. Export/Import/Backup — Works, low feedback
- Resolved (2026-09-09, partial): the mounted results export dialog now performs pre-flight validation for empty results, missing SQL table name, and missing Excel connection/query before download or backend mutation. Validation messages are localized and targeted coverage is present.
- Resolved (2026-09-09, partial): the existing backup service is now mounted from the Explorer connection context menu. Native save/open pickers filter by provider/format; SQLite exposes file backup only, PostgreSQL exposes plain/custom formats, restore requires an impact warning, and operations report progress/success/failure through the shared snackbar.
- Resolved (2026-09-09): backup and restore now emit started/completed/failed backend progress events, and the completed operation keeps a native “Reveal location” action with platform-specific folder opening and missing-path feedback.
- Import parse file/line context remains not applicable: the current product has no import UI or import command to attach that error contract to.

### 11. User management (PG-only) — Capability cliff
- Resolved (2026-09-09): the Users sidebar gates by the selected connection driver and explains why SQLite has no role/privilege management surface instead of falling through to an empty/error state. PostgreSQL now lists roles, shows the selected role's table-privilege count, and requires an impact-summary AlertDialog before `DROP ROLE`.
- Plan: collect PostgreSQL/SQLite provider and runtime evidence for the source-complete gated role flow.

### 12. Command palette / Quick Open / Agent — Discoverable vs misleading
- Good: palette + quick open + ranking exist.
- Resolved (2026-09-09, QA-P2-04/05): Agent preview affordances and platform-aware shortcut labels are implemented; runtime verification remains pending.
- Plan: keep runtime evidence for the preview-only flow.

**Resolution (2026-09-09, QA-P2-04):** Agent now shows a localized `Preview` badge; starter actions and the composer are disabled, with an explicit preview-only explanation. The existing platform-aware shortcut fix covers QA-P2-05. Full pnpm frontend gates pass; provider and packaged-desktop runtime verification remain pending.

### 13. Shell chrome / i18n / tokens — Inconsistent polish
- Resolved (2026-09-09): topbar traffic-light spacing is macOS-only; sidebar and Agent resize handles expose keyboard arrows, separator semantics, focusability, and bounded ARIA values.
- Resolved (2026-09-09, QA-P2-06): ER overview controls, search, stats, candidate details, and the full-schema action now use localized `schemaWorkspace.*` labels; runtime/token smoke remains pending.
- Plan: collect light/dark token smoke and finish only remaining user-visible i18n gaps.

## 2026 trend lens (applied in IMPROVEMENT_PLAN.md)
Copilot-not-autopilot · purposeful trust-building motion · raw schematic clarity (mono+grid) · user-controlled motion/a11y · fluid `clamp()` type · off-white comfort + adaptive dark · bento modular blocks · micro-interaction feedback · anti-liquid-glass legibility · dense dark-first dev-tool idiom. Detail per wave: `IMPROVEMENT_PLAN.md`.

## 2026-09-10 deep UI/UX audit — remaining findings

This is a static/source audit using the installed UI/UX, accessibility, desktop-HIG, and design-review skills. It does not replace browser/desktop evidence. Findings are consolidated by systemic cause so the same fix is not repeated in every screen.

### UX-P1 — Pointer-only actions remain in core workflows
**Status:** FIXED  
- Query history, local history, snippets: clickable `div`s converted to native `button` elements with keyboard handlers.
- EXPLAIN tree nodes: converted to `button` with `role="treeitem"` and keyboard support.
- ER detailed column rows: converted to `button` with keyboard handlers.
- Grid row selection: row number is a keyboard-focusable `role="button"` control with Enter/Space support, preserving the grid's existing row-cell structure.
- Grid sortable headers: added `role="button"`, `tabIndex={0}`, and keyboard handlers.

### UX-P1 — Hover-only controls can receive focus while invi

… [truncated for memory entry; see source file]


## Source: `VERIFICATION.md`

# UX Friendliness Audit — Verification

- Source evidence: STATUS.md, FEATURE_LIFECYCLE.md, RC1 PLAN/FINDINGS/WAVE1_REPORT read; frontend `src/routes`, `commons/components/shell`, `modules/{connection,query,data-grid,schema,er-diagram,export,backup,user-management}` inventoried; `crates/tauri-app/src/lib.rs:157-213` command groups inventoried.
- Automated: `pnpm install --frozen-lockfile`, `pnpm run typecheck`, `pnpm run lint` (0 errors, 3 pre-existing warnings), `pnpm run format:check`, `pnpm run check:tokens`, and `pnpm run build` pass. The reliable full frontend run `pnpm exec vitest run --pool=threads --maxWorkers=2` passes 147 files / 1733 tests; the unconstrained default remains resource-starved by worker-start failures in this environment. The backup change has targeted service coverage (7 tests) and a Tauri missing-directory regression test. The frontend now uses `pnpm-lock.yaml`; the invalid/stale frontend npm lockfile and registry resolutions are no longer used. Build output has the existing dynamic-import/chunk-size warnings; the production vendor chunk cycle is fixed.
- Wave A3 regression evidence: `request-close-tab.test.ts` and `workspace-reassign.test.ts` pass (11 tests); production close paths use the shared guarded request service, and orphan object tabs are closed before target-resource re-pick.
- Wave A1 regression evidence: `pnpm exec vitest run src/modules/connection/__tests__/connection-lifecycle-p1.test.tsx src/modules/connection/__tests__/connection-editor.test.tsx src/modules/connection/__tests__/connection-dialog.test.tsx --testTimeout=15000` passes (3 files, 29 tests), covering SQLite credentials/SSH stripping, SSH defaults, stale test state, Browse failure feedback, backend `userMessage`, driver-change password handling, and late callback session guards.
- Wave A2 regression evidence: `pnpm exec vitest run src/modules/data-grid/__tests__/data-grid.test.tsx src/modules/data-grid/__tests__/change-bar.test.tsx` passes (2 files, 19 tests), including read-only editing prevention and change-bar behavior.
- Wave A4 regression evidence: `pnpm exec vitest run src/modules/er-diagram/__tests__/search-first-entry.test.tsx src/modules/er-diagram/__tests__/large-schema.test.ts src/modules/er-diagram/__tests__/show-all-transition.test.tsx src/modules/er-diagram/__tests__/er-diagram-navigation.test.tsx src/modules/er-diagram/__tests__/viewport-lod-integration.test.tsx` passes (5 files, 100 tests). The suite proves search-first initial render, no initial renderer/layout, bounded `<=100` neighborhood materialization, compact first paint, explicit Show All overview activation, navigation state, and viewport LOD behavior.
- Wave A5 regression evidence: `pnpm exec vitest run src/modules/connection/__tests__/connection-list.test.tsx` passes (7 tests); the delete action opens the shared `AlertDialog` before mutation. Source inspection confirms query history/import and both connection-delete paths use `useConfirmDialog`, and `rg` finds no native `confirm()` call under `frontend/src`.
- Wave A6 source evidence: column mutation flow renders classified operation/risk/warning/SQL before confirmation; the DDL editor now routes every previewed statement through an impact-summary `AlertDialog` before `execute_ddl`.
- Wave B explorer source evidence: `SearchView` now debounces catalog filtering, matches table/view names or schemas, renders only virtualized result rows, and displays a localized result count. `SchemaObjectGroup` now virtualizes expanded table/view children with bounded scroll height; connection switching collapses stale trees while preserving the selected tree. Runtime evidence remains open.
- Wave B result-sort evidence: `ResultGrid` displays a localized warning when sorting at least 10,000 returned rows; existing `QueryStatusBar` covers the large-result state. Targeted result-grid regression coverage added.
- Wave B ER evidence: search candidates display `schema.table`, selection remains explicit via highlighted Enter/click, and Fit uses `rendererRef.current?.fit()`. React Flow shows a localized explanation when MiniMap is omitted for large graphs.
- Wave B ER keyboard evidence: `pnpm exec vitest run src/modules/er-diagram/__tests__/er-table-node.test.tsx src/modules/er-diagram/__tests__/search-first-entry.test.tsx --testTimeout=15000` passes (2 files, 41 tests); React Flow table nodes now expose schema-qualified accessible names and Enter/Space focus behavior. Native keyboard-only end-to-end remains pending because the current Wayland desktop provider cannot focus the Tauri window for key injection.
- Wave C welcome evidence: `pnpm exec vitest run src/commons/components/__tests__/welcome-view.test.tsx` passes (7 tests), including first-run empty state and SQLite path-focused recent connection rendering.
- Wave C shell source evidence: topbar padding is gated by `isMac`; both resize separators are keyboard-focusable with Arrow-key resizing and min/max/now ARIA values.
- Wave C user-management evidence: `pnpm exec vitest run src/commons/components/__tests__/users-view.test.tsx` passes (4 tests); the sidebar explains the PostgreSQL-only capability for SQLite and the PostgreSQL path lists roles and confirms `DROP ROLE` with its table-privilege count.
- Wave C export evidence: `pnpm exec vitest run src/modules/export/__tests__/export-validation.test.ts src/modules/export/__tests__/export-generators.test.ts` passes (2 files, 18 tests); empty results, SQL table-name, and Excel connection/query pre-flight failures are localized and block execution.
- Wave C backup evidence: `d1b1936` mounts the existing backup/restore service from the Explorer connection context menu. `pnpm exec vitest run src/modules/backup/__tests__/backup-queries.test.tsx src/modules/backup/__tests__/backup-service.test.tsx --testTimeout=15000` passes (2 files, 7 tests); the dialog uses native save/open pickers, gates custom format for SQLite, confirms restore, keeps operation state visible, consumes backend `backup-progress` events, and exposes a native reveal-location action. `cargo test -p db-pro-tauri reveal_rejects_a_missing_directory` passes. Import parse context is not applicable until an import UI exists.
- Wave C query i18n evidence: `pnpm exec vitest run src/modules/query/__tests__` passes (23 files, 259 tests); query result, explain, completion, cancellation, and message-state copy now resolves through EN/JA locale keys.
- Wave C ER i18n evidence: `pnpm exec vitest run src/modules/er-diagram/__tests__/search-first-entry.test.tsx src/modules/er-diagram/__tests__/show-all-transition.test.tsx src/modules/er-diagram/__tests__/cytoscape-view.test.tsx` passes (3 files, 49 tests); overview controls and Show All labels resolve through EN/JA locale keys.
- Wave C ER copy evidence: `pnpm exec vitest run src/modules/er-diagram/__tests__/search-suggestions.test.tsx src/modules/er-diagram/__tests__/cytoscape-view.test.tsx src/modules/er-diagram/__tests__/search-first-entry.test.tsx` passes (3 files, 45 tests); search, counts, candidate details, empty state, hop scope, and overview labels resolve through EN/JA locale keys.
- Wave B explorer evidence: `pnpm exec vitest run src/commons/stores/__tests__/explorer.store.test.ts src/commons/components/shell/sidebar-views/search-view.test.tsx` passes (2 files, 12 tests); switching connections filters stale expanded trees while preserving the selected connection/schema paths.
- Wave C workspace evidence: `pnpm exec vitest run src/hooks/__tests__/use-tab-keyboard.test.ts` passes (1 test); keyboard/context relative order now matches the pinned-first tab-bar projection.
- Wave C query connection evidence: `pnpm exec vitest run src/modules/query/__tests__/query-command-bar.test.tsx` passes (4 tests); known disconnected connections expose the localized reconnect action while missing configurations remain explicitly unavailable.
- Wave A data-grid evidence: `pnpm exec vitest run src/modules/data-grid/__tests__/data-grid.test.tsx src/modules/data-grid/__tests__/change-bar.test.tsx src/modules/unified-grid/__tests__/unified-grid-copy.test.tsx` passes (3 files, 25 tests); column visibility uses one toggle path and grid context menus clamp/focus/close with Escape.
- Wave A data-grid resize evidence: the same unified-grid test file includes keyboard separator coverage; Arrow-right changes the controlled width from 150 to 160 and the handle exposes min/current ARIA values.
- Wave A connection evidence: `pnpm exec vitest run src/modules/connection/__tests__/connection-crud-queries.test.tsx` covers favorite rollback after persistence failure.
- Wave A schema evidence: the ColumnEditDialog action/confirmation labels now resolve through EN/JA schema locale keys; targeted schema DDL/capability tests remain green from the prior A6 gate.
- Wave A schema copy evidence: `pnpm exec vitest run src/modules/schema/__tests__/column-mutation-risk.test.ts src/modules/schema/__tests__/ddl-capabilities.test.ts src/modules/schema/__tests__/ddl-capabilities-comprehensive.test.ts` passes (3 files, 97 tests); risk levels, confirmation warnings, and dynamic classifier warnings resolve through EN/JA keys at the render boundary.
- Wave A connection evidence: duplicate context-menu activation uses a real table-row trigger, and successful duplication shows that credentials were intentionally not copied; `connection-list.test.tsx` passes (7 tests).
- Rust gates: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets`, and `cargo test --workspace` pass. With the repo PostgreSQL fixture healthy, `DATABASE_URL=postgres://dbpro:dbpro_test@127.0.0.1:5432/dbpro_fixture cargo test -p db-pro-infrastructure --test pg_integration -- --ignored` also passes (9/9); SQLite integration/runtime suites pass in the workspace run.
- Performance audit: `bash .skills/perf-audit/scripts/perf-scan.sh` reports 10 budget tests passed and clippy clean, but fails the existing aggregate budgets (current scan: 2.30 MB JS > 2 MB; 112 KB CSS > 100 KB; largest JS chunk 552 KB). ER code-splitting and chunk-count checks pass; Rust benchmarks and ER/DB runtime checks are skipped by the `all` scan.
- Deep UI/UX source audit (2026-09-10): `pnpm run check:tokens`, `pnpm run typecheck`, `pnpm run format:check`, and `pnpm run build` pass; `pnpm run lint` passes with the existing 3 `no-explicit-any` warnings. The implementation closes the audited keyboard parity, focus-within discoverability, reduced-motion, snackbar, semantic-tab, and EN/JA copy gaps. Findings and Wave D remain recorded in `FINDINGS.md` and `IMPROVEMENT_PLAN.md`.
- Contrast evidence (static token calculation, not browser-rendered proof): light tertiary on white ≈2.56:1, dark tertiary on common dark surfaces ≈3.69–4.08:1, light accent on white ≈4.47:1, white on light semantic solids ≈3.19–3.77:1. A browser-rendered alpha/state matrix is still required before classifying the final result.
- Full Vitest evidence: `pnpm exec vitest run --pool=threads --maxWorkers=2 --reporter=dot` passes 147 files / 1733 tests. Targeted post-fix query/ER coverage passes 25 files / 301 tests; the ConnectionEditor fixture repair passes 2 files / 16 tests.
- Provider runtime: PASS for PostgreSQL and SQLite backend suites and native UI walkthroughs. PostgreSQL `UX Fixture → public → orders` showed live typed enum/numeric values after `5958698`; SQLite `A4 SQLite → main → typed_values` showed INTEGER/REAL/TEXT/BLOB/NULL rows, column metadata, and DDL.
- A4 desktop runtime: PASS for the search-first entry. With 500 temporary PostgreSQL tables added to `dbpro_fixture`, the Tauri dev window opened `ER: public`; the accessibility tree showed `510 tables`, the search entry, and schema-qualified suggestions while no graph renderer was mounted. The theme control was observed in Light, Dark, and System states. The fixture tables were dropped and the temporary `UX Scale` connection/cache were removed after the run.
- A4 native navigation evidence: the schema r

… [truncated for memory entry; see source file]

