use crate::tokens::component::input::INPUT_HEIGHT_DEFAULT;

/// Maximum number of option rows shown before the menu must scroll.
pub const MAX_VISIBLE_ITEMS: usize = 8;
/// Minimum trigger width, in egui points, set to twice the standard input height for usable controls.
pub const MIN_TRIGGER_WIDTH: f32 = INPUT_HEIGHT_DEFAULT * 2.0;
/// Minimum space for trigger text, in egui points, allowing roughly two small body-font ems.
pub const MIN_TRIGGER_TEXT_WIDTH: f32 = crate::tokens::FONT_SIZE_BODY_SM * 2.0;
/// Trigger width reserved for trailing icon and its text gap, in egui points.
pub const TRIGGER_TEXT_RESERVED_WIDTH: f32 = crate::tokens::ICON_SM + crate::tokens::ICON_TEXT_GAP;
/// Smallest popup width allowed after screen constraints, in egui points, set to two input heights.
pub const MENU_MIN_SCREEN_WIDTH: f32 = INPUT_HEIGHT_DEFAULT * 2.0;
/// Side length of the optional per-row icon painted before a menu label.
pub const OPTION_ICON_SIDE: f32 = 16.0;
/// Multiplier placing half a measured text height above its vertical center.
pub const VERTICAL_CENTER_FACTOR: f32 = 0.5;
/// Multiplier for symmetric dimensions such as two-sided padding and screen insets.
pub const DOUBLE_FACTOR: f32 = 2.0;
