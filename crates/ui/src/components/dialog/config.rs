// Shared corner radius used by modal dialog frames, in egui points.
pub const DIALOG_RADIUS: f32 = 16.0;
// Default dialog width before viewport constraints are applied, in egui points.
pub const DIALOG_WIDTH: f32 = 420.0;
// Minimum horizontal breathing room between a dialog and the viewport edge, in egui points.
pub const DIALOG_HORIZONTAL_MARGIN: f32 = 16.0;
// Vertical spacing used when centering a dialog within the viewport, in egui points.
pub const DIALOG_VERTICAL_MARGIN: f32 = 24.0;
// Default width of the right-side sheet before viewport constraints, in egui points.
pub const SHEET_WIDTH: f32 = 360.0;
// Maximum vertical offset used by the dialog's opening/closing motion, in egui points.
pub const DIALOG_TRANSLATE_PX: f32 = 8.0;
// Maximum horizontal offset used by the sheet's slide motion, in egui points.
pub const SHEET_TRANSLATE_PX: f32 = 16.0;

/// Vertical space reserved for the dialog frame outside its scrollable body.
///
/// This covers the card margins, header, separator, and sticky footer. Keeping
/// it explicit prevents tall forms from consuming the viewport and making the
/// dialog look top-aligned instead of centered.
pub const DIALOG_CHROME_HEIGHT: f32 = 220.0;
