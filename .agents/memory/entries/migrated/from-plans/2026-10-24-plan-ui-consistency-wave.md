---
schema_version: 1
id: "MEM-PLAN-ui-consistency-wave-4f19b86dd4ae"
title: "Plan archive: Ui Consistency Wave"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, ui-consistency-wave]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "4f19b86dd4ae48358ede438c5c56de6febabcb44b8143fc79a09f044cf8c3fa0"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/ui-consistency-wave/PLAN.md"
  - "docs/plans/active/ui-consistency-wave/FINDINGS.md"
  - "docs/plans/active/ui-consistency-wave/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/ui-consistency-wave/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# UI consistency wave

State: REVIEW. Baseline SHA: `89095cc7d4b056a9fcc19dd9368f7c2863c8759f`. Fix commit:
`0e370498`; gates and runtime captures recorded in `VERIFICATION.md`.

## Scope

Fix the systemic UI consistency findings from the two audit rounds (see `REPORT.md`):
the shared Select contract, affected call sites, the three P1 layout defects, and the
concrete report-1 layout/state findings that share the same components.

## Acceptance

- P1: explorer Refresh no longer overlaps Filter; agent composer renders top-down;
  schema picker scrolls and is keyboard-reachable.
- Select: no layout re-anchoring, no light-theme default, disabled visual, draft
  arrow selection with Escape revert, option semantics, single-row label contract,
  menu rows truncate instead of overflow.
- Call sites: one height per row context, widths from tokens, row caps localized,
  sidebar filters right-aligned, engine select themed in dark.
- Runtime evidence: light + dark at 1280×800, 1440×900, 1920×1080 for affected
  surfaces, including an open Select menu.
- Gates: fmt, check, clippy `-D warnings`, workspace tests, locked release build,
  `scripts/check-ui-architecture.sh`, clean-code diff ratchet.

## Non-goals

- Full ~417-string i18n sweep (tracked as UI-22; only strings touched by this wave are
  localized).
- Full dialog-width preset refactor (UI-20) — documented, not bundled with Select work.
- Provider/runtime behavior changes; no DB mutations.

## Plan docs

- `CHECKLIST.md` — execution checklist
- `FINDINGS.md` — severity and root-cause summary
- `VERIFICATION.md` — commands, counts, captures
- `REPORT.md` — consolidated audit report

## Wave 4 (2026-10-22) — Select width contract + workspace spacing rhythm

State: RUNTIME_VERIFY. Baseline SHA: `f984385a`.

Owner request: Selects whose content is tiny render absurdly wide and then fail to show
that content fully; re-measure how many places actually render UI fused together.

Scope:

- `Select` trigger width becomes content-hugging unless `.width()` pins it; the popup
  sizes from the measured widest option and pins itself with `ui.set_width`.
- `shell_frame_view.rs` stops zeroing `item_spacing` and carries the theme rhythm
  (`SPACE_SM × SPACE_XS`) so a view cannot silently render flush.
- Six icon-glyph + caption-label rows move from 0–2 px to `ICON_TEXT_GAP`.
- The audit skill gains a `ZERO-SPACING` static rule, a measured `fusion_scan.py`
  runtime scanner, two heuristics, and a variant-safe capture filename.

Out of scope: the pre-existing Schema Compare clipped diff list (UI-30) and the shared
`ScrollArea`'s always-hidden scroll bar (UI-31) are filed, not fixed.

Acceptance:

- Row-limit popup measures ≈ option width, not screen width, with no ellipsized option.
- Full re-verify matrix: 17 surfaces × 3 sizes (1280×800, 1440×900, 1920×1080) ×
  dark + light, checked for over-spaced regressions after the shell flip.
- Fusion counts come from painted geometry with a pixel crop behind every claim.
- Captures never open a window on the owner's desktop (Xvfb, and capture mode now
  creates an unmapped X11 window).


## Source: `FINDINGS.md`

# Findings — UI consistency wave

Consolidated detail: `REPORT.md`. Persistent audit memory:
`.skills/ui-consistency-audit/references/findings-ledger.md`.

## Counts at baseline `89095cc7`

- P0: 0
- P1: 3 (UI-01 explorer toolbar overlap, UI-02 composer order, UI-03 schema picker
  reachability)
