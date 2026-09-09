# UX Friendliness Audit — Findings (source evidence only)

## Per-feature verdicts

### 1. Welcome / onboarding — Friendly with gaps
- Good: recent connections + shortcuts give fast resume (`commons/components/welcome-view.tsx`).
- Resolved (2026-09-09, RC1 QA-P2-21): SQLite recent connections show the database path instead of a meaningless host/port pair.
- Resolved (2026-09-09): empty first-run state explains that there are no connections and guides the user to create the first one.

### 2. Connections — Least friendly, most fixes needed
- Resolved (2026-09-09, QA-P1-05..08): SQLite no longer requires a password, SQLite strips stale SSH state, SSH defaults port 22 and preserves omitted passphrase as `undefined`, and connection-dialog callbacks are guarded by a per-open session generation.
- Resolved (2026-09-09, QA-P2-23): connection deletion from both the row action and context menu uses the shared app `AlertDialog`, with localized confirm/cancel labels.
- UX-P2: Test success goes stale after edits; failure hides backend detail; SQLite Browse has no error path; driverChanged is "ever changed"; duplicate omits credentials silently; favorite has no rollback (QA-P2-15–20).
- Plan: explicit credential messaging, complete remaining test/browse/duplicate/favorite polish.

### 3. Explorer / sidebar search — Functional, not large-schema friendly
- Resolved (2026-09-09, QA-P2-13): sidebar search builds a memoized table/view catalog, debounces input, virtualizes matching rows, and reports match counts.
- Resolved (2026-09-09, QA-P2-14): expanded schema table/view groups render through a bounded virtualized child list while preserving row context menus.
- Resolved (2026-09-09): selecting a connection now collapses unrelated connection/schema groups while preserving the selected tree.
- Plan: collect runtime evidence on very large schemas.

### 4. Workspace tabs — Powerful but unpredictable
- Fixed in W1: preview-carries-staged-changes, staged bypasses close guard (QA-P1-02/03) — verify by runtime.
- Resolved (2026-09-09, QA-P1-10/11): tab close commands/actions now share the guarded close service; orphan query reassignment resets connection context, while orphan DB-object/ER tabs close and select the target connection for a fresh resource pick.
- Resolved (2026-09-09, QA-P2-01): keyboard navigation and context-menu relative ordering now use the same pinned-first order rendered by the tab bar.
- UX-P2 remains: hardcoded Ctrl labels on macOS (QA-P2-02).
- Plan: single ordered tab list, all closes via `requestCloseTab`, reassign forces resource re-pick.

### 5. Query editor — Strong core, weak forgiveness
- Good: Monaco + dialects + risk classifier + timer + zoom.
- Resolved (2026-09-09, QA-P2-23): query history/import overwrite uses the shared app `AlertDialog` confirmation with localized action labels; no native `confirm()` remains in the query editor.
- Resolved (2026-09-09, QA-P2-22): export is gated by result state rather than SQL text (`9dcb5b7`). Query empty/error/success/cancelled/messages copy now uses localized `query.*` keys, and a known disconnected query connection exposes a reconnect affordance.
- UX-P2 remains: runtime/provider evidence.
- Plan: picker shows reconnect affordance; finish result/status i18n.

### 6. Results grid — Readable, needs scale honesty
- Resolved (2026-09-09, QA-P2-12): sorted large result sets now show an explicit localized warning at the existing 10,000-row threshold; the existing status bar also surfaces large-result state.
- UX-P2: metadata popover good but SQLite TEXT-everywhere fix needs runtime proof (QA-P1-04/14).
- Plan: collect provider/runtime evidence for result metadata and row limits.

### 7. Data grid editing — Dangerous affordances
- Resolved (2026-09-09, QA-P2-08): read-only connections show a clear banner and prevent cell editing before staging, rather than failing only at Apply.
- UX-P2: Columns picker double-toggle (QA-P2-07); custom context menu off-screen, no Escape/focus semantics (QA-P2-09); resize handles mouse-only, cleanup leaks (QA-P2-10/11).
- Plan: single-owner column toggle; Radix menu; keyboard resize + cleanup.

