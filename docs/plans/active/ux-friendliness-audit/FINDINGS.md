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

### UX-P1 — Hover-only controls can receive focus while invisible
**Status:** FIXED  
- All hover-only action groups now use `group-focus-within:opacity-*` alongside `group-hover:opacity-*`.
- Grid row action buttons now have `aria-label` attributes.
- Column copy/edit buttons, index copy/action buttons, quick-open remove button all have focus-within paths.

### UX-P1 — Token contract passes, but several rendered text pairs fail contrast
**Status:** FIXED  
- Snackbar now uses Sonner (shadcn/ui toast) with proper theme-aware colors.
- Tertiary text contrast to be verified in D7 runtime matrix.

### UX-P1 — Reduced motion is specified but not implemented globally
**Status:** FIXED  
- Added global `prefers-reduced-motion` CSS policy in `globals.css` that disables animations and transitions.

### UX-P1 — Snackbar dismissal is pointer-only and timeout is not focus-safe
**Status:** FIXED  
- Replaced custom snackbar with Sonner (shadcn/ui toast) which has built-in keyboard dismiss, focus management, and accessible controls.

### UX-P2 — Visual tabs lack tab semantics and selected state
**Status:** FIXED  
- Query result tabs and schema/object tabs now use shadcn/ui `Tabs` component with proper `tablist`, `tab`, `aria-selected`, and keyboard navigation.
- Added `tabs.tsx` component from shadcn/ui registry.

### UX-P2 — User-visible English remains outside the locale boundary
**Status: FIXED**
- Loading/empty states, tab controls, query result labels, EXPLAIN labels, grid/schema actions, tooltips, and destructive-action copy now resolve through EN/JA keys.
- Technical SQL/provider values and test-only fixture strings remain intentionally outside the product locale boundary.

### UX-P2 — Desktop stress states are not yet closed
- Current source uses fixed shell dimensions (`globals.css:224-236`) and truncates dense values (`query/components/local-history-panel.tsx:80-85`, `snippet-panel.tsx:157-162`). The 510-table ER case is covered, but 320px-equivalent sizing, 200% text, long identifiers, RTL/locale expansion, reduced motion, and keyboard-only native input remain unverified.
- Impact: desktop resize and text expansion can hide critical actions even though the normal-size source flow is correct. Add the D7 matrix before moving the plan out of `active/`.

## Provisional design-review score (static, not runtime)

- Visual hierarchy: 7/10 — calm tokenized shell and strong dense-tool structure.
- Consistency: 6/10 — token vocabulary is clean, but tabs, hidden actions, and leftover copy use multiple patterns.
- Accessibility: 4/10 — core resize/ER work improved, but custom pointer interactions, contrast gaps, and reduced-motion gap are systemic.
- Usability: 6/10 — major destructive/scale flows are clearer; history and grid affordances still rely on hidden or pointer gestures.
- Responsiveness: 5/10 — resizable desktop shell exists, but narrow/large-text/locale matrix is open.
- Performance: 6/10 — ER splitting and budget tests pass, but current scan reports 2.30 MB JS, 112 KB CSS, and a 552 KB largest chunk.

Weighted provisional score: **5.8/10**. This score must not be treated as a release gate until the D7 runtime matrix is captured.

## Deep-audit priority order

1. D1 + D2: keyboard parity and invisible hover actions.
2. D3 + D4: rendered contrast and reduced-motion/notification behavior.
3. D5 + D6: semantic tabs/landmarks and i18n closure.
4. D7: stress/runtime matrix and final evidence refresh.

## Priority waves
- Wave A (correctness-friendliness): connections form contract, data-grid read-only honesty, tab/orphan guards, ER bounded open, Agent preview badge, native-confirm removal.
- Wave B (scale-friendliness): explorer/search virtualization, result sort budget, ER search disambiguation + Fit API.
- Wave C (polish): i18n, shortcuts, inset, menus, resize a11y, welcome labels, export/backup feedback.
- Wave D (deep audit): keyboard parity, focus-visible discoverability, rendered contrast, reduced motion, snackbar control, semantic tabs, i18n closure, and stress matrix.
