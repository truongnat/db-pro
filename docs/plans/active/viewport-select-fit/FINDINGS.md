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
  the 300 pt preference. Applied `auto_shrink([true, true])` to both scroll areas so heights shrink
  to content. Added clean fallback empty state labels when lists are empty.

### F3: Duplicate ScrollArea ID Collision (P2)
- **File / Line:** `crates/app/src/query_workbench/query_context_picker_view.rs:43,78`
- **Symptom:** Debug warning banners ("First use of ScrollArea ID 45C9" and "Second use of ScrollArea ID 45C9") painted directly over picker options in both themes.
- **Root Cause:** Both `ScrollArea::new(context.theme)` calls were initialized in the same `ui` context without unique `id_salt`.
- **Resolution:** Assigned explicit, distinct salts: `"query_context_connections_scroll"` and `"query_context_schemas_scroll"`.

### F4: Component-First Exemption in Settings Appearance (P2)
- **File / Line:** `crates/app/src/settings/settings_appearance_view.rs:49-52`
- **Symptom:** Direct invocation of raw `ui.checkbox` instead of `crates::components::Checkbox`.
- **Resolution:** Migrated to `crate::components::Checkbox::new(&mut self.preferences.reduce_motion, "Reduce motion", *self.theme)`.

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
