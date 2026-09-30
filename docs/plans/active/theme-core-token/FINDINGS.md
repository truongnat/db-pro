# Findings

## P2 — Core values had multiple owners (slice 1, fixed)

`DbProTheme` owned raw colors and semantic roles in the same constructors.
`tokens.rs` and component config files also owned overlapping button values.
This allowed visual drift without changing a semantic token name.

**Resolution (this slice):** raw colors live in `tokens/primitive.rs`
(palettes), roles in `tokens/semantic.rs`, component-owned sizes in
`tokens/component/*`, and `DbProTheme` is a flat compatibility facade built
from `SemanticTokens`.

## Decision — layering and ownership

- Primitive palettes are private to the `tokens` module tree: component
  rendering cannot read a raw theme value; colors arrive through
  `DbProTheme`/`theme.semantic`. Numeric scales stay reachable via the
  `crate::tokens` facade so existing `use crate::tokens::*` imports resolve
  unchanged (single-definition re-exports, no duplicated values).
- `DbProTheme` gained one additive field, `semantic: SemanticTokens`, so
  components that need role detail (alert fill/border pairs) stop multiplying
  raw values at the paint site. No struct-literal construction exists outside
  `theme.rs` (verified at `bd787db1e5d75862ecd08c86539e910d4e651ebd`), so the
  added field breaks no caller.
- Renderer adapter constants (`EGUI_ITEM_SPACING`, `EGUI_BUTTON_PADDING`,
  `EGUI_INTERACT_SIZE`, `EGUI_WINDOW_MARGIN`, `EGUI_MENU_MARGIN`,
  `EGUI_INDENT`, `RADIUS_EGUI_*`) moved from `tokens.rs` into private consts
  in `theme.rs` — their only consumer was `theme.apply()` (grep verified).

## P2 — State-precedence defects (fixed, measured)

- **Button `disabled + loading` rendered the spinner.** `Button::show()`
  branched on `loading` before consulting `enabled`, so the gallery's
  `.enabled(false).loading(true)` sample painted loading colors instead of the
  disabled palette. Fixed via `button_contract::shows_loading`
  (`disabled → loading`); observable only in the disabled+loading combination.
- **Disabled input could wear focus/error borders.** `paint_field_chrome`
  evaluated `has_error`/`focused` before `!enabled`, contradicting the
  required precedence. Fixed via `input_contract::resolve_chrome`
  (`disabled → error → focus → hover → default`); enabled-state rendering is
  byte-identical (the hover lerp still ticks on exactly the previous
  condition).

## Decision — compatibility aliases (one authoritative role)

- `surface_2` → authoritative role `background.subtle` (quiet fill: disabled
  button fills, card icon boxes). Value differs from `background.panel` in both
  themes, so it cannot collapse into another role without a visual change.
- `text_muted` → authoritative role `foreground.muted`. Light coincides with
  `foreground.tertiary` (#8a8a8a); dark deliberately differs (#8c96a0 vs
  #828c97), so both roles stay (pinned by test).
- `border.separator` is defined per the prescribed role list with the value of
  `border.default` and currently has no consumer — documented in the role so
  future separator work has an owner instead of an inline literal.
- `StatusRole.foreground` equals `solid` in the shipped palettes (alert icons
  consume it); it exists so on-status text can diverge without touching
  consumers.

## Decision — button dimensions: 32/38 is authoritative

At baseline `bd787db1`, `tokens.rs` carried `BUTTON_HEIGHT_DEFAULT = 36.0` and
`BUTTON_HEIGHT_LG = 42.0` with **no consumers**, while
`components/button/config.rs` owned the rendered literals 32.0/38.0. Slice 1
re-pointed config at the tokens and corrected the tokens to 32.0/38.0, so
rendered dimensions never changed. The Button contract now owns
28/32/38 (pinned by `dimensions_preserve_the_existing_rendered_values`).

## Notes — classification kept out of this slice (P2, deferred)

- `DIALOG_RADIUS = 16.0` (product dialog frame, `components/dialog/config.rs`)
  vs `tokens::RADIUS_DIALOG = 6.0` (egui window rounding via the adapter) are
  different surfaces with confusingly similar names; documented in the
  Dialog/Overlay contract instead of renamed.
- `TABLE_ROW_HEIGHT_COMPACT` is consumed by `components/select` menu rows, not
  by the table component; moved under the Table contract as its name implies,
  with the quirk recorded here.
- Pre-existing dead constants kept (not introduced by this slice):
  `TREE_ROW_HEIGHT`, `SIDEBAR_ROW_HEIGHT`, `TABLE_ROW_HEIGHT_DEFAULT`,
  `TABLE_HEADER_HEIGHT`, and the `BUTTON_ICON_SIZE_*` hit-box vectors (the
  icon hit-box/glyph separation the naming rules ask for — hit-box sizes now
  sit next to glyph sizes in the Button contract).
- Editor helpers (`editor_gutter_fill`, `editor_current_line_fill`,
  `editor_selection_fill`, `editor_line_number`) still own raw values in
  `theme.rs`; they are derivation helpers outside the five representative
  contracts and deferred to a follow-up editor-role slice.
- Badge borders still apply inline `linear_multiply(0.4/0.5)` multipliers
  (`components/badge/handler.rs`); they differ per variant (0.4 success, 0.5
  others) and were left as component-local rather than silently unified.
- Clean-code scan warnings accepted with reason: 6 `as` casts are the
  verbatim `soft_tint` recipe moved into `semantic::subtle_wash` (changing
  them to `try_from` is impossible on floats and would risk value drift);
  `paint_field_chrome` (5 params) and `paint_focus_ring` (4 params) are
  inherited signatures; `SemanticTokens::light/dark` (54 lines) stay as one
  constructor per theme so a reviewer can diff them 1:1 against the baseline
  constructors.