- P2: 22

## Root causes

1. `Select::show` re-anchors itself with `with_layout(top_down(Align::Min))`
   (`select.rs:170`) and defaults to the light theme (`select.rs:83`).
2. Component state coverage is incomplete: disabled, error/required, option
   semantics, Escape-revert.
3. Container layouts are hand-rolled and nest RTL/LTR incorrectly (explorer toolbar,
   history filter) or reverse authored order (composer `bottom_up`).
4. Strings, sizes, and widths are duplicated as literals instead of tokens/locales.

## Resolution (commit `0e370498` on top of `89095cc7`)

- P1: 3/3 fixed (explorer toolbar, composer order, schema picker scroll).
- P2: 20/22 fixed; UI-20 (dialog width presets + destructive footer) and UI-22
  (~417-string i18n sweep) deferred with rationale in `VERIFICATION.md`.
- Evidence: 42 runtime captures in `/tmp/ui-audit/wave2/`; gates recorded in
  `VERIFICATION.md`. No new P0/P1 introduced.

## Baseline vs regression

No regression is introduced by this wave; every finding is inherited from the baseline.
Verification must re-measure the same geometry and report inherited issues that remain.

## Wave 4 counts (baseline `f984385a`)

- P0: 0
- P1: 2 fixed (UI-26 Select popup width, UI-27 shell zeroed `item_spacing`) + 1 open
  pre-existing (UI-30 Schema Compare diff list unreachable)
- P2: 2 fixed (UI-28 six fused icon+label rows, UI-29 gallery separator stub) + 1 open
  (UI-31 shared `ScrollArea` hides its scroll bar)

### How many places actually render fused

| Measurement | Before | After |
|---|---|---|
| Static `ZERO-SPACING` candidates (≥2 children, no spacing call) | 142 | 148 — advisory only: the shell no longer zeroes spacing, so a hit now means "no explicit gap here", not "fused" |
| Runtime `fusion_scan.py` over 17 surfaces @ 1440×900 dark | 20 raw pairs → 17 shared SidePanel edges + **3 confirmed fusions** | n/a |
| Runtime `fusion_scan.py` over the full matrix (17 surfaces × 3 sizes × dark+light = 102 dumps) | n/a | 142 raw pairs → 130 panel edges + 12 × one tree-row pair disproved by pixel measurement → **0 confirmed fusions** |
| Icon-glyph + caption-label rows at 0–2 px (grep + crop) | **6** → all moved to `ICON_TEXT_GAP` | 0 |
| Product audit rules (`overlap.suspicious`, `visual.touching_controls`) | 0 issues | 0 issues (10 011 overlap pairs evaluated on the results surface alone) |

So the honest answer to "how many places are fused": **9 real ones** (3 container-level
from the zeroed shell + 6 icon/label rows), all fixed; the remaining scanner hits are
panel edges and rect-padding artifacts, each checked against painted pixels.

### Root causes

1. `Select` sized its trigger from `ui.available_width()`, floored the menu at a magic
   `MENU_MIN_WIDTH = 152`, and left the popup unpinned while its rows self-reported the
   screen width — three independent defects that all had to go for the popup to hug content.
2. egui propagates `item_spacing` into `Frame`/`ScrollArea`/`horizontal`/`vertical`; the
   workspace `CentralPanel` set it to `ZERO`, so every view that forgot an explicit gap
   inherited flush rendering. Views compensated locally instead of the shell being fixed.
3. A lucide glyph painted as a `ui.label` fills its em box, so `item_spacing.x` of 0–2 px
   puts the next label's ink inside the glyph strokes.


## Source: `VERIFICATION.md`

# Verification — UI consistency wave

Baseline audit: `89095cc7d4b056a9fcc19dd9368f7c2863c8759f`.
Fix commit: `0e370498` (`fix(ui): resolve UI consistency audit findings`) on `main`.

## Commands (actually executed)

