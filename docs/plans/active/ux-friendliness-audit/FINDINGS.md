# UX Friendliness Audit — Findings (source evidence only)

## Per-feature verdicts

### 1. Welcome / onboarding — Friendly with gaps
- Good: recent connections + shortcuts give fast resume (`commons/components/welcome-view.tsx`).
- UX-P2: SQLite recent subtitle shows meaningless `host:port` (`:0 / <path>`) — file-focused label needed (RC1 QA-P2-21).
- UX-P2: no first-run empty state guiding "New connection → Explorer → Query".

### 2. Connections — Least friendly, most fixes needed
- Resolved (2026-09-09, QA-P1-05..08): SQLite no longer requires a password, SQLite strips stale SSH state, SSH defaults port 22 and preserves omitted passphrase as `undefined`, and connection-dialog callbacks are guarded by a per-open session generation.
- Resolved (2026-09-09, QA-P2-23): connection deletion from both the row action and context menu uses the shared app `AlertDialog`, with localized confirm/cancel labels.
- UX-P2: Test success goes stale after edits; failure hides backend detail; SQLite Browse has no error path; driverChanged is "ever changed"; duplicate omits credentials silently; favorite has no rollback (QA-P2-15–20).
- Plan: explicit credential messaging, complete remaining test/browse/duplicate/favorite polish.

### 3. Explorer / sidebar search — Functional, not large-schema friendly
- Resolved (2026-09-09, QA-P2-13): sidebar search builds a memoized table/view catalog, debounces input, virtualizes matching rows, and reports match counts.
- Resolved (2026-09-09, QA-P2-14): expanded schema table/view groups render through a bounded virtualized child list while preserving row context menus.
- Plan: auto-collapse unrelated groups and collect runtime evidence on very large schemas.

### 4. Workspace tabs — Powerful but unpredictable
- Fixed in W1: preview-carries-staged-changes, staged bypasses close guard (QA-P1-02/03) — verify by runtime.
- Resolved (2026-09-09, QA-P1-10/11): tab close commands/actions now share the guarded close service; orphan query reassignment resets connection context, while orphan DB-object/ER tabs close and select the target connection for a fresh resource pick.
- UX-P2: pinned visual order ≠ store/keyboard order; hardcoded Ctrl labels on macOS (QA-P2-01/02).
- Plan: single ordered tab list, all closes via `requestCloseTab`, reassign forces resource re-pick.

### 5. Query editor — Strong core, weak forgiveness
- Good: Monaco + dialects + risk classifier + timer + zoom.
- Resolved (2026-09-09, QA-P2-23): query history/import overwrite uses the shared app `AlertDialog` confirmation with localized action labels; no native `confirm()` remains in the query editor.
- Resolved (2026-09-09, QA-P2-22): export is gated by result state rather than SQL text (`9dcb5b7`). Connection picker clarity and result/status i18n remain open.
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
- UX-P2: search auto-picks first substring match (QA-P2-24); derived filter churn + synthetic Fit View keypress (QA-P2-25); MiniMap disabled >200 without explanation.
- Plan: bounded initial state (empty/search prompt + explicit Show All), safe-tier first paint, disambiguated search list, real Fit API.

### 10. Export/Import/Backup — Works, low feedback
- UX-P2: progress dialog exists but enablement/scope errors surface late; import parse errors need file/line context; backup/restore (pg_dump vs sqlite copy) needs provider-specific progress + "where did my file go" reveal action.
- Plan: pre-flight validation, per-file error list, reveal-in-folder.

### 11. User management (PG-only) — Capability cliff
- UX-P2: SQLite users surface must explain PG-only with reason (capability gate), not empty/error; destructive role drop needs privilege-impact summary.
- Plan: gated empty state + confirm with affected objects.

### 12. Command palette / Quick Open / Agent — Discoverable vs misleading
- Good: palette + quick open + ranking exist.
- Resolved (2026-09-09, QA-P2-04/05): Agent preview affordances and platform-aware shortcut labels are implemented; runtime verification remains pending.
- Plan: keep runtime evidence for the preview-only flow.

**Resolution (2026-09-09, QA-P2-04):** Agent now shows a localized `Preview` badge; starter actions and the composer are disabled, with an explicit preview-only explanation. The existing platform-aware shortcut fix covers QA-P2-05. Full pnpm frontend gates pass; provider and packaged-desktop runtime verification remain pending.

### 13. Shell chrome / i18n / tokens — Inconsistent polish
- UX-P2: macOS traffic-light inset on all OSes (`topbar.tsx`); hardcoded English breaks EN/JA contract (QA-P2-06); resize/dock a11y gaps.
- Plan: platform-gated inset, i18n sweep, ARIA separators + focus-visible.

## 2026 trend lens (applied in IMPROVEMENT_PLAN.md)
Copilot-not-autopilot · purposeful trust-building motion · raw schematic clarity (mono+grid) · user-controlled motion/a11y · fluid `clamp()` type · off-white comfort + adaptive dark · bento modular blocks · micro-interaction feedback · anti-liquid-glass legibility · dense dark-first dev-tool idiom. Detail per wave: `IMPROVEMENT_PLAN.md`.

## Priority waves
- Wave A (correctness-friendliness): connections form contract, data-grid read-only honesty, tab/orphan guards, ER bounded open, Agent preview badge, native-confirm removal.
- Wave B (scale-friendliness): explorer/search virtualization, result sort budget, ER search disambiguation + Fit API.
- Wave C (polish): i18n, shortcuts, inset, menus, resize a11y, welcome labels, export/backup feedback.
