//! Table/Grid component contract.
//!
//! States supported by rows: default, hover, selected; headers add hover.
//! `selected` outranks `hover` (row hover animation is suppressed while a row is
//! selected — `components/table/ui.rs`). Rows have no disabled/loading states;
//! grid-level busy states are owned by the calling view.
//!
//! Semantic roles consumed (via `DbProTheme`): `background.hover/selected` for row
//! washes, `border.subtle/default` for dividers, `foreground.primary/secondary/
//! muted` for cell text, `accent.solid` for selection affordances.
//!
//! Owned values: the shared row/header heights below, which serve compact grids
//! and are also reused by `components/select` for menu rows. The table's own
//! coordinate system (checkbox column, header height, row height 44pt, padding)
//! intentionally stays in `components/table/config.rs` as implementation
//! geometry — it is not part of the shared scale and is pinned by its own test.

/// Shared compact row height, in egui points.
pub const TABLE_ROW_HEIGHT_DEFAULT: f32 = 40.0;
/// Shared compact row height, in egui points.
pub const TABLE_ROW_HEIGHT_COMPACT: f32 = 32.0;
/// Shared header height, in egui points.
pub const TABLE_HEADER_HEIGHT: f32 = 36.0;
