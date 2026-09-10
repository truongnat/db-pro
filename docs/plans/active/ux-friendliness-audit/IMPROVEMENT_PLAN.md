# UX Improvement Plan — ChatGPT App Style × 2026 Trends (detail)

Working tree: `main` (user-directed override of the repository branch rule). Companion: `FINDINGS.md` (evidence), `PLAN.md` (scope), `VERIFICATION.md`.
RC1 freeze respected: no Agent/MCP/new-product work; Waves A–C are fix-shaped.
Design north star: **ChatGPT app idiom** — calm, minimal, conversational — applied to a database IDE without losing dev-tool density.

## ChatGPT style contract (what we copy, what we don't)
Copy: quiet warm-neutral surfaces; narrow calm sidebar (new-action-first, recents, favorites, search); centered composer on empty states; greeting + suggestion cards; plain-language helper text; subtle borders over shadows; restrained status color; conversation-shaped Agent thread; `⌘K` command bar as the "ask anything" entry.
Don't copy: chat-only IA (we keep Explorer/Query/Data/Schema/ER tabs), oversized marketing empty states, low-density oversized controls — data grid, DDL, ER keep compact mono + dense rows.
Blends with 2026 backbone: copilot manners, purposeful trust motion, schematic clarity, user-controlled motion, fluid `clamp()` type, off-white comfort + adaptive dark, micro-feedback, anti-liquid-glass (blur only on overlays).
1. **AI copilot, not autopilot** — assistive, optional, asks before acting.
2. **Purposeful motion = trust** — animate state/structure only; believable > slick; instant success on destructive ops feels suspicious → staged confirm.
3. **Raw schematic clarity** — mono type for data, visible grid, foreground structure (ER, DDL, results).
4. **User in control** — `prefers-reduced-motion` honored, toggles, escape hatches, no trapped animation.
5. **Fluid type (`clamp()`)** — steady scaling, no breakpoint jumps; data density preserved.
6. **Off-white comfort + adaptive dark** — soften pure-white fatigue for long sessions; WCAG contrast in both modes; smooth non-flashy transition.
7. **Bento modular** — Welcome + inspector as scannable blocks, not long lists.
8. **Micro-interactions as feedback** — button press, save, copy, test-connection get instant subtle confirmation.
9. **Anti-liquid-glass** — no blur/gloss competing with dense data; depth only for overlays (dialog/popover/toast); legibility wins.
10. **Dense dark-first dev-tool idiom** — 4px grid, 13px base, compact controls, border-defined surfaces, status-only color.

## Wave A — correctness-friendliness (do first)
### A1 Connections form = visible model
- Remove SQLite password requirement/display; SSH toggle owns model (`undefined` on off, full default incl. port 22 on on); per-open session token kills stale create; test result invalidates on any field edit; backend `userMessage` shown on failure; Browse failure gets error toast.
- Trend: trust motion (test spinner → believable confirm), schematic clarity (provider-specific sections), control (explicit credential messaging on duplicate).
### A2 Data-grid honesty
- Read-only connection → banner + disabled cells before staging (no fail-at-Apply surprise).
- Trend: anti-gloss clarity + copilot-style "why disabled" hint.
### A3 Tab/orphan guards
- All closes via `requestCloseTab` (incl. orphan view); reassign forces target resource re-pick, clears stale schema/object.
- Trend: forgiveness + user control.
### A4 ER bounded open
- >200 tables opens to search-prompt/empty state + explicit "Show all N"; first paint at safe LOD tier 0/1; full graph is opt-in.
- Trend: raw schematic + purposeful motion (no surprise freeze).
### A5 Agent preview honesty + confirmation consistency
- Agent header gets Preview badge, composer disabled-with-reason; replace native `confirm()` calls with the shared app AlertDialog; export enabled by result state.
- Trend: copilot manners (asks before acting), no false affordance.
### A6 Schema risk language
- Mutation risk → plain consequence ("will rewrite table, undo = restore from…") + impact-summary confirm; DDL always preview-before-execute.
- Trend: believable motion/trust for infrequent critical actions.

## Wave B — scale-friendliness
- Explorer/search: debounced indexed search, virtualized rows, result counts, auto-collapse; result sort moves off main thread or shows capped-sort notice; ER search lists disambiguated candidates (Enter picks), Fit uses React Flow API, MiniMap-off explained.
- Trend: schematic clarity + fluid type + micro-feedback (skeleton, count-up) under reduced-motion respect.

## Wave C — polish (ChatGPT-style calm)
- Welcome becomes ChatGPT home: greeting + centered "Ask your database…" composer (routes to Query/Quick Open) + suggestion cards (New connection, Open recent, Run sample query) as quiet bordered cards, not bento rainbow; first-run guide inline; file-focused SQLite labels. platform shortcut labels + gated traffic-light inset; EN/JA i18n sweep; Radix menus (clamped, Esc/outside/focus); keyboard resize + listener cleanup; export/backup pre-flight + per-file errors + reveal-in-folder; PG-only users gated empty state + role-drop impact confirm; off-white/dark token smoke.
- Trend: bento, adaptive modes, inclusive visuals, crafted-not-prompted copy.

