/// Smallest usable text field width. Fields inside narrow containers (clamped dialogs,
/// sheets, split panes) may shrink to this but no further.
pub const INPUT_MIN_WIDTH: f32 = 120.0;
pub const TEXTAREA_INNER_MARGIN_Y: f32 = 6.0;
pub const SEARCH_SHORTCUT_RESERVATION: f32 = 36.0;
pub const SEARCH_CLEAR_RESERVATION: f32 = crate::tokens::component::button::BUTTON_ICON_SIZE_SM.x;
/// Width reserved for the password visibility toggle plus its gap, matching the
/// small icon-button slot used by the search clear button.
pub const PASSWORD_EYE_RESERVATION: f32 = crate::tokens::component::button::BUTTON_ICON_SIZE_SM.x;
/// Square hit target for the clearable text field's clear button, in egui points.
pub const CLEAR_BUTTON_HIT: f32 = 24.0;
/// Smallest frame width a single-line input may shrink to, in egui points.
pub const MIN_FRAME_WIDTH: f32 = 60.0;
/// Smallest editable text width inside the frame, in egui points.
pub const MIN_EDIT_WIDTH: f32 = 40.0;
pub const INPUT_LABEL_FONT_SIZE: f32 = crate::tokens::FONT_SIZE_CAPTION;
pub const INPUT_ICON_SIZE: f32 = crate::tokens::ICON_SM;
pub const INPUT_AUX_FONT_SIZE: f32 = crate::tokens::FONT_SIZE_BADGE;