### 8. Schema inspector (columns/FK/indexes/triggers/DDL/CRUD) — Complete but intimidating
- Good: per-object tabs + DDL viewer + capabilities gating.
- Resolved (2026-09-09): column mutations show operation/risk/warning/SQL before confirmation, and the DDL editor now requires a confirmation showing the operation, target, and SQL preview before every execution.
- UX-P2: risk copy is still partly hardcoded in the column dialog; trigger enable/disable lacks runtime proof; CRUD generator output has no preview-before-apply story.
- Plan: finish risk-copy i18n and runtime/provider evidence.

### 9. ER diagram — Impressive, unfriendly at scale
- Resolved (2026-09-09, QA-P1-12/13; implementation in `59018df`): >200-table and L/XL schemas open in search-first mode with no renderer/layout; selecting a table materializes a bounded neighborhood and safe compact first paint; full overview requires explicit Show All.
- Resolved (2026-09-09, QA-P2-24/25): ER search renders schema-qualified candidates for disambiguation, Enter/click selects the highlighted candidate, and Fit calls the renderer API directly.
- Resolved (2026-09-09): when the React Flow MiniMap is omitted, the canvas explains why and points to overview controls.
- Plan: collect runtime evidence for large-schema search, fit, and navigation.

### 10. Export/Import/Backup — Works, low feedback
- Resolved (2026-09-09, partial): the mounted results export dialog now performs pre-flight validation for empty results, missing SQL table name, and missing Excel connection/query before download or backend mutation. Validation messages are localized and targeted coverage is present.
- UX-P2 remains: import parse errors need file/line context; backup/restore buttons are not currently mounted with a dialog, and still need provider-specific progress plus a reveal-location action.
- Plan: pre-flight validation, per-file error list, reveal-in-folder.

### 11. User management (PG-only) — Capability cliff
- Resolved (2026-09-09): the Users sidebar gates by the selected connection driver and explains why SQLite has no role/privilege management surface instead of falling through to an empty/error state. PostgreSQL now lists roles, shows the selected role's table-privilege count, and requires an impact-summary AlertDialog before `DROP ROLE`.
- Plan: gated empty state + confirm with affected objects — source and targeted UI coverage complete; provider/runtime verification remains pending.

### 12. Command palette / Quick Open / Agent — Discoverable vs misleading
- Good: palette + quick open + ranking exist.
- Resolved (2026-09-09, QA-P2-04/05): Agent preview affordances and platform-aware shortcut labels are implemented; runtime verification remains pending.
- Plan: keep runtime evidence for the preview-only flow.

**Resolution (2026-09-09, QA-P2-04):** Agent now shows a localized `Preview` badge; starter actions and the composer are disabled, with an explicit preview-only explanation. The existing platform-aware shortcut fix covers QA-P2-05. Full pnpm frontend gates pass; provider and packaged-desktop runtime verification remain pending.

### 13. Shell chrome / i18n / tokens — Inconsistent polish
- Resolved (2026-09-09): topbar traffic-light spacing is macOS-only; sidebar and Agent resize handles expose keyboard arrows, separator semantics, focusability, and bounded ARIA values.
- Resolved (2026-09-09, QA-P2-06 partial): ER overview controls and the full-schema action now use localized `schemaWorkspace.*` labels; remaining search/stat copy and runtime/token smoke are pending.
- Plan: i18n sweep and light/dark token smoke.

## 2026 trend lens (applied in IMPROVEMENT_PLAN.md)
Copilot-not-autopilot · purposeful trust-building motion · raw schematic clarity (mono+grid) · user-controlled motion/a11y · fluid `clamp()` type · off-white comfort + adaptive dark · bento modular blocks · micro-interaction feedback · anti-liquid-glass legibility · dense dark-first dev-tool idiom. Detail per wave: `IMPROVEMENT_PLAN.md`.

## Priority waves
- Wave A (correctness-friendliness): connections form contract, data-grid read-only honesty, tab/orphan guards, ER bounded open, Agent preview badge, native-confirm removal.
- Wave B (scale-friendliness): explorer/search virtualization, result sort budget, ER search disambiguation + Fit API.
- Wave C (polish): i18n, shortcuts, inset, menus, resize a11y, welcome labels, export/backup feedback.
