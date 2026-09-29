# Tree

Native database tree rows for the egui UI. The component renders a compact database-object row, optional right-aligned detail text, optional selection styling, and optional expand/collapse state for nested content.

## Usage

```rust
let mut expanded = true;

DatabaseTreeNode::new("public.users", TreeNodeKind::Table, 1, theme)
    .detail("42 columns")
    .selected(false)
    .expanded(&mut expanded)
    .show(ui);

reveal_children(ui, ui.id().with("public.users.children"), expanded, |ui| {
    DatabaseTreeNode::new("id", TreeNodeKind::PrimaryKey, 2, theme).show(ui);
});
```

## Public API

- `DatabaseTreeNode::new(name, kind, depth, theme)`: creates one row.
- `.detail(text)`: paints secondary right-aligned metadata.
- `.selected(bool)`: applies selected background, accent rail, and selected colors.
- `.expanded(&mut bool)`: enables a chevron and toggles the mutable state when the row is clicked.
- `TreeNodeKind`: database object categories and their legacy Lucide icon mapping through `TreeNodeKind::icon()`.
- `reveal_children(ui, id, open, add_contents)`: animates nested row visibility with clipped child layout.

## Behavior and constraints

Rows reserve the chevron slot even when a node is a leaf so icon and label columns stay aligned across siblings. The handler owns icon mapping, row/background color decisions, expansion toggling, chevron crossfade layers, row geometry, and reveal clipping math. The UI layer owns egui allocation, painting, animation calls, and child UI clipping.

Animation state salts are kept stable (`hover`, `chev_anim`, and `content_h`) so existing egui animation/data identity semantics are preserved across the migration. Shared colors come from `DbProTheme`; `config.rs` only stores Tree-owned dimensions, thresholds, and ID salts.

## Layers

- `mod.rs`: public entry and stable re-exports.
- `ui.rs`: egui allocation, animation invocation, painting, and nested child clipping.
- `handler.rs`: `TreeNodeKind`, icon mapping, interaction decisions, row geometry, chevron layers, and reveal calculations with focused tests.
- `config.rs`: Tree-specific dimensions, thresholds, font sizes, and legacy animation/data ID salts.
