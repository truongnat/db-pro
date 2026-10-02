# Gallery & component layout collisions

Baseline SHA: `78d914887642dd400c7e2048db9a24d0403619c8` (main, working tree).

## Problem

Native capture reproduction (1280×800, `db-pro-native --capture` harness)
confirmed four layout collisions on the Component Gallery surfaces, matching the
owner-reported screenshots:

1. **Explain plan stat overlap** — `ExplainPlanTree::render_node`
   (`crates/ui/src/components/explain/ui.rs`) anchors the per-node stat text with
   `Layout::right_to_left` filling the row's remaining width. When the remaining
   width is smaller than the stat text, egui places the label so its left edge
   slides under the icon/title/badges. Used in production via
   `crates/ui/src/query_output_actions_view.rs`.
2. **Connection card column overflow** — `ConnectionCard::show`
   (`crates/ui/src/components/database/ui.rs`) reserves `SPACE_XS` before the SSL
   badge, but egui additionally inserts `item_spacing.x` (8pt) between
   `add_sized` and `allocate_exact_size`. The row claims 8pt more than the inner
   width, so the card's `min_rect` extends into the `ui.columns(2)` gutter and
   under column 2's content. Measured: column `562..899`, card `562..907`.
3. **StatusBar left/right collision** — `StatusBar::show`
   (`crates/ui/src/components/workspace/ui.rs`) paints left items forward from
   the left edge and right items backward from the right edge with no bound, so
   `READ COMMITTED` (left) overlaps `Ln 42, Col 18` (right) at gallery width.
4. **Light-theme glyph samples invisible** — `draw_rendering_text_samples`
   (`crates/ui/src/component_gallery_rendering.rs`) hardcodes
   `Color32::from_rgba_unmultiplied(224, 228, 235, alpha)` — a light gray chosen
   for dark surfaces — making every alpha row nearly invisible in light theme.

## Severity

P2 — non-blocking UX/polish, no data or correctness impact. The explain-plan
overlap also affects the production query output surface.

## Scope

Presentation-only fixes in `crates/ui`; no runtime/provider behavior changes.
The production `QueryStatusBar` (`query_status_bar_surface_view.rs`) lays out
right-then-left inside a bounded scope and was not observed colliding — out of
scope.

## Acceptance

- `render_node` stat region is allocated at least as wide as its text; narrow
  rows widen the ScrollArea content instead of overlapping.
- `ConnectionCard` response width ≤ its column width (verify at 1280×800).
- `StatusBar` never paints left items across the right block; clipped items are
  indicated with "…" and remain discoverable via hover.
- Text-rasterization samples derive their base color from `theme.text_primary`.
- Runtime captures: gallery `database-shell` + `devtools` + `rendering`
  sections at 1280×800, dark + light.