| Command | Result |
|---|---|
| `cargo +nightly fmt --all -- --check` | pass, exit 0 |
| `cargo check --workspace` | pass, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass, exit 0 |
| `cargo test --workspace` | pass, exit 0 (includes new Select/contrast tests) |
| `cargo build --release --locked -p db-pro-native` | pass, exit 0 (`Finished release`, 4m 23s) |
| `bash scripts/check-ui-architecture.sh` | `UI architecture boundary: PASS` |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | 0 fail, 5 warn (pre-existing size/clone debt retained by ratchet) |
| `python3 tools/cc-scan.py . --fail-on error` | exit 1 — 21 legacy errors with an empty `.clean-code-baseline.json`; every flagged function predates this diff (including untouched `core`/`infrastructure`). The diff ratchet above is the wave gate; the baseline was **not** regenerated. |
| `python3 .skills/ui-consistency-audit/scripts/ui_consistency_scan.py` | 21 call sites, **0 contract findings**; flags: 12 width literals, 6 hand-rolled labels (pre-existing designer forms). |

## Runtime captures

`/tmp/ui-audit/wave2/` — 42 PNG, `capture.tsv` 43 data rows, all `captured` / `exit=0`.
Binary: `db-pro-native --features capture`, X11 under `xvfb-run`, 180 settle frames.

| Surface | Sizes | Theme | States captured |
|---|---|---|---|
| query (+ row-limit menu open) | 1280×800, 1440×900, 1920×1080 | dark + light | normal, menu open |
| results | 3 sizes | dark + light | populated grid, toolbar |
| new-connection | 3 sizes | dark + light | normal |
| connection-error | 3 sizes | dark + light | all-fields error + toast |
| settings | 3 sizes | dark | normal |
| compare | 3 sizes | dark | normal |
| history | 3 sizes | dark | expanded entry |
| diagram | 3 sizes | dark | zoom controls |
| explorer-filter | 3 sizes | dark | normal, schema loading (1280) |
| agent | 3 sizes | dark | composer |

## Finding verification

