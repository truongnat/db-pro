---
schema_version: 1
id: "MEM-PLAN-layout-aesthetics-audit-613790690eda"
title: "Plan archive: Layout Aesthetics Audit"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, layout-aesthetics-audit]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "613790690edaf1971b645f152addca33d3aea8c22fe443cfbad1c1cf2cd9a5ef"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/layout-aesthetics-audit/PLAN.md"
  - "docs/plans/active/layout-aesthetics-audit/FINDINGS.md"
  - "docs/plans/active/layout-aesthetics-audit/VERIFICATION.md"
  - "docs/plans/active/layout-aesthetics-audit/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/layout-aesthetics-audit/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Layout Aesthetics Audit

## State

- **State:** PLANNING
- **Branch:** `main` (direct edit per owner override)
- **Baseline SHA:** `493e3b9b` (after the viewport/select/chrome fixes)
- **Surface:** the shipped native `egui` application — every primary workspace, not a component gallery.

## Goal

Produce an evidence-backed assessment of how the native layout *looks and reads* — composition,
hierarchy, density, rhythm, type, colour and component consistency — scored against explicit
anchors, with prioritized findings that each name an owner plan.

This is an **evaluation**, not a redesign. No visual direction changes land here.

## Why now

The viewport overflow, complex-picker geometry and light-chrome separation work is finished
(`457cd645`), so the owner asked for a design/aesthetic read of the result. Those fixes were
correctness fixes; nobody has yet judged the composition as a whole.

## Evidence and failure scenario

- Existing plans cover *structure* (`native-visual-redesign`), *spacing tokens*
  (`ui-spacing-consistency`) and *component tokens* (`core-component-style-system`), but none owns a
  repeatable aesthetic evaluation. Without one, each review round re-litigates taste with no shared
  rubric, and regressions are only noticed when the owner spots them (as happened with the light
  chrome step).
- At `493e3b9b` there is no document stating, for example, how many distinct corner radii, type
  sizes or emphasis levels appear on one screen — the numbers that make "looks busy" arguable.

## Scope

**In scope (measurement and judgement only):**

| Dimension | What is measured |
|---|---|
| Composition & hierarchy | primary action per screen, competing emphasis, panel width ratios |
| Density & rhythm | vertical rhythm on repeated rows, gutter consistency, dead space |
| Alignment & grid | edges shared across panels, optical alignment of icons/labels |
| Typography | type scale in use, line lengths, weight usage, truncation behaviour |
| Colour & surface hierarchy | elevation steps, contrast ratios, accent usage per screen |
| Component consistency | same action rendered the same way across surfaces |
| Iconography | size/weight consistency, alignment to text baseline |
| Responsive composition | what survives at 1280×800 with panes open |

**Matrix coverage gaps (declared, not hidden).** The 60-cell run captures one representative per
group, so three planned sub-surfaces are **not** covered yet: Table *structure* (only the data grid is
captured), Settings → *Appearance* (only General), and the New Connection *error* state (only blank).
They stay open in `CHECKLIST.md`; no group is scored as fully covered from a single capture.

**Out of scope:** behaviour changes, provider/database work, new components or primitives, any
redesign proposal that is not backed by a finding in this audit, motion recording (static frames
only).

**Exception — routed defect fixes.** A defect the owner reports while the audit runs may be fixed in
its own scoped commit and recorded here as a routed finding (pre-fix capture kept for comparison),
because leaving a visibly broken surface in place would poison the scoring. The audit does not become
an implementation plan by doing so: it still owns judgement, the fix is listed under the finding that
justifies it.

## Method

1. **Surface inventory (fixed list).** Welcome, Query workspace (editor + output dock), Table
   workspace (data grid, structure), Schema explorer, ER canvas, Settings (General + Appearance),
   Connection dialog (blank + error), Component Gallery, Agent panel, Command palette.
2. **Capture matrix.** Each surface at 1280×800, 1440×900 and 1920×1080, light and dark, plus one
   real state where it exists (empty/loading/error). Use the existing `--features capture` harness;
   record requested vs rendered size and the `exit=-6` TouchBar teardown honestly. The 1440/1920
   requests render at 838 logical height on this host, so the report states that instead of claiming
   full-height verification.
3. **Measure, don't vibe.** For every scored screen record objective numbers alongside the image:
   panel widths, gutters, type sizes, contrast ratios, count of distinct radii/spacing steps, count
   of competing emphasis elements. A score without a number or a cited image is not evidence.
