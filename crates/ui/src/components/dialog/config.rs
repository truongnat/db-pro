pub const DIALOG_RADIUS: f32 = 16.0;
pub const DIALOG_WIDTH: f32 = 420.0;
pub const DIALOG_HORIZONTAL_MARGIN: f32 = 16.0;
pub const DIALOG_VERTICAL_MARGIN: f32 = 24.0;
pub const SHEET_WIDTH: f32 = 360.0;
pub const DIALOG_TRANSLATE_PX: f32 = 8.0;
pub const SHEET_TRANSLATE_PX: f32 = 16.0;

/// Vertical space reserved for the dialog frame outside its scrollable body.
///
/// This covers the card margins, header, separator, and sticky footer. Keeping
/// it explicit prevents tall forms from consuming the viewport and making the
/// dialog look top-aligned instead of centered.
pub const DIALOG_CHROME_HEIGHT: f32 = 220.0;
