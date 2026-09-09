# UX Friendliness Audit — Findings (source evidence only)

## Per-feature verdicts

### 1. Welcome / onboarding — Friendly with gaps
- Good: recent connections + shortcuts give fast resume (`commons/components/welcome-view.tsx`).
- UX-P2: SQLite recent subtitle shows meaningless `host:port` (`:0 / <path>`) — file-focused label needed (RC1 QA-P2-21).
- UX-P2: no first-run empty state guiding "New connection → Explorer → Query".

### 2. Connections — Least friendly, most fixes needed
- UX-P1: SQLite create requires password although backend ignores it (`connection-editor.tsx`, `sqlite/connector.rs`) — blocks primary flow (QA-P1-05).
- UX-P1: SSH toggle doesn't clear model; visible port 22 may not be submitted (QA-P1-06/07) — visible ≠ saved.
- UX-P1: late async create can poison next New Connection session (QA-P1-08).
- UX-P2: Test success goes stale after edits; failure hides backend detail; SQLite Browse has no error path; driverChanged is "ever changed"; duplicate omits credentials silently; favorite has no rollback (QA-P2-15–20).
- Plan: unify form model = visible model, per-open session token, test-state invalidation, explicit credential messaging.

### 3. Explorer / sidebar search — Functional, not large-schema friendly
- UX-P2: search scans + renders all matches per keystroke, no debounce/virtual list (`search-view.tsx`); expanded schema mounts all rows (`explorer-view.tsx`) (QA-P2-13/14).
- Plan: debounced indexed search, virtualized list, auto-collapse + result counts.

### 4. Workspace tabs — Powerful but unpredictable
- Fixed in W1: preview-carries-staged-changes, staged bypasses close guard (QA-P1-02/03) — verify by runtime.
- Resolved (2026-09-09, QA-P1-10/11): tab close commands/actions now share the guarded close service; orphan query reassignment resets connection context, while orphan DB-object/ER tabs close and select the target connection for a fresh resource pick.
- UX-P2: pinned visual order ≠ store/keyboard order; hardcoded Ctrl labels on macOS (QA-P2-01/02).
- Plan: single ordered tab list, all closes via `requestCloseTab`, reassign forces resource re-pick.

### 5. Query editor — Strong core, weak forgiveness
- Good: Monaco + dialects + risk classifier + timer + zoom.
- UX-P1: dirty history/import overwrite uses native `window.confirm` (`query-tab-content.tsx`) — inconsistent in Tauri (QA-P2-23, raised to UX-P1 for desktop).
- UX-P2: export enabled by SQL text not result state (QA-P2-22); connection picker unclear when disconnected; result/status strings bypass i18n (QA-P2-06).
- Plan: app AlertDialog guard, result-driven export, picker shows reconnect affordance.

### 6. Results grid — Readable, needs scale honesty
- UX-P2: sorting is sync main-thread over full array (`query-tab-content.tsx`) (QA-P2-12); metadata popover good but SQLite TEXT-everywhere fix needs runtime proof (QA-P1-04/14).
- Plan: worker/off-main sort or explicit large-result notice + row cap messaging.

### 7. Data grid editing — Dangerous affordances
- UX-P1: read-only connections still show editable affordances, fail only at Apply (`data-section.tsx`, QA-P2-08 → UX-P1: wasted work + surprise).
- UX-P2: Columns picker double-toggle (QA-P2-07); custom context menu off-screen, no Escape/focus semantics (QA-P2-09); resize handles mouse-only, cleanup leaks (QA-P2-10/11).
- Plan: read-only banner + disabled cells upfront; single-owner toggle; Radix menu; keyboard resize + cleanup.

### 8. Schema inspector (columns/FK/indexes/triggers/DDL/CRUD) — Complete but intimidating
- Good: per-object tabs + DDL viewer + capabilities gating.
- UX-P2: risk language (`column-mutation-risk`) needs plain-language "what will happen + undo"; trigger enable/disable lacks runtime proof; CRUD generator output has no preview-before-apply story.
- Plan: risk → consequence + confirm dialog with impact summary; DDL preview always before execute.

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
- UX-P1: Agent panel looks functional but is preview-only; header lacks Preview badge (QA-P2-04 → UX-P1: false affordance); `⌘↵` hint on Windows/Linux (QA-P2-05).
- Plan: Preview badge + disabled-with-reason composer, platform shortcut labels.

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