## Deep UI/UX audit — Wave D (new, evidence-backed)
Wave D is the remaining systemic quality work found after the Wave A–C source fixes. It is deliberately narrower than a redesign: repair interaction parity and trust failures before adding visual features.

### D1 Keyboard parity for custom interactions — UX-P1
- Replace pointer-only `div`/`span` actions with native buttons where the action is a command; otherwise add the complete keyboard path and an accessible name/state.
- In scope: query history/local history/snippets, EXPLAIN tree nodes, ER detailed column rows, grid sortable headers/row-number selection/cell editing, and snackbar dismissal.
- Acceptance: every pointer action has a keyboard equivalent; focus is visible; Enter/Space behavior is tested; no action depends on double-click or hover alone.

### D2 Hidden actions must remain discoverable — UX-P1
- Change hover-only action groups to reveal on `:focus-within` as well, and give icon-only controls an `aria-label`; preserve compact density.
- In scope: history actions, local-history/snippet delete, schema column/index copy/edit/actions, Quick Open recent removal, grid row actions, and tab close.
- Acceptance: tabbing never moves focus onto an invisible control; actions are reachable and understandable without a pointer.

### D3 Rendered contrast contract — UX-P1
- Add a small automated contrast matrix for light/dark rendered foreground/background pairs and manually verify alpha-composited states in the browser.
- Revisit tertiary text used for essential metadata, light-theme accent text, and white text on success/warning/danger/info solids. Keep tertiary for genuinely supplementary content or darken/replace the token.
- Acceptance: normal text ≥4.5:1, large text ≥3:1, non-text controls/focus indicators ≥3:1; state meaning is not conveyed by color alone.

### D4 Reduced motion and notification control — UX-P1
- Add a global `prefers-reduced-motion` policy covering spinners, pulse/skeleton, panel transitions, and Radix transitions; retain a non-motion loading/status affordance.
- Make snackbar dismissal a real keyboard-accessible button, pause timeout on hover and focus, and ensure errors remain recoverable long enough to read.
- Acceptance: reduced-motion browser mode has no non-essential animation; notifications can be dismissed and understood with keyboard/screen reader.

### D5 Semantic navigation model — UX-P2
- Give query results and schema/object section navigation real `tablist`/`tab`/`tabpanel` relationships with selected state and stable controls; retain horizontal overflow behavior.
- Audit landmarks and document language (`lang`) at runtime; add a skip-to-main path if the shell prevents direct keyboard entry to the main workspace.
- Acceptance: screen-reader tree exposes one coherent main workspace, selected tab, panel relationship, and current language.

### D6 Copy and localization closure — UX-P2
- Move remaining user-visible English into EN/JA keys, prioritizing loading/empty/error/destructive actions and tooltip labels. Keep SQL/provider enum values as technical data, not UI copy.
- Initial evidence: `Loading...`, `No data`, tab scroll labels, index actions/confirmation, grid row actions, EXPLAIN labels, and several tooltip strings.
- Acceptance: an EN/JA sweep finds no user-facing fallback English outside an explicit technical-data allowlist.

### D7 Stress matrix before closing RUNTIME_VERIFY — UX-P2
- Verify 320px-equivalent narrow window, 200% text scaling, long identifiers, Light/Dark/System, reduced motion, keyboard-only navigation, and RTL/locale expansion where supported.
- Re-run the 510-table ER case and capture browser accessibility/contrast evidence; keep PostgreSQL and SQLite provider checks independent.
- Acceptance: no clipped critical control, focus loss, unreadable state, or accidental full-graph mount; record unavailable provider capabilities as explicit limitations.

## Token/perf guardrails
- Keep `check:tokens` canonical (`--surface/text/border/accent/state-*`); shadcn aliases only; no new `--app-*` colors or raw semantic vars.
- `check:tokens` is necessary but not sufficient: add the D3 rendered-contrast matrix and review alpha/foreground combinations.
- Perf budgets enforced (`performance-budgets.test.ts`, sqlite benches, `perf-scan.sh`); LOD/minimap rules unchanged.
- Sidebar follows ChatGPT rail: top "New query" + "New connection" actions, sectioned recents/favorites/search, calm hover, collapse-safe; Command palette + Quick Open share one `⌘K` calm input with grouped results.
- Copy: hand-written, plain-language, no AI-slop adjectives.

## Exit
- Each wave ships as focused `fix/` PR(s) with PLAN/CHECKLIST/VERIFICATION refs, gates actually run, PG+SQLite accounted separately. Runtime evidence (packaged smoke, 500-table ER, keyboard, themes) closes RUNTIME_VERIFY — never source alone.
