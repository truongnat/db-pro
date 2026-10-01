// Fixed height of the search input row, in egui points.
pub(super) const INPUT_HEIGHT: f32 = 44.0;
// Horizontal inset for the search glyph, in egui points.
pub(super) const INPUT_ICON_INSET_X: f32 = 14.0;
// Search glyph font size, in egui points.
pub(super) const INPUT_ICON_SIZE: f32 = 16.0;
// Gap between the search glyph and editable text, in egui points.
pub(super) const INPUT_TEXT_GAP: f32 = 24.0;
// Trailing inset that leaves room for the input's right-side affordance, in egui points.
pub(super) const INPUT_TEXT_RIGHT_INSET: f32 = 32.0;
// Vertical inset around the text-edit region, in egui points.
pub(super) const INPUT_TEXT_INSET_Y: f32 = 8.0;
// Search input text font size, in egui points.
pub(super) const INPUT_FONT_SIZE: f32 = 13.5;
// Bottom separator inset that centers a one-point line on the allocated edge.
pub(super) const INPUT_SEPARATOR_INSET: f32 = 0.5;

// Fixed height of each command result row, in egui points.
pub(super) const ITEM_HEIGHT: f32 = 36.0;
// Leading inset for command-row content, in egui points.
pub(super) const ITEM_LEFT_INSET: f32 = 10.0;
// Corner radius of selected and hovered command rows, in egui points.
pub(super) const ITEM_ROUNDING: f32 = 6.0;
// Icon font size in a command row, in egui points.
pub(super) const ITEM_ICON_SIZE: f32 = 14.0;
// Horizontal advance reserved after an item icon, in egui points.
pub(super) const ITEM_ICON_ADVANCE: f32 = 22.0;
// Main command title font size, in egui points.
pub(super) const ITEM_TITLE_SIZE: f32 = 13.0;
// Subtitle font size, in egui points.
pub(super) const ITEM_SUBTITLE_SIZE: f32 = 11.5;
// Gap between measured title text and its subtitle, in egui points.
pub(super) const ITEM_SUBTITLE_GAP: f32 = 8.0;
// Hover animation amount above which the row background is visible.
pub(super) const ITEM_HOVER_VISIBLE_THRESHOLD: f32 = 0.001;
// Reserved shortcut area width measured from the row's trailing edge, in egui points.
pub(super) const SHORTCUT_REGION_WIDTH: f32 = 80.0;
// Trailing inset of the shortcut region, in egui points.
pub(super) const SHORTCUT_RIGHT_INSET: f32 = 8.0;
// Vertical inset of the shortcut badge region, in egui points.
pub(super) const SHORTCUT_INSET_Y: f32 = 6.0;

// Space before a command group heading, in egui points.
pub(super) const GROUP_TOP_GAP: f32 = 6.0;
// Command group heading font size, in egui points.
pub(super) const GROUP_HEADING_SIZE: f32 = 11.0;
// Space between a command group heading and its contents, in egui points.
pub(super) const GROUP_CONTENT_GAP: f32 = 2.0;
// Empty-state top and bottom breathing room, in egui points.
pub(super) const EMPTY_STATE_VERTICAL_SPACE: f32 = 24.0;
// Empty-state message font size, in egui points.
pub(super) const EMPTY_STATE_FONT_SIZE: f32 = 13.0;

// Opacity used for an unselected row while its hover animation is settling.
pub(super) const ITEM_DEFAULT_TITLE_ALPHA: f32 = 0.9;
