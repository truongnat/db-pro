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
//! and are also reused by `components/select` for menu rows. The table reads
//! these heights instead of keeping a second copy. Checkbox and header radii
//! stay on the shared radius scale.

/// Shared row height, in egui points.
pub const TABLE_ROW_HEIGHT_DEFAULT: f32 = 44.0;
/// Shared compact row height, in egui points.
pub const TABLE_ROW_HEIGHT_COMPACT: f32 = 32.0;
/// Shared header height, in egui points.
pub const TABLE_HEADER_HEIGHT: f32 = 36.0;
