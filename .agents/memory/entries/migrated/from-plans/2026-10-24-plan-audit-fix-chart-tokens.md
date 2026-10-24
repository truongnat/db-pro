---
schema_version: 1
id: "MEM-AUDIT-fix-chart-tokens-bc56ca4e8a68"
title: "Plan archive: Audit Fix Chart Tokens"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, audit-fix-chart-tokens]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "bc56ca4e8a6889a58ee92b25697ae2abf038ed62866cfd84b88967b478d1f4fe"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/_audit-2026-10-13/fix-chart-tokens.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ audit slice `docs/plans/_audit-2026-10-13/fix-chart-tokens.md`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

# Fix handoff — Chart series tokens + P2 Color32 sweep (slice 05)

Date: 2026-10-13. Worker: `ChartTokensFix`.

## What changed

### P1 — chart palette → DbProTheme tokens

- `crates/ui/src/tokens/primitive.rs` — added categorical data-viz hues to both
  raw palettes: `light::CHART_RED_600`, `light::CHART_PURPLE_600` (light chart
  blue/amber/green/cyan reuse the existing `BLUE_700`, `AMBER_600`, `GREEN_600`,
  `CYAN_700` primitives — same hex values), and `dark::CHART_BLUE_400` /
  `CHART_RED_400` / `CHART_YELLOW_400` / `CHART_GREEN_500` /
  `CHART_PURPLE_400` / `CHART_CYAN_500`.
- `crates/ui/src/tokens/semantic.rs` — new `Chart` role group
  (`pub series: [Color32; 6]`) + `SemanticTokens.chart`, wired in `light()` and
  `dark()`. Doc comment records indexing semantics (`i % series.len()`).
- `crates/ui/src/theme.rs` — new `DbProTheme.chart_series` flat facade field
  (mirrors `semantic.chart.series`), wired in `light()`/`dark()`. New test
  `chart_series_stay_legible_on_the_canvas` asserts every series color keeps
  ≥3.0 contrast vs `surface_app` in both themes.
- `crates/ui/src/chart_view.rs` — deleted `PALETTE_LIGHT`, `PALETTE_DARK`, and
  the `palette()` helper (12 literals gone). `series_color` and both `draw_pie`
  call sites now read `theme.chart_series`. Color values are unchanged from the
  old palettes, so rendered output is identical.

### P2 — production-visible shadow/backdrop literals

- `components/alert/ui.rs` — modal backdrop now paints `self.theme.overlay`
  (the `background.overlay` scrim role already used by dialog/sheet) instead of
  `from_black_alpha(140)`; dialog shadow is `self.theme.floating_shadow()`
  (theme-aware alpha 20 light / 40 dark vs old flat 80; same offset/blur as the
  primitive elevation scale). Dropped `Color32` import.
- `components/alert/config.rs` — removed `BACKDROP_OPACITY` and
  `DIALOG_SHADOW_{OFFSET_Y,BLUR,OPACITY}` (now dead); offset/blur live in the
  primitive elevation scale.
- `components/hover_card/ui.rs` + `config.rs` — `Shadow` replaced by
  `self.theme.floating_shadow()` per the component README's "standard shadow"
  contract (old: offset 4 / blur 16 / alpha 40 → new: offset 8 / blur 24 /
  alpha 20 light, 40 dark). Dropped `SHADOW_{ALPHA,BLUR,OFFSET_Y}` config
  constants and the `Color32` import.
- `components/legacy.rs` — `toast()`'s hand-painted drop shadow now uses
  `theme.floating_shadow().color` (alpha 20 light / 40 dark) instead of
  `from_black_alpha(20)`; identical in light, slightly stronger in dark.

## Intentionally left (P2, noted for audit record)

- `components/animation.rs:137` `faded_overlay` — derives alpha from the
  `overlay` parameter (callers pass `theme.overlay`); not a literal.
- `component_gallery_rendering.rs` — dev-gallery fixture colors (backdrop/shadow
  swatches, `from_gray(96)` raster reference) intentionally fixed; not product UI.
- `dev_tools/mod.rs:311-317` — debug-overlay rect/text colors (dev-only tool).
- `Color32::WHITE` at `diagram_view.rs:671,685,950`, `tree/ui.rs:219`,
  `toggle/ui.rs:242`, `command/ui.rs:201` — measurement-only galleys, never
  painted; slice-04 already verified diagram painted text uses theme tokens.
- `Color32::PLACEHOLDER` galley fallbacks (`selection/ui.rs`, `code/ui.rs`,
  `diff/ui.rs`, `workspace/ui.rs`, `agent_*`, `schema_compare_view.rs`, etc.) —
  egui fallback used only when a glyph lacks a color; the galleys are laid out
  with explicit theme text colors, so the fallback never fires. Benign by
  convention (component-layer-structure VERIFICATION rejected this concern).
- `tree/ui.rs:186`, `legacy.rs:452` (`tinted`) — `from_rgba_unmultiplied` helpers
  that re-alpha a caller-supplied theme color; no literal present.
- `theme.rs` internals (`apply()`, `floating_shadow`, `text_on_solid`) and
  `tokens/` — raw-value layer by design.

## Verification

- `cargo fmt --check -p db-pro-ui` — PASS (shared tree, after fix).
- `cargo check -p db-pro-ui` — PASS in an isolated scratch tree
  (HEAD + only my files) and again on the shared tree once a sibling's
  mid-flight `table_structure_view.rs` edit was fixed. Only sibling-owned files
  erred during the interim.
- `cargo test -p db-pro-ui --lib -- theme:: tokens:: alert hover_card chart` —
  51 passed, 0 failed, incl. new `chart_series_stay_legible_on_the_canvas`.
- `grep -n "Color32::" chart_view.rs alert/ui.rs hover_card/ui.rs` — zero hits.

## Not done / left for others

- `tokens/component/dialog.rs` doc comment still says elevation comes through
  `floating_shadow` "(`SHADOW_OFFSET_Y`, `SHADOW_BLUR`, `WINDOW_SHADOW_ALPHA`)"
  — `floating_shadow` actually uses `SHADOW_ALPHA_LIGHT/DARK`. Pre-existing doc
  drift; not fixed here (doc-only, outside my literal-fix scope).
- `palette_surface_view.rs` raw spacing/radius literals and
  `component_gallery_view.rs:347` `196.0` rail width — other P2s in the slice,
  not part of this assignment.