4. **Score with anchors.** Eight dimensions from the table above, 0–4 each (32 points), with written
   anchors (0 = broken, 2 = usable but inconsistent, 4 = deliberate and consistent). Two passes: self,
   then an independent reviewer on the same evidence; disagreement is reported, not averaged away.
5. **Prioritize and route.** Every finding carries severity (P0/P1/P2), the dimension it hits, its
   evidence pointer, and the owning plan. Nothing is filed as "polish later" without an owner.

## Acceptance criteria

- [ ] Every surface in the inventory is captured at all three widths in both themes, and each image is
      inspected; missing cells are listed as missing, never inferred.
- [ ] Each scored screen carries measured numbers next to its score.
- [ ] The rubric, its anchors and the per-surface scores are published.
- [ ] Every finding has severity, evidence pointer and an owning plan; no orphan findings.
- [ ] The three surfaces fixed in `457cd645` are re-judged and their before/after is stated.
- [ ] An independent reviewer repeats the scoring pass; agreements and disagreements are recorded.
- [ ] `docs/plans/STATUS.md` reflects this plan's real state.

## Risks and decisions

- **Taste presented as fact.** Mitigated by anchors, measurements and a second pass; scores are
  labelled as expert judgement, not user testing.
- **Scope explosion into redesign.** Findings route to existing owner plans; this plan does not
  implement visual direction changes.
- **Host height clamp.** 1440/1920 requests render at 838 logical height on this machine; the report
  states that instead of claiming full-height verification.
- **Overlap with `ui-spacing-consistency` and `core-component-style-system`.** Those plans own the
  fixes; this audit owns the judgement and the evidence. Findings that duplicate their open items are
  linked, not re-planned.

## PostgreSQL / SQLite impact

Not applicable: presentation evaluation only; no provider or database behaviour is touched.


## Source: `FINDINGS.md`

# Layout Aesthetics Audit — Findings

Baseline: `493e3b9b`. Scores live in `REPORT.md`; this file holds the issues themselves.

## Fixed during the audit

### F7b: Selected rows were invisible in the light theme (P1, found by three agents)
- **Surfaces:** activity rail, sidebar rows, gallery category nav, settings section rail.
- **Symptom:** `ButtonVariant::Secondary` rest fill is `surface_2` = `background.subtle` = `#f6f6f6`,
  which equals `surface_panel` — the fill of every rail — so the active row was pixel-identical to its
  background in light mode. Dark escaped by accident (Δ21).
- **Rejected fix:** reordering `Secondary`'s fill ladder. `palette.rs` documents that hover must not use
  the selected role, and `Secondary`/`surface_2` also serve ordinary secondary actions and chips.
- **Fix:** selection is marked by the shared accent pill (`nav::paint_selection_pill`), used by
  `sidebar_item`, the gallery category nav, the settings section rail and the activity rail icon button.
- **Evidence:** Welcome 6/6 cells re-inspected after the change; gallery/settings 1280 cells
  re-inspected. The remaining gallery/settings cells were re-captured but not re-opened, so those
  scores stay annotated as pre-fix.

### F7a: Explorer filter popover content had no inset (P2, owner-reported "siêu xấu")
- **Surface:** Explorer toolbar → object-filter popover, both themes.
- **Symptom:** Every row, the separator and the `Reset` link sat 4 pt from the popover's rounded edge
  (`menu_margin = SPACE_XS`), so the separator bled edge to edge, the section headers read as ordinary
  rows, and the block looked like an unstyled form rather than a menu.
- **Fix:** the popup body is wrapped in a margin-only frame (`SPACE_SM` inset, no fill or stroke — the
  popup frame still owns the surface), row spacing tightened to `SPACE_XXS`, and `SPACE_SM` added
  above the separator, above "Object kinds" and above `Reset`.
- **Evidence:** `evidence/explorer-{light,dark}-{1280x800,1440x900,1920x1080}.png` — all six cells
  captured from the fixed tree and inspected.

## Open observations (need scoring, not yet rated)

