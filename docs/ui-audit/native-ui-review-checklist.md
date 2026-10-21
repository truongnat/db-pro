# DB Pro native UX review & update guideline

**Status:** active · **Last updated:** 2026-10-21 · **Supersedes:** the previous four-gate checklist in this file

One guideline for three jobs:

1. **Check** a shipped surface against the product's UX bar.
2. **Decide** what to update (defect vs preference vs redesign) and how severe it is.
3. **Prove** the update — with runtime evidence, tests, and an audit report.

Use it for `crates/ui`, the UI state in `crates/app`, and the native shell in `crates/native-app`.
It complements [`RULES.md`](../../RULES.md) (component/token rules), [`AGENTS.md`](../../AGENTS.md)
(UI quality gates) and [`10-egui-native-migration-plan.md`](../10-egui-native-migration-plan.md)
(visual acceptance), and does not replace them.

---

## 0. Sources of truth

| Kind | Source |
|---|---|
| Product intent, users, aesthetic | [`.impeccable.md`](../../.impeccable.md) |
| Component / token / state rules | [`RULES.md`](../../RULES.md) |
| Build + visual evidence expectations | [`AGENTS.md`](../../AGENTS.md), [`10-egui-native-migration-plan.md`](../10-egui-native-migration-plan.md) |
| Headless audit contract | [`AGENT_CLI.md`](AGENT_CLI.md) |
| What can be automated | [`automated-native-ui-checks-research.md`](automated-native-ui-checks-research.md) |
| Interaction heuristics | [Nielsen, 10 usability heuristics](https://www.nngroup.com/articles/ten-usability-heuristics/) |
| Dialogue principles | [ISO 9241-110:2020](https://www.iso.org/obp/ui#iso:std:iso:9241:-110:ed-2:v1:en) |
| Accessibility targets | [WCAG 2.2](https://www.w3.org/TR/WCAG22/), [What's new in 2.2](https://www.w3.org/WAI/standards-guidelines/wcag/new-in-22/) |
| Desktop keyboard conventions | [Apple HIG — Keyboards](https://developer.apple.com/design/human-interface-guidelines/keyboards), [Microsoft — Keyboard accessibility](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/keyboard-accessibility) |

WCAG is written for web content. Here it is used as a **design target** for a native desktop app;
this guideline does not claim WCAG conformance. ISO 9241-110 is a framework for judging dialogue
quality, not a certifiable checklist. Product rules (`RULES.md`, `DbProTheme`) always win when a
generic guideline conflicts with them.

---

## 1. Product UX principles

Each principle is testable through the gates in §4. A surface that fails a principle is a UX
finding even if the pixels look fine.

| # | Principle | In DB Pro this means | Fails when |
|---|---|---|---|
| P1 | **Task fit** (ISO 1) | Controls map to the database workflow the user is doing: connect → inspect → query → verify → edit safely. | The UI exposes an internal/technical knob instead of the user's task, or forces a detour through an unrelated surface. |
| P2 | **Self-descriptive** (ISO 2) | State, scope and consequences are readable without trial and error: what connection, what table, what will change, how many rows. | A destructive action needs the user to infer its target; an empty or error state has no next action. |
| P3 | **Predictable** (ISO 3, N4) | Same action, wording and placement across surfaces; platform conventions for keys, dialogs and menus. | The same concept is called "Data", "Rows" and "Table data" in three places; a shortcut does something else on another surface. |
| P4 | **In control** (ISO 5, N3) | Cancel, undo, escape hatches, no surprise navigation, edits staged and reviewable. | A long query cannot be cancelled; leaving a tab discards work silently; a dialog traps the user. |
| P5 | **Error-robust** (ISO 6, N5/N9) | Prevent high-cost errors first; when one happens, explain what/why/next and preserve work. | A failed apply loses unrelated staged edits; the only error text is a raw driver string. |
| P6 | **Quiet and engaging** (ISO 7, N8) | Density is intentional, decoration is restrained, long sessions stay calm. | Every panel competes for attention; success confetti on a row edit; dead empty states. |

---

## 2. Evidence baseline (run before reviewing)

Do not report PASS from source reading alone. Pick the narrowest evidence that can prove the claim.

| Evidence | Command / source | Required for |
|---|---|---|
| Release build | `cargo build --release --locked -p db-pro-native` | Every UI change (AGENTS gate) |
| Capture build | `cargo build -p db-pro-native --features capture` | Screenshots / audit runs |
| Capture | `DB_PRO_CAPTURE_TO=/tmp/x.png DB_PRO_WINDOW_SIZE=1280x800 ./target/debug/db-pro-native` | Any visual/layout claim |
| Audit report | `DB_PRO_AUDIT_JSON=/tmp/out.json DB_PRO_AUDIT_SCENARIO=shell-1440 ./target/debug/db-pro-native` | Gate automation (§4) and CI policy |
| Scenario matrix | `bash tools/ui-audit-matrix.sh ./target/debug/db-pro-native /tmp/ui-audit` | Broad review; 25 named scenarios + `coverage-matrix.json` |
| Tests | `cargo test -p db-pro-ui -p db-pro-app` | Layout/clip/interaction invariants |
| Benchmarks | `cargo bench -p db-pro-app --bench result_grid_benchmarks` | Performance-sensitive surfaces |
| Component rule grep | command in [`RULES.md`](../../RULES.md) §6 | Any widget change |

**Evidence artefacts** — name them `<surface>-<state>-<theme>-<viewport>-<scale>.png` plus the
matching audit `.json`, and attach them to the PR/issue (or `docs/ui-audit/evidence/<date>/` when
they must persist). State explicitly which viewport × theme × locale × scale combinations were
**not** run. “Not run” is never reported as pass.

---

## 3. Classification and severity

Classify every finding before fixing it:

- **UI defect** — violates an objective invariant (clipped control, missing label, wrong state).
- **UX defect** — works as coded but fails a principle/gate (hidden recovery path, ambiguous target).
- **Visual preference** — taste, not a measurable failure. Fix only with the maintainer's agreement.
- **Redesign proposal** — changes the visual language or information architecture. Requires an
  explicit request; it cannot ride along with a bug fix.

| Severity | Use when | Examples |
|---|---|---|
| **Blocker** | Data loss/corruption, unsafe write, inaccessible core task, or a dead end with no recovery. | Applying a staged edit writes a truncated value; a read-only connection accepts a write; keyboard users cannot reach Apply. |
| **Major** | Core flow works but requires workaround, misleads, or costs significant time. | Filter reuses the previous table's filter; overwrite prompt names a different file than it writes; missing key renders `table.no_dependencies`. |
| **Minor** | Polish, density, copy, alignment; no task impact. | Inconsistent spacing between two empty states. |

Record **confidence** (`high` = reproduced with evidence, `medium` = source-traced, `low` = hypothesis).
A `low`-confidence item is never a blocker.

---

## 4. Review gates

Mark every item `PASS`, `FAIL`, or `N/A (reason)`. A gate is passed only when all applicable items
have evidence.

### Gate A — Task correctness and state

- [ ] One state per concept, visible and current: connection (connected / connecting / failed / read-only), execution (idle / running / cancelled / failed / success), staged changes, scheduled task.
- [ ] Context switches (connection, schema, table, tab) reset stale transient state; no success/error text from the previous context remains.
- [ ] Success feedback states **what happened and its scope** (rows affected, target table, capped vs all rows).
- [ ] Loading, empty and error are visually and textually distinct; none of them is a dead end (each offers the next action or a way back).
- [ ] Active filter/sort/search is visible and resettable; the user never has to guess why rows are missing.
- [ ] The primary action for the current task is reachable without recalling a previous screen.

### Gate B — Layout, density, resizing

- [ ] Content and hit areas stay inside their container at 800×600, 1280×800, 1440×900, 1920×1080, plus each surface's constrained-width case.
- [ ] Required identifiers, headers, primary actions and focus rings are never clipped; truncation is intentional and the full value is available (tooltip, inspector, detail view).
- [ ] Multi-panel surfaces document which panel shrinks/collapses first and preserve primary actions under pressure.
- [ ] Scale factors 1.0 / 1.25 / 1.5 / 2.0 and text sizes are checked where the host can set them; unavailable combinations are recorded as not run.
- [ ] Density is deliberate (row height, alignment, monospace where it aids scanning); no card-on-card stacking or decorative borders per `.impeccable.md`.
- [ ] Scrolling is used only where the container is meant to scroll; clipped-but-scrollable content is not presented as complete.
- [ ] Long identifiers, Unicode (Vietnamese), CJK glyphs and emoji fallback render without breaking alignment or meaning.

### Gate C — Visual system, contrast, states

- [ ] Every color, spacing, radius, font size and weight comes from `DbProTheme` / tokens (RULES §3); no literal `Color32` or magic dimensions in views.
- [ ] Text contrast ≥ **4.5:1** (normal) and ≥ **3:1** (large: ≥18 pt / 14 pt bold); non-text UI boundaries, icons and focus indicators ≥ **3:1** against adjacent colors, in both themes.
- [ ] Interactive controls expose default / hover / active / focus / disabled states, each visually distinct and token-driven.
- [ ] The focus indicator is visible and not fully obscured by sticky headers, footers, overlays or the status bar (WCAG 2.4.11).
- [ ] Color is never the only signal: status, dirty state, selection, masking and diff use text/icon/shape as well.
- [ ] Typography resolves as intended (family, weight, fallback); informative secondary/muted text still meets contrast, and disabled text remains legible.
- [ ] Icon-only actions have a tooltip and an accessible name; decorative icons are not announced as controls.

### Gate D — Keyboard and pointer parity

Keyboard is a primary interaction model for an IDE, not a fallback.

- [ ] Every core flow is completable keyboard-only: connect, open table, filter, execute, cancel, edit/apply a cell, export, switch tab, open/close dialogs, palette actions.
- [ ] Tab order follows visual order; composite widgets keep one tab stop and use arrows internally (grid, tree, tabs, segmented controls).
- [ ] Region switching works and is reversible (e.g. `F6` / `Shift+F6` where the shell implements it); focus is restored when a dialog closes.
- [ ] `Enter`/`Space` activate; `Escape` cancels or closes; grid supports arrows, Home/End, PageUp/PageDown, and Shift-selection; tree supports arrows and expand/collapse.
- [ ] Focus is always visible and at least partially on screen after the focus moves (WCAG 2.4.7 / 2.4.11).
- [ ] Pointer-only actions have keyboard equivalents. Drag interactions (column resize/reorder, ER pan or node move, split panes) have a non-drag alternative or a documented essential exception (WCAG 2.5.7).
- [ ] Target size ≥ **24×24 px** or ≥24 px spacing / equivalent control / inline exception (WCAG 2.5.8). Dense grid cells count as targets.
- [ ] Context menus opened by right-click have a keyboard path (`Shift+F10` / Menu key) for actions that are otherwise pointer-only.
- [ ] Shortcuts are discoverable (menu, palette, tooltip), conflict-free across modifier aliases, and do not fire while typing in the SQL editor unless that is the intent.

### Gate E — Semantics and assistive technology

AccessKit is enabled in the native shell, so semantics are checkable, not assumed.

- [ ] Every interactive element exposes role, accessible name and state; custom-painted controls (grid cells, ER nodes/edges, tabs, charts) expose meaningful semantics rather than an unlabeled canvas.
- [ ] Accessible name matches the visible label (label-in-name); no "button"/"icon"/"close" without context.
- [ ] Dialog title and purpose are programmatic, not only painted text.
- [ ] Status changes that matter (query finished, apply failed, connection lost) are announced or reachable, not only color/flash (WCAG 4.1.3 as a target).
- [ ] At least one manual screen-reader pass (NVDA, VoiceOver or Orca) is recorded for a changed core flow; if not possible, record `NOT TESTED` with the reason.
- [ ] Keyboard-only use does not require the pointer or a visual-only cue.

### Gate F — Feedback, errors, recovery

- [ ] Errors state **what happened, why, and what to do next**, with the target context (connection/table/statement). A raw driver message alone is not an error message.
- [ ] Details are copyable; the user can retrieve an error after a toast disappears.
- [ ] Operations >1 s show progress and allow cancel; cancel produces truthful feedback and leaves a usable state.
- [ ] Transient failures offer retry, and retry does not discard unrelated work.
- [ ] Empty states explain why they are empty and offer the next action; loading states are labelled and do not steal focus.
- [ ] Success is truthful about partial results (partial apply, capped export, skipped rows).

### Gate G — Safety and data-integrity UX

- [ ] Destructive SQL shows a confirmation naming the operation, target and blast radius; read-only behaviour is enforced by the backend, not only the UI.
- [ ] Staged edits show count/affected rows before apply; a partial failure preserves unsubmitted edits; conflicts show both versions and a clear resolution path.
- [ ] Navigating away with unsaved work warns; the warning is accurate about what will be lost.
- [ ] File overwrite prompts name the **effective** destination and format; no silent overwrite after changing the format/path (see issue #539).
- [ ] Truncated or capped values are marked before editing; the UI never writes back a truncated value (see issue #537).
- [ ] Tables/cells that cannot be edited explain why (no PK, read-only, write-blocked column) at the point of attempt.
- [ ] Secrets are masked by default; no credential appears in logs, toasts or exported files.

### Gate H — Localization and formatting

The product ships `en`, `vi`, `zh`, `ja`, `ko` ([catalogs](../../crates/ui/locales)).

- [ ] Every user-visible string goes through the locale catalog; no hardcoded copy in empty states, errors, tooltips, menus, dialogs or status text.
- [ ] Key parity across all five catalogs; a missing key must fail a test, never render as `table.no_dependencies`.
- [ ] Layout survives +30 % string growth (Vietnamese/Chinese/Japanese/Korean) without clipping actions or labels.
- [ ] Dates, times, numbers, bytes, durations and counts are formatted for the active locale; database values themselves stay exact and unlocalized.
- [ ] CJK/Vietnamese glyphs render with the shipped fonts; emoji is never required to convey meaning.
- [ ] RTL is recorded `N/A` unless the product adds an RTL locale; do not claim support from untested CSS-like rules.

### Gate I — Performance and responsiveness

Budgets are product budgets for the supported row limit (100 000 rows) and 64 KiB cell cap.

- [ ] No work proportional to the whole result set runs per frame (no full-result clone, no O(rows) preview per repaint). Measured issues: #541 (clone), #542 (cell ellipsize).
- [ ] Filter/sort/search feedback stays under ~100 ms for the supported row limit; non-ASCII data does not take a per-cell allocation path (#543).
- [ ] Pan/zoom/search on a large schema does not re-normalize the whole catalog each frame (#544).
- [ ] Idle repaint only happens for real deadlines or in-flight work, not continuous 50 ms polling (#545).
- [ ] Operations >1 s show progress and cancel; >10 s keep the UI responsive and report in the status bar.
- [ ] Memory is bounded: virtualization for rows, capped cell text, streamed/capped exports; a large dataset does not freeze the dialog.

### Gate J — Consistency and maintainability

- [ ] Views use the core components from `crates/ui/src/components`; no raw egui widgets outside the documented exceptions (`RULES.md` §1, §5).
- [ ] No duplicated tokens or view-local color/spacing constants; a new value becomes a token first.
- [ ] The same concept has the same word and affordance everywhere (e.g. "Apply" vs "Save", "Table data" vs "Data", "Filter" vs "Search").
- [ ] Recurring patterns (empty state, destructive confirm, pagination, export, connection picker) are shared components, not re-invented per surface.
- [ ] The change updates the relevant tests, audit scenario, locale keys and docs; no dead feature flag or stale shortcut entry remains.

---

## 5. Surface matrix

Use the scenario names from `tools/ui-audit-matrix.sh` as the baseline evidence per surface.

| Surface | Core flows to exercise | Surface-specific invariants | Baseline scenarios |
|---|---|---|---|
| **Shell / topbar / status bar** | switch connection, switch workspace tab, window drag, menu/palette, read-only indicator, task status | Connection identity (name + engine + read-only) always visible; no status-bar label collision at 800/1280; drag hit target is not a stray tab/focus stop; focus never hidden behind the topbar/status bar | `shell-800`, `shell-1440`, `shell-1920` |
| **Query workbench** | type/format SQL, run current/selection/all, cancel, inspect multi-statement output, export, history, saved drafts | Current-statement indicator matches what runs; cancel leaves a usable editor; output pane selection is explicit; export prompt uses the effective path/format; dirty state survives tab switches correctly | `query-dark`, `query-light`, `results-dock`, `results-dock-light` |
| **Result grid / table data** | filter, sort, page, select, copy, edit cell, apply, conflict resolution | Displayed order equals copied order; NULL vs empty is distinct; staged count matches what is applied; no-PK/read-only reasons are visible; large values stay fast and editable only when complete | `table-data`, `table-structure-light`, `table-indexes` |
| **Explorer / schema** | filter tree, keyboard traverse, pin/recent, context menu, open table/view/routine, schema refresh | Filter and counts agree; stale pin/recent clears on connection switch; keyboard reaches every node; context-menu actions have keyboard paths; long names truncate with full value available | `explorer-filter`, `table-ddl`, `table-profile` |
| **ER diagram** | pan/zoom, search/focus, neighbourhood, layout, select/drag node | Search feedback appears within the same frame budget; no-match state explains the result; pan/zoom has a non-drag path; labels stay legible at fit zoom; large schemas respect the table limit | `diagram` |
| **Dialogs / forms** | connect/edit connection, export, delete/confirm, insert row, settings | Labels and required/optional states visible; errors inline and actionable; Enter/Escape behave predictably; focus trapped and restored; destructive copy names the target | `new-connection-dialog`, `quick-open-light`, `screen-fixture-good`, `screen-fixture-ugly` |
| **Agent panel** | ask, draft SQL, review, confirm a mutating tool, cancel, provider error, key entry | No DB mutation without explicit confirmation; streaming status and cancel visible; provider/key errors explain recovery; secrets never rendered | `agent` |
| **Settings / keybindings** | search, change theme/language, remap shortcut, reset | Settings apply predictably and persist; shortcut conflicts surfaced across modifier aliases; disabled options explain why | `settings` |
| **Welcome / first run** | connect, browse a sample, read empty state | Empty state explains the product and next step; connection errors offer retry and help; no dead end | `welcome-light`, `quick-open-light` |

---

## 6. Finding format

```text
ID:          UX-<surface>-<nn>
Surface:     <screen/panel> · Gate: <A–J> · Principle: <P1–P6>
Severity:    blocker | major | minor      Confidence: high | medium | low
Environment: <commit> · <OS> · <theme> · <viewport> · <locale> · <scale>
Evidence:    <png path / audit json key / test name>
Observed:    <what the user sees, with the measured or reproduced detail>
Expected:    <which gate/check and what should happen instead>
Repro:       <numbered steps + fixture>
Fix:         <smallest change that satisfies the gate>
Guard:       <test / bench / audit scenario that prevents regression>
Issue:       #<number>
```

Rules:

- One finding per issue; link the gate in the issue body.
- A screenshot diff is an alert, not a verdict. Never approve a baseline change without reviewing the image.
- Do not mix a redesign proposal into a defect fix.
- Prefer a guard (test/bench/scenario) over a one-off manual verification.

---

## 7. Definition of done for a UX update

- [ ] The finding's gate items pass, re-checked after the change (not before).
- [ ] Runtime evidence exists for the changed surface: before/after at 1280×800, 1440×900, 1920×1080 (or documented N/A), both themes, light/dark, and the affected loading/error/empty states.
- [ ] Keyboard-only path exercised; focus visible and restored; no new drag-only action.
- [ ] `cargo test -p db-pro-ui -p db-pro-app` passes; a regression test, bench or audit scenario guards the fix.
- [ ] Locale keys exist in all five catalogs; no hardcoded copy added.
- [ ] If the change touches rendering or data volume, the relevant bench/probe shows the surface stays within the Gate I budget.
- [ ] Issue closed with evidence links; `docs/ui-audit/` or user guide updated when behaviour changed.

---

## 8. Standards mapping

| Gate | Nielsen | ISO 9241-110:2020 | WCAG 2.2 (target) |
|---|---|---|---|
| A Task correctness/state | 1, 2 | Suitability, self-descriptiveness | — |
| B Layout/density | 8 | Suitability | 1.4.10 Reflow, 1.4.12 Text spacing, 1.4.4 Resize |
| C Visual/contrast/states | 4, 8 | Conformity with expectations | 1.4.1 Use of color, 1.4.3 Contrast, 1.4.11 Non-text contrast, 2.4.7 Focus visible |
| D Keyboard/pointer parity | 3, 7 | Controllability | 2.1.1 Keyboard, 2.4.3 Focus order, 2.4.11 Focus not obscured, 2.5.7 Dragging, 2.5.8 Target size |
| E Semantics/AT | 4, 6 | Self-descriptiveness | 4.1.2 Name/role/value, 4.1.3 Status messages, 3.3.8 Accessible authentication |
| F Feedback/errors | 1, 3, 9 | Use error robustness | 3.3.1 Error identification, 3.3.3 Error suggestion, 3.2.6 Consistent help |
| G Safety/data integrity | 5 | Use error robustness | 3.3.7 Redundant entry (avoid re-asking), 3.3.8 Accessible authentication |
| H Localization | 2, 4 | Conformity with expectations | 3.1.x Language of content (as a target) |
| I Performance/responsiveness | 1, 7 | Controllability | Timing/latency targets are product budgets, not WCAG SC |
| J Consistency/maintainability | 4 | Learnability | — |

---

## 9. Known gaps and follow-ups (2026-10-21)

These are accepted gaps in the current process, not excuses to skip a gate:

1. **No CI guard for i18n key parity / hardcoded strings.** A missing key shipped as
   `table.no_dependencies` (issue #540). Add a test that every literal `t!("…")` key exists in
   `en.yml` and all locales, plus a hardcoded-copy scan for user-visible strings.
2. **No automated keyboard-only traversal suite.** Keyboard paths are checked manually and with
   synthetic frames; add a repeatable scenario for shell → editor → grid → dialog focus order.
3. **Locale and scale-factor coverage is not in the 25-scenario matrix.** Add `vi`/`ja` × 1.25 /
   1.5 scenarios before claiming localization readiness.
4. **ER drag-only interactions need an alternative** (WCAG 2.5.7) or a documented essential
   exception.
5. **Performance gates are not enforced in CI.** The measured hotspots are #541 (result clone per
   frame), #542 (cell ellipsize), #543 (filter allocation), #544 (ER search), #545 (idle repaint).
   Add the probes/benches to the perf job before setting a threshold.
6. **Windows/Linux runtime visuals are unverified** (README: build-verified only). Do not mark
   platform-specific visual gates as passed from a macOS capture.

---

## Sources

- Nielsen, J. *10 Usability Heuristics for User Interface Design.* NN/g. https://www.nngroup.com/articles/ten-usability-heuristics/
- ISO. *ISO 9241-110:2020 — Ergonomics of human-system interaction — Part 110: Dialogue principles.* https://www.iso.org/obp/ui#iso:std:iso:9241:-110:ed-2:v1:en
- W3C. *Web Content Accessibility Guidelines (WCAG) 2.2.* https://www.w3.org/TR/WCAG22/
- W3C WAI. *What's New in WCAG 2.2.* https://www.w3.org/WAI/standards-guidelines/wcag/new-in-22/
- W3C WAI. *Understanding SC 1.4.3: Contrast (Minimum).* https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html
- W3C WAI. *Understanding SC 2.5.8: Target Size (Minimum).* https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html
- W3C WAI. *Understanding SC 2.4.11: Focus Not Obscured (Minimum).* https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html
- Apple. *Human Interface Guidelines — Keyboards / Accessibility.* https://developer.apple.com/design/human-interface-guidelines/keyboards · https://developer.apple.com/design/human-interface-guidelines/accessibility
- Microsoft. *Keyboard accessibility — Windows apps.* https://learn.microsoft.com/en-us/windows/apps/design/accessibility/keyboard-accessibility
- Project: [`RULES.md`](../../RULES.md), [`AGENTS.md`](../../AGENTS.md), [`.impeccable.md`](../../.impeccable.md), [`AGENT_CLI.md`](AGENT_CLI.md), [`automated-native-ui-checks-research.md`](automated-native-ui-checks-research.md)
