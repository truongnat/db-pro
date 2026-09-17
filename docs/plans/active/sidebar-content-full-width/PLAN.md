# Sidebar content full width

## Problem

Navigator tree content did not fill the left sidebar the way VS Code / DBeaver do.
Screenshot evidence: tree labels / `Failed:` hints clipped around ~200px while
`New query` / filter chrome reached ~360px and the drag separator sat near ~445px.

## Cause

1. `ScrollArea` content width settled on intrinsic (short-label) size, so tree rows
   truncated mid-panel.
2. Resize handle used egui `SidePanel` frame `response.rect`, which could disagree
   with the `exact_width(sidebar_width)` we requested — separator floated past content.

## Fix

- Derive sidebar content rect from clamped `sidebar_width` (asymmetric pad: left
  `SPACE_SM`, right `SPACE_XXS`).
- Lock drag separator to `panel_left + sidebar_width`.
- Force explorer `ScrollArea` + tree rows to the padded content width every frame.

## Scope

`crates/ui` only: `sidebar_view.rs`, `explorer_view.rs`, `explorer_tree.rs`.

---

## Follow-up: the width contract broke on the clip (2026-09-17)

The `max_rect`-based width above is a *preference*, not a cap. Four defects followed.

### Cause

1. `Ui::set_max_width` ends with `max_rect = max_rect.union(min_rect)` (`placer.rs`), so a
   sibling that overflowed earlier in the frame inflates `max_rect` for the rest of that
   frame. The explorer toolbar overflowed: it reserved a hardcoded `actions_width = 28.0`
   for the refresh button, but `compact_icon_button` is a style-sized `Button` and measures
   **34** wide. Field 240 + spacing 8 + button 34 = 282 inside a 240px column.
2. Everything downstream sized from `max_rect()` / `available_width()`, so tree rows were
   **282** wide in a **240** column. `Painter::with_clip_rect` *intersects*, so the overflow
   was discarded without complaint: the `Native Test` driver badge was painted at
   `x 239–286` against a clip ending at `248`, leaving a ~1px sliver of its label.
3. The badge width was additionally estimated per character — `badge.len() * 6.5 + 8.0`
   yields 47 for `SQLITE`, whose measured label is 31.
4. Separately, the sidebar clipped to exactly the content column while content filled it.
   `Shape::rect_stroke` paints *entirely outside* its path (`StrokeKind::Outside`), so every
   1px border on a widget filling the column lost both vertical edges — the filter field,
   the `New query` button, the header action — while the horizontal ones survived, because
   those sit well inside the clip.

### Fix

- `explorer_view`: lay the refresh button out first in a `Layout::right_to_left` row and give
  the field the remainder, so the button's width is *measured*, never assumed.
- `explorer_view`: bound `tree_width` by the clip as well as `max_rect`, and do it *before*
  the `ScrollArea`. Inside the row the clip is narrowed by the scrollbar
  (`inner_size = outer_size - current_bar_use`), so a clip-derived row would jitter narrower
  every time the connection list crossed the fold.
- `explorer_tree`: measure the badge label with the painter instead of estimating it.
- `sidebar_view`: clip to the content column plus a 1px bleed — the least room a 1px outside
  stroke needs to render, and far too little to hide a real layout overflow.
- `components/input`: `paint_field_chrome` strokes an inset path, so the field's own border
  lands inside the field whatever container it sits in.

### Scope

`crates/ui` only: `sidebar_view.rs`, `explorer_view.rs`, `explorer_tree.rs`,
`components/input/{layout,search}.rs`.
