---
schema_version: 1
id: "MEM-SRC-b04813d1fe99caef"
title: "Source mirror: docs/plans/active/viewport-select-fit/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/viewport-select-fit/FINDINGS.md"
source_sha256: "b04813d1fe99caefed10d2d4d801c72fd8fdc8fa666878a79241a1cc6460714a"
mirror_body_sha256: "b04813d1fe99caefed10d2d4d801c72fd8fdc8fa666878a79241a1cc6460714a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Viewport Select and Radio Allocation Fit — Findings

## Severity Legend
- **P0:** Blocker, crash, data loss.
- **P1:** Correctness failure, major broken user flow.
- **P2:** UI layout defect, visual polish, maintainability.

## Findings Inventory

### F1: Horizontal Radio / Checkbox Overflow (P2)
- **File / Line:** `crates/ui/src/components/selection/ui.rs:86-104`, `376-388`
- **Symptom:** `Radio` allocated `ui.available_width()`, taking the entire remaining row in horizontal layouts. In Settings -> Appearance, "Light" pushed "Dark" off the card's right boundary, causing text clipping.
- **Root Cause:** Sizing logic lacked distinction between horizontal flow and vertical stacked flow.
- **Resolution:** Introduced `.compact(bool)` and automatic detection of horizontal parent direction (`ui.layout().main_dir() == egui::Direction::LeftToRight`). In compact/horizontal mode, allocated width is derived from measured text galley and icon size instead of full available width.

### F2: Oversized Sparse Query Context Picker (P2)
- **File / Line:** `crates/app/src/query_workbench/query_context_picker_view.rs:27-40`, `47-84`, `96-128`
- **Symptom:** The query context picker opened at ~600px width and ~450px height even for short lists (2 connections, 1 schema), dominating the screen with blank space.
- **Root Cause:** `ui.set_min_width(280.0)` did not pin or constrain width; child rows called `paint_option` which allocates `ui.available_width()`. Additionally, both `ScrollArea` instances defaulted to `auto_shrink([false, false])`, expanding to `max_height(180.0)` each.
- **Resolution:** Content width is `anchor.width().clamp(300.0, 360.0).min(max_available)`, where
  `max_available = (content_rect.width() - VIEWPORT_GUTTER - SURFACE_INSET).max(1.0)`. The
  `VIEWPORT_GUTTER` (16) and `SURFACE_INSET` (18 = frame margin 8×2 + border 1×2) are named
  constants, so the popup's outer width never exceeds the viewport even when it is narrower than
  the 300 pt preference. Each list is an **allocated region** whose height is
  `rows × (row height + gap)`, capped at 180 pt, and the scroller fills that region. `auto_shrink`
  alone was not enough: inside an auto-sized `Area` the child `Ui` only offers the height the
  previous frame produced, so a shrinking scroller converged to two visible rows and left the rest of
  a long list unreachable — a 24-schema capture now shows five rows in the bounded window.

### F3: Duplicate ScrollArea ID Collision (P2)
- **File / Line:** `crates/app/src/query_workbench/query_context_picker_view.rs:43,78`
- **Symptom:** Debug warning banners ("First use of ScrollArea ID 45C9" and "Second use of ScrollArea ID 45C9") painted directly over picker options in both themes.
- **Root Cause:** Both `ScrollArea::new(context.theme)` calls were initialized in the same `ui` context without unique `id_salt`.
- **Resolution:** Assigned explicit, distinct salts: `"query_context_connections_scroll"` and `"query_context_schemas_scroll"`.

### F4: Component-First Exemption in Settings Appearance (P2)
- **File / Line:** `crates/app/src/settings/settings_appearance_view.rs:49-52`
- **Symptom:** Direct invocation of raw `ui.checkbox` instead of `crates::components::Checkbox`.
- **Resolution:** Migrated to `crate::components::Checkbox::new(&mut self.preferences.reduce_motion, "Reduce motion", *self.theme)`.

### F7: Light chrome was indistinguishable from the canvas (P2, owner-reported)
- **File / Line:** `crates/ui/src/theme/tokens/semantic.rs` (light `background.panel`), `primitive.rs` (light ramp)
- **Symptom:** In light mode the sidebar, topbar and status bar shared almost the same fill as the white
  editor: `panel` was `#f9f9f9` against a `#ffffff` canvas — a 6/255 step that read as one region, so
  panels did not look like separate areas.
- **Root Cause:** the chrome role carried a step too small to be perceived on large flat areas.
- **Resolution:** light `panel` → `NEUTRAL_75` (`#f6f6f6`), one visible step below the canvas, while
  `elevated`/`floating` stay `#ffffff` so cards and popovers read as raised above the chrome. Dark
  tokens are unchanged (its `panel` already sits below `canvas`). The step deliberately stops short
  of `NEUTRAL_100` (`#f1f1f1`) because that equals light `hover` and would make a hovered sidebar row
  identical to its base fill; the ladder is canvas → panel `#f6f6f6` → hover `#f1f1f1` → selected
  `#eeeeee`, now pinned by a regression test.
- **Accessibility consequence:** light `tertiary`/`muted` moved to a new `NEUTRAL_525` (`#6b6b6b`,
  4.93:1 on the new panel) so the theme contrast test keeps enforcing 4.5:1. The two now-unused light
  steps (`NEUTRAL_50`, `NEUTRAL_500`) were deleted.
- **Related role fix:** four transient popups (SQL completion, rich hover, signature help, table
  column suggestions) painted `surface_panel`; with chrome now darker they would read as sunken, so
  they use `surface_elevated`, the documented role for transient surfaces.

### F5: Compact Row Cap Read the Wrong Width (P2, caught by the new narrow test)
- **File / Line:** `crates/ui/src/components/selection/ui.rs` (`Checkbox::show`, `Radio::show`)
- **Symptom:** The first compact implementation capped `row_width` with the width reported *inside*
  `ui.horizontal`. A 70 pt parent produced an 84 pt row — the child row is not bounded by a narrow
  parent, so the cap did not hold and the control still overflowed.
- **Resolution:** Capture `parent_available = ui.available_width()` before entering the horizontal
  child and use it for both the text wrap width and the row cap. The glyph+gap floor
  (`CONTROL_SIZE + CONTROL_SPACING`) remains the documented minimum when the parent is narrower
  than the indicator itself. Covered by `compact_rows_honour_a_parent_wider_than_the_glyph_floor`
  and `compact_rows_keep_the_glyph_floor_in_a_sliver_parent`.

### F6: Picker Viewport Cap Ignored Its Own Surface Inset (P2, caught by the new narrow test)
- **File / Line:** `crates/app/src/query_workbench/query_context_picker_view.rs`
- **Symptom:** A 240 pt viewport produced a 242 pt popup: the content width was capped at
  `viewport - 16`, but the floating frame adds its own margin (8×2) and border stroke (1×2), so the
  outer surface still exceeded the viewport.
- **Resolution:** Subtract both the viewport gutter and `SURFACE_INSET` from the available width.
  Covered by `query_context_picker_caps_to_a_narrow_viewport`.