| Ref | Observation | Surface |
|---|---|---|
| A1 | In light mode the Welcome cards are white on a white canvas separated only by a hairline border, while the same cards in dark mode use an elevated fill — light relies on borders alone for elevation | Welcome / Explorer |
| A2 | The picker's dense state shows five rows in a 180 pt window with scrollbars hidden by the component contract, so a long list has no visible "more below" affordance | Query picker |
| A4 | The status bar is a single low-contrast run of six segments across the full width; it reads as a caption rather than a band | All workspaces |
| A5 | Selection glyphs are 16 pt (`CONTROL_SIZE = SPACE_LG`) against ~11–13 pt labels, so indicators dominate the row; this is component-wide, not popover-specific | All surfaces with `Checkbox`/`Radio` |
| A6 | At 1920 the Welcome cluster stays left-of-centre with a large empty right side; the composition does not recentre or scale | Welcome |

A5 belongs to `core-component-style-system` (component geometry), A1/A4/A6 to
`native-visual-redesign` (composition and elevation), A2 to this audit's own follow-up because the
scroll affordance is a picker decision.

## Cursor affordance audit (owner-reported: cursor does not match behaviour)

**Evidence level: source only.** This inventory comes from reading every `CursorIcon` site and every
`Sense::click`/`Sense::hover` allocation; no runtime hover was driven yet, so each gap below is a
source-derived expectation, not an observed mismatch. Runtime confirmation is a checklist item.

Inventory: 51 `CursorIcon` sites in `crates/ui` + `crates/app` — 33 `PointingHand`, 12 resize
variants, `Grab`/`Grabbing`/`Move` in the diagram canvas, `Wait` on a busy button, `NotAllowed` on a
disabled select option, one `Default`. **`CursorIcon::Text` is never used anywhere.**

The convention this codebase already follows, stated once so the gaps are obvious:

| Element | Cursor |
|---|---|
| Button, link, tab, menu item, row that navigates or selects | `PointingHand` |
| Editable or selectable text | `Text` |
| Splitter / column border / dock grip | `ResizeHorizontal` · `ResizeVertical` |
| Window edge / corner | `Resize*` in the matching direction |
| Canvas pan / drag / movable object | `Grab` · `Grabbing` · `Move` |
| Disabled control | `NotAllowed` (or `Default`) |
| Busy | `Wait` |

### Gaps (clickable, but cursor stays the default arrow)

| Ref | Element | File | Expected |
|---|---|---|---|
| C1 | Result-grid cell — double-click opens the inline editor | `result_grid/result_grid_cell.rs:48` | `Text` on editable cells |
| C2 | Result-grid row gutter — click selects the row | `result_grid/result_grid_row_gutter_view.rs:11` | `PointingHand` |
| C3 | Query status-bar labels — toggle transaction, edit bind parameters | `query_workbench/query_status_bar_surface_view.rs:133,148` | `PointingHand` |
| C4 | Explorer tree rows — click selects, disclosure toggles | `explorer/explorer_tree.rs:98` | `PointingHand` |
| C5 | Tools-hub cards | `tools/tools_hub_surface_view.rs` | `PointingHand` |
| C6 | Table data toolbar clickable cells | `table/table_data_toolbar_surface_view.rs` | `PointingHand` |

### Judged acceptable

- Diagram: `Grab` on empty canvas, `Grabbing` while panning/dragging, `Move` over a node — the node
  case could arguably be `Grab`, but it is deliberate and consistent.
- `NotAllowed` on a disabled select option and `Wait` on a busy button match the table above.
- `PointingHand` on tabs, menu rows, tree rows in the core `Tree` component and select options is
  correct; the custom explorer tree simply does not inherit it (C4).

Routed to: `core-component-style-system` for C1/C2 (grid affordances live in the shared table/grid
components), and this audit's own follow-up for C3–C6 (view-level omissions).

## Wide-layout distribution (audit item 5)

| Ref | Surface | Before | After |
|---|---|---|---|
| W1 | Welcome grid | fixed 620 pt at every width (38.8 % at 1920, 13.6 % canvas coverage) | column count from the local width: 4 columns at 1920 (100 % of the content column, card 396 pt), 2 at 1440/1280, 1 below 572; `CARD_MAX_WIDTH = 420` keeps the reading measure |
| W2 | Gallery specimen card | 541 pt inside a 1120 pt column (48 %); header actions ~110 pt outside the column | card fills the column (`Card::fill_width`); header and body share one `detail_column` helper so both land on the same edges |
| W3 | `nav::section_label` | `SECTION_LABEL_FONT_SIZE = 10.5` literal | `FONT_SIZE_BADGE` — the last off-token label size is gone |
| W4 | Query toolbar | run/explain/format, then the row cap, then a ~60 % empty run to the pinned overflow | resolved by grouping, not by distributing: the row cap moved into the trailing group with the overflow action, so a wide strip reads "actions left, view settings right". The earlier equal-gap attempt was reverted because it clipped the cap label to "50…" |

