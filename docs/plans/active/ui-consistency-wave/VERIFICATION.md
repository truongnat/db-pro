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
