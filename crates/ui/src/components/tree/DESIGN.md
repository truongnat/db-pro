# Tree design

`mod.rs` re-exports `TreeNodeKind`, `DatabaseTreeNode`, and `reveal_children`. `ui.rs` allocates each row, paints icons/labels/details and chevrons, and clips animated child content. `handler.rs` owns node icon mapping, row layout/background, click and focus decisions, and reveal geometry. `config.rs` stores tree dimensions, animation thresholds, and legacy state ID salts.

## Event flow and cost

The caller builds rows in hierarchy order and supplies each row's depth. A row response reports clicks to the caller; when `.expanded(&mut bool)` is supplied, the same row click toggles that state. The caller uses `reveal_children` with a stable egui ID to draw nested rows under an animated height clip. The helper remembers the last child height and uses a safe initial estimate before measurement. Rendering cost is proportional to rows the caller builds/draws; this component does not own tree data, perform lazy loading, or virtualize children.

## Accessibility and layout

Rows publish button metadata with name, optional detail, and selected state. Focused rows get a theme focus outline; selected rows use a distinct background and accent. The chevron slot is reserved even for leaves, keeping sibling columns aligned. Depth increases indentation linearly; extremely deep trees leave less width for labels. Long names/details are painter-rendered in the row and can collide in narrow panels, so callers should bound displayed detail or provide a wider scrollable parent. Reveal and chevron motion use the shared egui animation helpers.
