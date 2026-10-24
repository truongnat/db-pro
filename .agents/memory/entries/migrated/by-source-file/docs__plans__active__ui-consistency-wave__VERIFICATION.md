---
schema_version: 1
id: "MEM-SRC-a864c14ce0fa2bc0"
title: "Source mirror: docs/plans/active/ui-consistency-wave/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-consistency-wave/VERIFICATION.md"
source_sha256: "a864c14ce0fa2bc0a0ddcc1d2d9d8b9f9178b9edd09449da91490435efc71e9c"
mirror_body_sha256: "a864c14ce0fa2bc0a0ddcc1d2d9d8b9f9178b9edd09449da91490435efc71e9c"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
| 20+ | 91–989 | 899 |

The card grew 434 → 899 px while its top slid 281 → 91. Cause: `DialogFrame::body` is
`auto_shrink([false, true])` (height follows content) and `calculate_dialog_layout`
centers on `prev_height`, so every late-arriving row (catalog, validation line, expanded
section) resizes the card and drags it upward. Steady state was 83 % of viewport height.

A one-line experiment (`form_row` reserving `0.0` instead of `ui.available_height()`)
moved the steady state to 972 px, i.e. it was not the cause; reverted.

### Measurement (after)

Same probe with `.body_height(DIALOG_FORM_BODY_HEIGHT)` on the connection form:

| frames | observation |
|---|---|
| 6 / 8 / 12 / 20 / 60 | card bottom edge identical (y=863–864) |
| 12 vs 60 | 0.13 % of pixels differ, inside the focused input row (caret) |

1280×800: card 57–745, footer pinned, overflow below the fold scrolls. 1920×1080: card
150–866 with the same scroll behaviour. Quick Open (palette) was pinned then reverted:
with 9 results the box showed ~200 px of dead space, so the palette stays content-sized.

### Evidence status

Runtime-verified: connection form (2 sizes), palette (rejected with evidence).
Pinned without a runtime capture — no fixture opens them yet: insert row, pending table
changes, row conflict, cell inspector. The pin path is one shared implementation, so the
mechanism is covered; the per-dialog *value* (560) is not screenshot-verified.