| ID | Status | Evidence |
|---|---|---|
| UI-01 | fixed | `explorer-filter-*-dark.png`: search ends before the Filter/Refresh slots; both buttons fully hit-testable. |
| UI-02 | fixed | `agent-*-dark.png`: Prompt label above the input, send button at the composer's bottom-right at all three sizes. |
| UI-03 | fixed (source) | `query_context_picker_view.rs`: schema list wrapped in `ScrollArea::new(theme).max_height(180.0)`, Escape closes, headers localized. Picker-open capture has no harness hook. |
| UI-04 | fixed | `query-*-dark.png` row-limit menu aligns with Run; `history-*-dark.png` filter is right-aligned. `standard_field_geometry_…` covers the Select trigger box. |
| UI-05 | fixed | `new-connection-*-{dark,light}.png`: engine Select follows the dialog theme; `Select` resolves `DbProTheme::from_visuals` when `.theme` is omitted. |
| UI-06 | fixed (source) | Disabled Select paints `surface_2` + `border_subtle` and muted text instead of looking enabled. No disabled-state capture hook. |
| UI-07 | fixed | Menu rows lay out through `option_label_job` (1 row, `break_anywhere`, `…`); handler tests cover the job. |
| UI-08 | fixed | `paint_option` publishes `WidgetInfo::selected(SelectableLabel, …)`; `MenuKeyAction` commits on Enter, reverts on Escape; `handler_tests.rs:73-77`. |
| UI-09 | fixed | Select label renders through `components::Label` + `LABEL_HELPER_GAP`; scanner contract findings = 0. |
| UI-10 | fixed | Designer forms now use `SelectSize::Default` beside 38 px Inputs; sidebar filter widths moved to `FILTER_*_WIDTH` tokens. Residual per-column form widths stay literals (P3 note). |
| UI-11 | fixed | Row caps localized (`query.row_limit_*`); capture shows the localized menu open. |
| UI-12 | fixed | Context chip is button-height (`BUTTON_HEIGHT_SM` minus margins/stroke) and centers with Run; `query-*-dark.png`. |
| UI-13 | fixed (source) | Chart config row uses `horizontal_wrapped` + fixed select widths, so narrow panes reflow instead of clipping. |
| UI-14 | fixed | `new-connection-*-*.png` at 1280/1440/1920; dialog frame scrollbar is `VisibleWhenNeeded`; error state still fits. |
| UI-15 | fixed | `connection-error-*-{dark,light}.png`: inline host/username (and all required) errors derive from `validate_connection_draft`; toast text matches and is localized. |
| UI-16 | fixed | `settings-*-dark.png`: symmetric ~40 px insets on both sides of the content column. |
| UI-17 | fixed | `results-*-*.png`: filter Input 38 px and toolbar buttons on one size token (`Default`/`Icon`). |
| UI-18 | fixed | `history-*-dark.png`: SQL block shrinks to content; "Copy SQL" wraps inside the rail. |
| UI-19 | fixed | `compare-*-dark.png`: page gutter on both sides, separated sidebar buttons, fields use Input label rhythm. |
| UI-21 | fixed | Status message soft-ellipsizes with a full-text tooltip; identity truncation also tooltipped. Visible in query/results captures. |
| UI-23 | fixed | Unused 24 px token removed; 16 px sheet title and empty-state icon use font helpers; no `.size(16.0)`/`.size(18.0)`/`font_page_title` remain in `crates/ui` or `crates/app`. |
| UI-24 | fixed | `dark_muted_text_meets_aa_on_elevated_surfaces` computes WCAG contrast of `NEUTRAL_450` (#999999) on elevated ≥ 4.5:1; test passes. |
| UI-25 | fixed (source) | Zoom reset exposes `WidgetInfo::labeled(Button, …)`; canvas/minimap publish named `Other` roles. Visual output unchanged; no screen-reader session run. |
| UI-20 | deferred | 13 one-off dialog widths + destructive footer refactor is a separate mechanical wave; most dialogs still lack capture hooks. |
| UI-22 | deferred | ~417 hardcoded strings need a dedicated i18n sweep; this wave only localized touched surfaces. |

## Evidence gaps

- Light mode has no capture hook for settings, compare, history, diagram, explorer, agent
  (the `light_env()` map in the capture script); those are dark-only.
- Loading/empty states were captured only for explorer (schema loading) and connection error;
  there are no empty-state hooks for results/history/settings.
- No vi/ja locale captures, no 800×600, no 125/150 % scale factor.
- Schema picker open state, disabled Select, chart wrap, and screen-reader output are
  source/unit-verified only.
- Repo-wide `cc-scan --fail-on error` fails on inherited legacy findings; see commands.

## Deferred

- **UI-20** — dialog width presets (13 one-offs) and destructive dialog footer/order.
- **UI-22** — full i18n sweep (~417 strings), including chart config labels.
- **P3 note** — residual literal widths in designer-form columns (120/140/180/220/260) are
  stable per-field widths, not shared-component drift; promote only with a form layout token.

---

# Wave 4 — Select width contract + workspace spacing rhythm

Baseline SHA: `f984385a`. Owner-approved scope: fix the root cause (shell spacing) and
re-verify the full surface matrix.

## Commands (actually executed)

| Command | Result |
|---|---|
| `cargo +nightly fmt --all -- --check` | 1 diff (the `main.rs` viewport match arm) → fixed with `cargo +nightly fmt --all`, re-checked clean (exit 0) |
| `cargo check --workspace` | pass, exit 0 (27.4 s) |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass, exit 0 (`Finished dev` in 19.8 s) |
| `cargo test --workspace` | pass — every suite `ok`, 0 failed; largest suite 712 passed (Select cap tests included), 64 ignored across provider suites |
| `cargo build --release --locked -p db-pro-native` | pass — `Finished release profile [optimized] target(s) in 4m 41s` |
| `bash scripts/check-ui-architecture.sh` | `UI architecture boundary: PASS` |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | `✓ pass: 11 ⚠ warn: 5 ✗ fail: 0` (same pre-existing size/clone debt as wave 3) |
| `python3 tools/cc-scan.py . --fail-on error` | exit 1 — 21 legacy errors / 125 warnings, **none in a file this wave touched** (grep of the report for `select/`, `shell_frame_view`, `component_gallery_view`, `tools_hub_surface_view`, `schema_compare_view` returns nothing). Baseline not regenerated. |
| `python3 .skills/ui-consistency-audit/scripts/fusion_scan.py /tmp/ui-audit/final/audit --gap 2.5` | `surfaces: 102  fused pairs: 12  (excluded adjacent-panel edge pairs: 130)` |
| `python3 .skills/ui-consistency-audit/scripts/ui_consistency_scan.py` | 148 advisory `ZERO-SPACING` candidates (was 142 pre-fix), 0 contract findings; every candidate that reaches screen was cleared by the runtime scan above |

## Runtime captures (full re-verify)

`/tmp/ui-audit/png/` — **108 PNG, all `captured`, 0 missing**; `/tmp/ui-audit/final/` —
102 `DB_PRO_INSPECTOR_DUMP` logs + audit JSON. Binary `db-pro-native --features capture`,
180 settle frames.

| Axis | Coverage |
|---|---|
| Surfaces | 17 (welcome, query, results, table, new/edit/error connection, settings, quick-open, explorer-filter, diagram, history, compare, agent, gallery, files, tools) |
| Sizes | 1280×800, 1440×900, 1920×1080 |
| Themes | dark for all 17; light for the 8 that have a `_LIGHT` fixture hook (welcome, query, results, quick-open, new-connection, connection-error, files, tools) |
| States | row-limit popup open at all 3 sizes × both themes |

Light-mode files for the other 9 surfaces are byte-identical duplicates of the dark
capture (no fixture hook exists) — recorded as an evidence gap, not as light coverage.

## Finding verification

| ID | Status | Evidence |
|---|---|---|
| UI-26 | fixed | Popup for a 108 px trigger: **598×156 px before → 145×158 px after** (dark 1440); `query-1920x1080-light-limit.png` shows all four options, including `100,000 rows`, fully legible with the check column reserved. |
| UI-27 | fixed | Shell `item_spacing` now `SPACE_SM × SPACE_XS`. Re-measured over 102 dumps: 142 raw pairs → 130 adjacent-panel edges + 12 × one tree-row pair that pixel measurement clears (see false positives). **0 confirmed fusions.** No view became over-spaced: welcome/query/results light at 1920 and 1280 reviewed. |
| UI-28 | fixed | Six icon+label rows at `ICON_TEXT_GAP`; `tools-1440x900-dark.png` ("⤓ Schema & Migration"), `agent-1440x900-dark.png` ("⚡ Agent", connection chips). |
| UI-29 | fixed | `gallery-1440x900-dark.png` — stray 13×24 separator stub gone, nav column separated by `SPACE_MD`. |
| UI-30 | open (pre-existing) | A/B rebuild at `f984385a` minus the shell change (`/tmp/c0.png`) clips identically → not a wave-4 regression. |
| UI-31 | open | Source-level: shared `ScrollArea` sets `ScrollBarVisibility::AlwaysHidden`. |

## Harness change

`crates/native-app/src/main.rs` — capture mode (`DB_PRO_WINDOW_SIZE` pinned) now creates
an **unmapped X11 window** (`.with_visible(!cfg!(target_os = "linux"))`). Verified by
capturing `query-1440x900-dark.png` on the desktop display with no window appearing; it
differs from the Xvfb capture in 13 554/1 296 000 pixels (1 %, caret/AA). macOS keeps
`visible: true` — a hidden window there never receives `drawRect` and the capture stalls.
The capture script header now tells callers to use Xvfb regardless.

## Wave-4 evidence gaps

- Light theme only for 8/17 surfaces (fixture hooks); no light hook for settings,
  compare, history, diagram, explorer, agent, gallery, files-dark, table.
- No empty/loading state re-capture beyond quick-open and connection error.
- No vi/ja locale, no 800×600, no 125/150 % scale factor.
- `widest_label_width` is bounded by a per-frame `cap` early-exit, but there is no
  frame-time measurement for a Select with hundreds of options (schema picker).


## Wave 5 — dialog size policy (UI-32)

### Measurement (before)

`DB_PRO_CAPTURE_SETTLE_FRAMES` probe on the New Connection dialog at 1920×1080, card
vertical extent read off the PNG:

| frames | card y | height |
|---|---|---|
| 4 | 281–714 | 434 |
| 6 | 263–868 | 606 |
| 8 | 211–920 | 710 |
| 12 | 107–1005 | 899 |
|

… [truncated for memory entry; see source file]

