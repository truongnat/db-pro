// Border weight that keeps transaction chrome aligned with other compact UI surfaces.
pub(super) const BAR_STROKE_WIDTH: f32 = 1.0;
// Corner radius for the transaction status bar frame, in egui points.
pub(super) const BAR_RADIUS: f32 = 8.0;
// Horizontal inner padding for the status bar, in egui points.
pub(super) const BAR_MARGIN_X: f32 = 12.0;
// Vertical inner padding for the status bar, in egui points.
pub(super) const BAR_MARGIN_Y: f32 = 8.0;

// Square allocation reserved for the transaction state indicator, in egui points.
pub(super) const STATUS_DOT_SIZE: f32 = 10.0;
// Painted state indicator radius, in egui points.
pub(super) const STATUS_DOT_RADIUS: f32 = 4.0;
// Space separating the state indicator from its status label, in egui points.
pub(super) const STATUS_DOT_GAP: f32 = 4.0;
// Status label font size, in egui points.
pub(super) const STATUS_TEXT_SIZE: f32 = 12.5;
// Space before the pending-mutation badge, in egui points.
pub(super) const PENDING_BEFORE_GAP: f32 = 8.0;
// Pending-mutation badge font size, in egui points.
pub(super) const PENDING_BADGE_TEXT_SIZE: f32 = 11.0;
// Fixed badge height that keeps its label vertically aligned, in egui points.
pub(super) const PENDING_BADGE_HEIGHT: f32 = 18.0;
// Horizontal space added around the measured pending-mutation label, in egui points.
pub(super) const PENDING_BADGE_PADDING_X: f32 = 12.0;
// Badge offset from the row cursor to align it with neighboring text, in egui points.
pub(super) const PENDING_BADGE_TOP_OFFSET: f32 = 2.0;
// Horizontal inset for pending-badge text, in egui points.
pub(super) const PENDING_BADGE_TEXT_OFFSET_X: f32 = 6.0;
// Vertical inset for pending-badge text, in egui points.
pub(super) const PENDING_BADGE_TEXT_OFFSET_Y: f32 = 2.0;
// Rounded pill radius used by the pending-mutation badge, in egui points.
pub(super) const PENDING_BADGE_RADIUS: f32 = 9.0;
// Space after the badge before isolation-level text, in egui points.
pub(super) const PENDING_AFTER_GAP: f32 = 4.0;
// Space before the isolation-level label, in egui points.
pub(super) const ISOLATION_GAP: f32 = 6.0;
// Isolation-level font size, in egui points.
pub(super) const ISOLATION_TEXT_SIZE: f32 = 11.0;
// Space between adjacent transaction action buttons, in egui points.
pub(super) const ACTION_GAP: f32 = 4.0;

// Dialog width that keeps destructive confirmation text readable without dominating the editor.
pub(super) const DIALOG_WIDTH: f32 = 480.0;
// Space above the destructive warning banner, in egui points.
pub(super) const DIALOG_TOP_SPACE: f32 = 4.0;
// Corner radius of the destructive warning banner, in egui points.
pub(super) const DIALOG_BANNER_RADIUS: f32 = 8.0;
// Inner padding of the destructive warning banner, in egui points.
pub(super) const DIALOG_BANNER_MARGIN: f32 = 12.0;
// Warning banner outline width, in egui points.
pub(super) const DIALOG_BANNER_STROKE_WIDTH: f32 = 1.0;
// Warning icon font size, in egui points.
pub(super) const DIALOG_ICON_SIZE: f32 = 16.0;
// Gap between the warning icon and its explanatory text, in egui points.
pub(super) const DIALOG_ICON_GAP: f32 = 6.0;
// Destructive warning title font size, in egui points.
pub(super) const DIALOG_TITLE_SIZE: f32 = 13.0;
// Gap between the warning title and its supporting text, in egui points.
pub(super) const DIALOG_TITLE_GAP: f32 = 2.0;
// Supporting warning text font size, in egui points.
pub(super) const DIALOG_WARNING_SIZE: f32 = 12.0;
// Vertical separation between destructive confirmation sections, in egui points.
pub(super) const DIALOG_SECTION_GAP: f32 = 14.0;
// Target and instruction label font size, in egui points.
pub(super) const DIALOG_LABEL_SIZE: f32 = 12.5;
// Gap between the target label and its database object, in egui points.
pub(super) const DIALOG_TARGET_GAP: f32 = 4.0;
// Gap above the confirmation input, in egui points.
pub(super) const DIALOG_INPUT_GAP: f32 = 6.0;
// Gap between confirmation input and action row, in egui points.
pub(super) const DIALOG_ACTION_GAP: f32 = 18.0;
// Gap between destructive dialog action buttons, in egui points.
pub(super) const DIALOG_BUTTON_GAP: f32 = 8.0;
