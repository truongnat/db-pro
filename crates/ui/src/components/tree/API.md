# Tree API

The tree API is re-exported from `components::tree` and `components`.

- `TreeNodeKind` describes `Server`, `Database`, `Schema`, `Table`, `View`, `Column`, `PrimaryKey`, `ForeignKey`, or `Index`; `icon()` returns the matching Lucide icon.
- `DatabaseTreeNode::new(name, kind, depth, theme)` creates a row. `detail(text)` adds right-aligned metadata; `selected(bool)` sets selected visuals/accessibility state; `expanded(&mut bool)` enables the chevron and toggles the borrowed value when clicked; `show(ui) -> Response` returns row interaction for caller handling.
- `reveal_children(ui, stable_id, open, closure)` invokes and clips nested content while the shared animation is visible. Use a stable unique ID for each parent. Child layout is measured and remembered across frames; an initial estimated height is used until measured.

Rows are not a data model and do not fetch children. The caller owns hierarchy, loading/empty states, selection mutation, and lazy loading. There is no built-in depth/node cap or virtualization; constrain large trees in the caller. Deep indentation and long row labels/details need sufficient viewport width.
