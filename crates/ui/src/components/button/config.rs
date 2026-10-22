//! Values owned by Button. Shared sizes, spacing and radius come from `crate::tokens`.

/// Link underline thickness, in egui points.
pub const LINK_UNDERLINE_WIDTH: f32 = crate::tokens::STROKE_THIN;

/// Inset subtracted from the button icon size to size the loading spinner diameter, in egui points.
pub const SPINNER_INSET: f32 = 2.0;
/// Stroke width of the loading spinner arc, in egui points.
pub const SPINNER_STROKE_WIDTH: f32 = 1.8;
