// Shared corner radius used by modal dialog frames, in egui points.
pub const DIALOG_RADIUS: f32 = crate::tokens::RADIUS_DIALOG;
// Default dialog width before viewport constraints are applied, in egui points.
pub const DIALOG_WIDTH: f32 = 420.0;
// Preset width for preview-style dialogs that render a code/SQL block —
// wide enough that multi-line DDL stays readable without horizontal scroll (#334/#345).
pub const DIALOG_WIDTH_PREVIEW: f32 = 640.0;
// Minimum horizontal breathing room between a dialog and the viewport edge, in egui points.
pub const DIALOG_HORIZONTAL_MARGIN: f32 = crate::tokens::SPACE_LG;
// Vertical spacing used when centering a dialog within the viewport, in egui points.
pub const DIALOG_VERTICAL_MARGIN: f32 = crate::tokens::SPACE_2XL;
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

/// Pinned body box for long forms, in egui points.
///
/// A form dialog left content-sized grows every time a validation line or an
/// asynchronously loaded section appears, and the card slides upward each time
/// it re-centers. Pinning the box gives the dialog one stable size and lets the
/// body scroll instead.
pub const DIALOG_FORM_BODY_HEIGHT: f32 = 560.0;

// Shared dialog content spacing, in egui points.
pub const DIALOG_CONTENT_HORIZONTAL_PADDING: f32 = crate::tokens::SPACE_2XL;
pub const DIALOG_CONTENT_VERTICAL_PADDING: f32 = crate::tokens::SPACE_XL;
pub const DIALOG_SECTION_SPACING: f32 = crate::tokens::CONTROL_GROUP_GAP;
pub const DIALOG_FOOTER_SPACING: f32 = crate::tokens::SPACE_SM;
// The total horizontal padding on both sides of the dialog content.
pub const DIALOG_TOTAL_HORIZONTAL_PADDING: f32 = 48.0;
pub const DIALOG_CONTENT_MIN_SIZE: f32 = 80.0;
pub const DIALOG_HEADER_MIN_HEIGHT: f32 = 40.0;

// Sheet layout values, in egui points.
pub const SHEET_PADDING: f32 = crate::tokens::SPACE_LG;
pub const SHEET_TOTAL_PADDING: f32 = crate::tokens::SPACE_3XL;
pub const SHEET_CORNER_RADIUS: f32 = crate::tokens::RADIUS_DIALOG;
pub const SHEET_CONTENT_MIN_SIZE: f32 = 80.0;
pub const SHEET_CLOSE_WIDTH: f32 = 28.0;
pub const SHEET_TITLE_GAP: f32 = crate::tokens::SPACE_SM;
