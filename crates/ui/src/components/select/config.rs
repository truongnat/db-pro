use crate::tokens::component::input::INPUT_HEIGHT_DEFAULT;

/// Maximum number of option rows shown before the menu must scroll.
pub const MAX_VISIBLE_ITEMS: usize = 8;
/// Minimum trigger width, in egui points, set to twice the standard input height for usable controls.
pub const MIN_TRIGGER_WIDTH: f32 = INPUT_HEIGHT_DEFAULT * 2.0;
/// Minimum space for trigger text, in egui points, allowing roughly two small body-font ems.
pub const MIN_TRIGGER_TEXT_WIDTH: f32 = crate::tokens::FONT_SIZE_BODY_SM * 2.0;
/// Trigger width reserved for trailing icon and its text gap, in egui points.
pub const TRIGGER_TEXT_RESERVED_WIDTH: f32 = crate::tokens::ICON_SM + crate::tokens::ICON_TEXT_GAP;
/// Minimum popup width, in egui points, set to four standard input heights for readable options.
pub const MENU_MIN_WIDTH: f32 = INPUT_HEIGHT_DEFAULT * 4.0;
/// Smallest popup width allowed after screen constraints, in egui points, set to two input heights.
pub const MENU_MIN_SCREEN_WIDTH: f32 = INPUT_HEIGHT_DEFAULT * 2.0;
/// Multiplier placing half a measured text height above its vertical center.
pub const VERTICAL_CENTER_FACTOR: f32 = 0.5;
/// Multiplier for symmetric dimensions such as two-sided padding and screen insets.
pub const DOUBLE_FACTOR: f32 = 2.0;