W4 is closed as a grouping decision: an IDE toolbar with actions on the left and view settings on the right is the convention, and it keeps every control at its natural width. Distributing equal gaps across four anchored groups was tried and rejected — it required pinning the cap trigger to a width smaller than its label.

## Live (Orca-driven) verification — 2026-10-23

The four open items were checked against the running app, not the capture harness.

| Item | Result |
|---|---|
| Table **data** surface | `evidence/live/table-data-light-1280.png` — employees, 3 rows, grid header/cells/gutter legible, row separators visible |
| Table **structure** surface | `evidence/live/table-structure-light-1280.png` — header row, PK/FK badges, Actions column with `Alter` links |
| Settings → **Appearance** | `evidence/live/settings-appearance-light-1280.png` — rail pill on the active section, card on the canvas, Light/Dark radios adjacent and unclipped, checkbox bounded |
| Connection **error** state | `evidence/live/connection-error-light-1280.png` — every invalid field carries a red border and its own message, plus a toast |
| Cursor | **test-verified + one live hover datum**: `stale_cursor_is_cleared_before_widgets_draw` asserts both directions of the frame-level reset (clear-first keeps a widget's `PointingHand`; clear-after erases it). Screenshot proof of the pointer glyph was not obtainable here — the shared desktop kept another window frontmost. Later pass: `screencapture -C` proven working
(Terminal I-beam renders), frontmost confirmed `db-pro-native`, yet hovering `Connected` (status bar)
and the `Welcome` tab still showed the OS **arrow** rather than `PointingHand`. Unresolved whether the
hit-miss is a hit-test gap or the OS cursor request not reaching winit; `clear_stale_cursor` itself is
locked by `stale_cursor_is_cleared_before_widgets_draw` |

New finding from the live pass:

### F8 (RESOLVED — not a defect): Connection dialog's last section was clipped at rest, scrollable (P3 nit)
- **Evidence:** `connection-dark-1440x900.png` — the `Cloud Presets & URI Importer` disclosure is cut mid-row at the body viewport bottom.
- **Resolution (live Orca drive, 2026-10-23):** `DialogFrame::body` already wraps the form in
  `egui::ScrollArea::vertical().max_height(560)` (DIALOG_FORM_BODY_HEIGHT) with a pinned footer
  outside the scroll region. The a11y tree exposes a `scroll bar` element, and
  `orca computer scroll --element-index` moved the body so the Cloud Presets row renders
  fully above the footer (screenshot `live/connection-scrolled-light-1280.png`).
  The clipped appearance at rest is the standard "more below" affordance of a scroll viewport.
- **Remaining P3 nit:** no scrollbar is painted at rest because `VisibleWhenNeeded`'s
  opacity animation only rises on interaction; the half-clipped row is the only affordance.
  Recorded as polish, not blocking.

## Chevron order in the trailing group (fixed)

Moving the row-cap picker into the right-to-left group mirrored its trigger to `⌄ 500 rows`. The picker now draws inside a forced left-to-right layout, so it reads `500 rows ⌄` again (verified in the re-captured `query-light-1280x800.png`).

## Independent scoring pass (second reviewer) — **partial**

This pass scored **Settings at 1280 only**, with visual estimates rather than pixel measurements, and could not attribute the Query/1920 images (three capture generations in the directory). It does **not** close the independent-review item and does not validate the other eight surfaces.

| Surface | First pass | Independent pass | Disagreement |
|---|---|---|---|
| Settings (General) | 18/32 light · 20/32 dark | **22/32 light · 22/32 dark** | +4 light / +2 dark. The second reviewer scored colour 3/4 (the new ladder makes the card read as raised without a border) and composition 3/4; the first pass had scored colour 1/4 on the flat-white theme that no longer exists |

Disagreements are recorded, not averaged. The independent pass also flagged two things the first pass missed or under-weighted:

- **Primary action changes appearance across themes** (dimension 6, 2/4): `Save` is accent blue in light and a light-grey fill in dark, because dark `accent.solid` is achromatic by design. That is a deliberate token choice, but it means "primary" is not one visual role across themes — worth a decision, not a silent pass.
- **Light rail edge is faint** (dimension 5): the nav panel and canvas differ by few luminance steps, so the rail boundary is weaker than the card boundary.

Query workspace could not be scored in that *earlier* pass (three capture generations in the directory). The full-matrix pass 2 afterwards scored Query 23/32 light and 24/32 dark — that gap is closed.

## Findings from the independent pass (post-matrix, SHA 65fad16d)

### F9 (P2): scrollable regions hide every scrollbar — no "more content" affordance
Two **distinct** scrollbar paths, one shared symptom — do not merge

… [truncated for memory entry; see source file]


## Source: `VERIFICATION.md`

# Layout Aesthetics Audit — Verification

**Status: converged.** Two scoring passes on one frozen matrix, five live Orca surfaces verified,
and the open items each carry an honest label (partial/blocked) rather than a false "done".

## Source reference

```text
code under test     65fad16d   matrix SHAs match this tree
evidence manifest   evidence/manifest.json — 60 cells, sha256 per cell, all inspected=true
live captures       evidence/live/ — Orca screenshots (not capture-harness)
audit HEAD          fede8872 + subsequent doc-only commits (77ddfa61)
```

## What exists

| Item | State |
|---|---|
| Rubric with 0–4 anchors, eight dimensions | `RUBRIC.md` |
| Matrix | 60 cells captured at `65fad16d`, `DB_PRO_CAPTURE_REASSERT_SURFACE=1`, sha256-verified |
| Cells inspected | **60 of 60** (5 scout scorers × 12 each + spot checks; `inspected=true` in manifest) |
| Live Orca surfaces | Table data, Table structure, Settings→Appearance, Connection error, dialog scroll, quick open — 6 captures in `evidence/live/` |
| Pass-1 scores | 9 surfaces scored pre-ladder (mean 17.4/32); Table had to score from source only (capture leaked Welcome) |
| Pass-2 (independent) scores | 5 reviewers, no access to REPORT.md; **all surfaces two numbers except Table** (single pass-2 number — gap admitted, not filled) |
| Cursor verification | `screencapture -C` pipeline proven (Terminal I-beam); `PointingHand` observed as arrow on live hover → recorded as **F15 defect**, not a test gap |
| Real-state captures (empty/loading/error) | error verified live; empty/loading not in the matrix |
| Heights >838 logical | not exercised (host clamp — needs an external display) |

## The matrix integrity story

- First run silently captured Welcome for all 6 table cells (missing `REASSERT`); fixed at
  `c245183e` and enforced in `capture_matrix.py`.
- One generation rule: any cell not in `manifest.json` at the current `code_sha` is stale, not
  evidence.
- `exit=-6` / `exit=134` in the runner log is the macOS TouchBar teardown crash **after** the PNG
  write — a crash, not a UI failure and not a clean pass.
- `inspected=true` means the PNG was opened by a reviewer, not just written.

## Open verification gaps

| Gap | Why | Plan |
|---|---|---|
| Table pass-1 score | first capture leaked Welcome; a retro score would fabricate | accept as an admitted single-number surface, or re-score the baseline SHA in a follow-up |
| Cursor hover | OS glyph never changes despite `on_hover_cursor` | F15 — needs a winit/egui fix before re-verifying |
| >838 logical height | host clamps the window | external display or a taller fixture |

## Verification commands executed

- `cargo test -p db-pro-ui -p db-pro-app` — 725 + 681 pass, 0 fail.
- `cargo +nightly fmt --all -- --check`, `cargo clippy --workspace --all-targets -D warnings`,
  `cargo build --release --locked -p db-pro-native` — all exit 0.
- `python3 evidence/capture_matrix.py` — 60/60 PNGs written, manifest sha256 verified.
- `orca computer scroll --element-index <scrollbar>` — F8: dialog body scrolls, clipped row
  revealed. `screencapture -C` calibration — Terminal I-beam proven, app arrow observed.

## 2026-10-23 — Dialog header/footer separators (64a96f57) — evidence caveat

**Change**: `DialogFrame` + `paint_dialog_card` paint `Separator` hairlines so
dialog cards read as header / body / footer (light & dark verified in
`evidence/dlg-sep-*`).

**Captured sizes** (18 PNGs): requested 1280×800 / 1440×900 / 1920×1080.
**Actual framebuffer**: 1280×800, 1440×838, 1920×838 logical points
(@2× → 2560×1600, 2880×1676, 3840×1676 px).

**Why**: macOS clamps `InnerSize` to the physical display's visible area — the
workstation screen is ~900 logical points tall minus menu bar/Dock, so heights
above ~838pt cannot be realized on a windowed capture. Widths land exactly.
This is an environment constraint, not a bug in the separators: the hairlines
are drawn relative to the card's own height and verified at every captured
dimension. Full-height evidence requires a display (or virtual display) with
≥1080pt visible height.

**States verified per size**: normal, error (`DB_PRO_CAPTURE_CONNECTION_ERROR` —
note it must be set *alone*; combined with `NEW_CONNECTION` the else-if chain
picks the normal dialog first), and pending (`TEST_PENDING`). Empty-state does
not apply to a form dialog.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Layout Aesthetics Audit

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | omp · audit lane — 9 per-surface `scout` agents (pass 1) + 5 `scout` scorers (pass 2) |
| Issue(s) | Owner request for an aesthetics/design evaluation + "audit kỹ hơn, mỗi feature một sub agent"; owner follow-ups on the Explorer popover and cursor behaviour |
| Task state | `Review` — all planned steps executed; two-pass scoring reconciled; 8 findings recorded (F9–F15), no P0 |
| Baseline SHA | `493e3b9b` · code under test `65fad16d` · docs trail `65fad16d..6f4b7255` |
| Branch / PR | `main` (direct edit per owner override) · no PR |
| Scope interpretation | Score every primary surface against the eight-dimension rubric using per-surface agents, fix owner-reported defects as routed findings, and report the recurring design problems with owners. |
| Out of scope | Redesign proposals without a finding; provider work; motion recording; scoring cells that do not exist. |

## 2. Progress checkpoint

- Evidence matrix at `65fad16d` (60 cells, `evidence/manifest.json` with per-cell sha256); audit docs trail it by a few doc-only commits.
- Completed:
  - [x] `RUBRIC.md` — 8 dimensions, 0–4 anchors.
  - [x] Pass 1: 9 agents scored the first matrix (pre-ladder theme).
  - [x] Fixes landed during pass 1: F7a popover inset (`1b94044e`), F7b nav selection pill (`bc99ad10`),
        light-theme elevation ladder (`c735211d`), input field boundary, chevron order (`faab6466`).
  - [x] Matrix re-captured at `65fad16d` with `DB_PRO_CAPTURE_REASSERT_SURFACE=1`; 60/60 PNGs,
        sha256-verified.
  - [x] Pass 2: 5 independent scorers × 12 cells each, no access to REPORT.md.
  - [x] Live Orca verification: Table data, Table structure, Settings→Appearance, Connection error,
        dialog scroll (F8), hover-cursor (F15).
- Independent pass — **partial**: all surfaces have two numbers **except Table** (pass-1 capture
  leaked Welcome, so Table carries only the pass-2 score of 21/22 light, 22/32 dark — the gap is
  admitted in REPORT.md, not filled retroactively).
- F8 — **resolved, not a defect**: `DialogFrame::body` wraps the form in `egui::ScrollArea` with
  `VisibleWhenNeeded`; live Orca scroll revealed the clipped section fully. Downgraded to P3
  affordance note.
- F15 — **recorded**: `on_hover_cursor(PointingHand)` observed as OS arrow on 3 of 6 sampled
  hover targets; `screencapture -C` proven via Terminal I-beam. Unit test green, runtime glyph
  wrong → open defect, root cause unproven (winit mapping vs egui propagation vs focus).
- F9 — **recorded**: shared `components::scroll_area::ScrollArea` hardcodes `AlwaysHidden` → grid
  right-edge clip invisible. **Separate cause** from the dialog's `VisibleWhenNeeded` (which fades
  but exists) — two distinct code paths, do not merge.
