use egui::Vec2;

/// Button corner radius, in egui points, preserving the existing compact native shape.
pub const BUTTON_ROUNDING: f32 = 4.0;
/// Gap, in egui points, between a leading icon/spinner and visible label text.
pub const ICON_TEXT_GAP: f32 = 6.0;
/// Compact gap between adjacent buttons in a grouped control, in egui points.
pub const BUTTON_GROUP_ITEM_GAP: f32 = 1.0;
/// Link-variant underline thickness in egui points, shown while hovered or focused.
pub const LINK_UNDERLINE_WIDTH: f32 = 1.0;
/// Small button minimum height, in egui points.
pub const SM_MIN_HEIGHT: f32 = 28.0;
/// Small button label font size, in egui points.
pub const SM_FONT_SIZE: f32 = 11.5;
/// Small button icon size, in egui points.
pub const SM_ICON_SIZE: f32 = 13.0;
/// Small button default square width, in egui points.
pub const SM_DEFAULT_WIDTH: f32 = 28.0;
/// Default button minimum height, in egui points.
pub const DEFAULT_MIN_HEIGHT: f32 = 32.0;
/// Default button label font size, in egui points.
pub const DEFAULT_FONT_SIZE: f32 = 12.5;
/// Default button icon size, in egui points.
pub const DEFAULT_ICON_SIZE: f32 = 14.5;
/// Default button square fallback width, in egui points.
pub const DEFAULT_WIDTH: f32 = 32.0;
/// Large button minimum height, in egui points.
pub const LG_MIN_HEIGHT: f32 = 38.0;
/// Large button label font size, in egui points.
pub const LG_FONT_SIZE: f32 = 13.5;
/// Large button icon size, in egui points.
pub const LG_ICON_SIZE: f32 = 16.0;
/// Large button square fallback width, in egui points.
pub const LG_DEFAULT_WIDTH: f32 = 38.0;
/// Icon button icon size, in egui points.
pub const ICON_SIZE: f32 = 15.0;
/// Small icon button minimum height and fallback width, in egui points.
pub const ICON_SM_DEFAULT_WIDTH: f32 = 26.0;
/// Small icon button label font size, in egui points, used only if callers add text.
pub const ICON_SM_FONT_SIZE: f32 = 11.0;
/// Small icon button icon size, in egui points.
pub const ICON_SM_SIZE: f32 = 13.5;

/// Small button padding, in egui points.
pub const SM_PADDING: Vec2 = Vec2::new(10.0, 4.0);
/// Default button padding, in egui points.
pub const DEFAULT_PADDING: Vec2 = Vec2::new(12.0, 5.0);
/// Large button padding, in egui points.
pub const LG_PADDING: Vec2 = Vec2::new(16.0, 8.0);
/// Icon button padding, in egui points.
pub const ICON_PADDING: Vec2 = Vec2::new(8.0, 8.0);
/// Small icon button padding, in egui points.
pub const ICON_SM_PADDING: Vec2 = Vec2::new(6.0, 6.0);
