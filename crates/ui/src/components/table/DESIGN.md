# Table design

`mod.rs` defines public column/table builders and keeps their exports stable. `ui.rs` owns header/row painting, egui interactions, and caller callbacks. `handler.rs` derives column geometry and select-all state; `config.rs` holds table-owned dimensions; `tests.rs` covers public behavior and geometry.

## Event and data flow

The caller supplies row count, a selected-state query, selection/sort callbacks, and a cell-render callback. Headers report sort intent but do not sort data. The selection header reports whether to toggle all; each row reports its index. The widget reads selected state through the callback and does not keep a copy of the dataset or selection.

Column widths are resolved against available width and optional fixed widths. A scrollable table body uses row geometry to determine visible cells; each row's caller renderer runs only when its row rectangle is visible. The loop still visits row indices to check viewport visibility, so CPU work includes O(total rows) visibility checks per frame while painting/cell work is limited to visible rows. Line-of-sight rendering therefore needs caller pagination for extremely large datasets.

## Layout, accessibility, and states

The table has an empty-state message when row count is zero, sortable headers, optional row selection, an indeterminate select-all state, and optional vertical grid lines. Row height is caller-configurable. Shared semantic colors/strokes come from theme/tokens. Narrow widths may require horizontal scrolling; right-aligned numbers and fixed columns retain their alignment. Sorting and selection semantics are communicated in the visible controls, while actual data updates remain caller-owned.
