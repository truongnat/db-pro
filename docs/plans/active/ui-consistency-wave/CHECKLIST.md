# Checklist — UI consistency wave

## Skill
- [x] `SKILL.md` with self-improve close-out
- [x] `references/method.md`, `heuristics.md`, `findings-ledger.md`
- [x] `scripts/ui_consistency_scan.py` (21 call sites, 0 contract findings after fixes)
- [x] `scripts/capture_ui_matrix.sh` (syntax-checked; cargo target + FILTER/CONNECTION_ERROR fixed)

## P1
- [x] UI-01 Explorer toolbar: Refresh/Filter no overlap
- [x] UI-02 Agent composer top-down order
- [x] UI-03 Schema picker scroll + keyboard + i18n headers

## Select contract
- [x] UI-04 Layout-neutral show
- [x] UI-05 Theme required; engine call site themed
- [x] UI-06 Disabled visual
- [x] UI-07 Menu row truncation
- [x] UI-08 Option widget_info + Escape revert
- [x] UI-09 Label matches form Label
- [x] UI-10 Size/width rhythm in forms and sidebar
- [x] UI-11 Row-cap i18n
- [x] UI-12 Query chip height + row alignment
- [x] UI-13 Chart select wrap

## Report-1 layout
- [x] UI-14 New Connection scroll
- [x] UI-15 Required inline errors
- [x] UI-16 Settings symmetry
- [x] UI-17 Results toolbar heights
- [x] UI-18 History rail dead space / clipped button
- [x] UI-19 Schema Compare touch pairs
- [x] UI-21 Status bar tooltip
- [x] UI-24 Dark muted contrast
- [x] UI-25 Diagram accessible names
- [x] UI-20/22 documented as deferred (dialog width wave, i18n sweep)
- [x] UI-23 type scale (unused 24 token removed, titles use font helpers)

## Verification
- [x] Unit tests for pure layout/selection decisions (`menu_key_action`, `option_label_job`,
      field geometry incl. Select, dark muted contrast)
- [x] `cargo +nightly fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `cargo build --release --locked -p db-pro-native`
- [x] `bash scripts/check-ui-architecture.sh`
- [x] Clean-code diff ratchet (0 fail); repo-wide cc-scan legacy errors recorded
- [x] Runtime captures: 42 PNG, 10 surfaces × 3 sizes, dark; light for query/results/
      new-connection/error; menu open captured. Light-hook gaps recorded.
- [x] Ledger statuses + evidence paths updated

## Wave 4 — Select width + spacing rhythm
- [x] UI-26 Select trigger content-hugging; popup measured from the widest option and
      pinned with `set_width` (598×156 px → 145×158 px)
- [x] UI-27 Shell `item_spacing` = theme rhythm; redundant per-view compensations removed
- [x] UI-28 Six icon+label rows → `ICON_TEXT_GAP`
- [x] UI-29 Gallery separator stub removed
- [x] UI-30 / UI-31 filed as open, not silently expanded into this wave
- [x] `widest_label_width` bounded by a `cap` early-exit + 2 new unit tests
- [x] New scanner `fusion_scan.py` (painted geometry, container edges counted separately)
- [x] Full re-verify: 108 PNG (17 surfaces × 3 sizes × dark+light, 8 with light hooks)
      + 102 inspector dumps; 0 confirmed fusions after pixel cross-check
- [x] Capture mode no longer opens a visible X11 window (`with_visible(!linux)`) +
      Xvfb documented in the script header
- [x] Gates re-run after the final edit (fmt/check/clippy/test/release/architecture/clean-code)
