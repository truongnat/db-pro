use crate::tokens::{FONT_SIZE_UI_LABEL, ICON_TEXT_GAP, SPACE_LG};

pub(crate) const COMPACT_ROW_HEIGHT: f32 = 20.0;
pub(crate) const DESCRIPTION_HEIGHT: f32 = 16.0;
/// Gap between a control glyph and its label/description text column.
pub(crate) const CONTROL_SPACING: f32 = ICON_TEXT_GAP;
/// Square edge of the Checkbox/Radio glyph.
pub(crate) const CONTROL_SIZE: f32 = SPACE_LG;
/// Minimum wrap width for the label/description text column beside a control.
pub(crate) const MIN_CONTROL_TEXT_WIDTH: f32 = 60.0;
/// Label font size for selection controls (proportional family; deliberately not
/// `font_ui_label()` which renders in the ui_medium family).
pub(crate) const CONTROL_LABEL_SIZE: f32 = FONT_SIZE_UI_LABEL;
/// Description font size for selection controls (proportional family).
pub(crate) const CONTROL_DESC_SIZE: f32 = 11.5;
/// Vertical gap between a label and its description. Deliberately tighter than
/// `LABEL_HELPER_GAP` so a description reads as part of the control, not a helper row.
pub(crate) const CONTROL_DESC_GAP: f32 = 3.0;
/// Top inset applied to the control glyph and its label column when a
/// description is present, so the glyph aligns with the first text line.
pub(crate) const CONTROL_LABEL_TOP_INSET: f32 = 2.0;
/// Inset from the radio ring edge to the selected dot.
pub(crate) const RADIO_DOT_INSET: f32 = 4.0;
pub(crate) const SWITCH_WIDTH: f32 = 36.0;
pub(crate) const SWITCH_HEIGHT: f32 = 20.0;
/// Inset between the switch track edge and the knob.
pub(crate) const SWITCH_KNOB_INSET: f32 = 2.0;
/// Minimum wrap width for the switch label/description text column.
pub(crate) const SWITCH_MIN_TEXT_WIDTH: f32 = 32.0;
pub(crate) const SLIDER_HEIGHT: f32 = 20.0;
pub(crate) const SLIDER_THUMB_RADIUS: f32 = 7.0;
pub(crate) const SLIDER_FOCUS_RADIUS: f32 = 9.5;
/// Half-height of the slider track (the track is 5pt tall).
pub(crate) const TRACK_HALF_HEIGHT: f32 = 2.5;