- Findings / risks: F9–F14 routed; F8 P3; F15 open defect; cursor list is a runtime-visual defect,
  not a test gap.
- Tests executed: `cargo test -p db-pro-ui -p db-pro-app` → 725 + 681 passed / 0 failed; fmt, clippy
  workspace, locked release build all 0.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | evidence matrix `65fad16d` · pass-1 fixes `1b94044e` `bc99ad10` `c735211d` `a87da3e6` `8ba71006` `faab6466` · docs trail = `git log` since matrix |
| Commit list | full history on `main` since `493e3b9b`; see `git log 493e3b9b..HEAD` |
| File / surface inventory | `docs/plans/active/layout-aesthetics-audit/{PLAN,RUBRIC,CHECKLIST,FINDINGS,VERIFICATION,REPORT,AGENT_EVIDENCE}.md` · `evidence/{manifest.json,capture_matrix.py,60 matrix PNGs,6 live PNGs}` · `crates/app/src/{explorer,settings,query_workbench,connection}` · `crates/ui/src/components/{common,nav,scroll_area,dialog}` |
| Acceptance mapping | rubric → `RUBRIC.md`; matrix → manifest 60/60 sha256-verified; two-pass scores → `REPORT.md`; findings → `FINDINGS.md` (F1–F15); independent pass → **partial (Table single number)** |
| Commands and counts | fmt 0 · clippy 0 · `db-pro-ui` 725 / `db-pro-app` 681 passed · release build 0 · 60/60 cells captured + 6 live |
| CI run IDs / status | not run (no PR) |
| Known limitations | 1440/1920 render at 838 logical height (host clamp); Table pass-1 number missing (gap admitted); cursor hover verified on 3 targets only; real-state cells (loading/error) not in matrix |
| Migrations / config implications | none; capture env vars are opt-in fixtures |
| Out-of-scope changes | `none` |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | pass 2 at `65fad16d`; live verification at `faab6466`+ |
| Verdict | `ACCEPT WITH P2` — no P0/P1 open |
| P0 / P1 / P2 counts | fixed: 0 / 1 / 2 (F7a P2, F7b P1, light-ladder P2 class) · open: F9 F10 F11 F12 F13 F15 (all P2) + F8 F14 (P3/P3-cluster) |
| Findings | `FINDINGS.md` (F1–F15) + `REPORT.md` (two-pass table + adjudication) |
| CI disposition | not executed |
| Next task(s) unblocked | F9 fix (scrollbar visibility wrapper) → own plan; F15 (winit cursor) → own plan; `native-visual-redesign` for F10–F12 |

## 5. Research / audit handoff

- **Source date:** 2026-10-23
- **References:** 9 pass-1 reports + 5 pass-2 reports (`agent://Audit*`, `agent://Score*`), `RUBRIC.md`,
  `evidence/manifest.json` (60 cells, sha256 each), 6 live Orca PNGs, `REPORT.md`.
- **Factual findings (measured):** light `canvas = elevated = editor = floating = #ffffff` with
  `panel #f6f6f6` before the ladder; `surface_2 = subtle = panel = #f6f6f6` made a selected row
  invisible in light; type scales 4–7 sizes per surface; wide layouts did not distribute (toolbar
  62% empty at 1920, welcome grid fixed 620 pt).
- **Pass-2 measurements (new, distinct):** Settings ~60% dead space; connection grid uses three
  column splits (~200px drift); 10 decorative accent headers on the diagram; active-tab light
  contrast below threshold; shared `ScrollArea` wrapper `AlwaysHidden` on every consumer.
- **Inference:** [INFERENCE] most light-theme weaknesses traced to the missing elevation step (fixed
  at `c735211d`); the remaining weakness cluster is affordance (F9) + consistency (F11–F13).
- **Decision / recommendation:** route F9+F15 to their own fixes (component wrapper + winit layer);
  F10–F13 to `native-visual-redesign`; F14 batch as polish.
- **Unresolved questions:** whether the light ladder should gain a 4th step; whether F15 is winit or
  egui (needs a minimal repro outside the app); whether the Table pass-1 number should be
  regenerated at the baseline SHA or accepted as missing.
- **Downstream tasks activated:** none yet — routing table only.

## 6. Tổng kết (Vietnamese summary)

Hai vòng chấm: 9 agent (pass 1, pre-fix, mean 17.4/32) → sửa light ladder + selection pill + input
boundary → 5 agent độc lập (pass 2, mean 23.4/32). Mọi delta ≥2 đã phán xử trong REPORT.

Live Orca xác nhận 4 sub-surface + dialog scroll (F8 không phải defect) + hover cursor (F15:
`on_hover_cursor` không tới OS — defect winit/egui, unit test xanh nhưng runtime sai).

Còn lại: Table chỉ có 1 con số (pass-1 leak Welcome, ghi gap rõ), F9–F15 chưa sửa (routing, không
trong scope audit). Không P0/P1 mở.

