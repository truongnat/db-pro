use crate::tokens::component::button::{
    BUTTON_DEFAULT_PADDING, BUTTON_DEFAULT_WIDTH, BUTTON_FONT_SIZE_DEFAULT, BUTTON_FONT_SIZE_LG, BUTTON_FONT_SIZE_SM,
    BUTTON_HEIGHT_DEFAULT, BUTTON_HEIGHT_LG, BUTTON_HEIGHT_SM, BUTTON_ICON_PADDING, BUTTON_ICON_SIZE_DEFAULT_GLYPH,
    BUTTON_ICON_SIZE_GLYPH, BUTTON_ICON_SIZE_LG_GLYPH, BUTTON_ICON_SIZE_SM_GLYPH, BUTTON_ICON_SM_FONT_SIZE,
    BUTTON_ICON_SM_PADDING, BUTTON_LG_PADDING, BUTTON_LG_WIDTH, BUTTON_SM_PADDING,
};
use egui::Vec2;

/// Button corner radius, in egui points, preserving the existing compact native shape.
pub const BUTTON_ROUNDING: f32 = crate::tokens::RADIUS_BUTTON;
/// Gap, in egui points, between a leading icon/spinner and visible label text.
pub const ICON_TEXT_GAP: f32 = crate::tokens::ICON_TEXT_GAP;
/// Compact gap between adjacent buttons in a grouped control, in egui points.
pub const BUTTON_GROUP_ITEM_GAP: f32 = 1.0;
/// Link-variant underline thickness in egui points, shown while hovered or focused.
pub const LINK_UNDERLINE_WIDTH: f32 = 1.0;
/// Small button minimum height, in egui points.
pub const SM_MIN_HEIGHT: f32 = BUTTON_HEIGHT_SM;
/// Small button label font size, in egui points.
pub const SM_FONT_SIZE: f32 = BUTTON_FONT_SIZE_SM;
/// Small button icon size, in egui points.
pub const SM_ICON_SIZE: f32 = BUTTON_ICON_SIZE_SM_GLYPH;
/// Small button default square width, in egui points.
pub const SM_DEFAULT_WIDTH: f32 = BUTTON_HEIGHT_SM;
/// Default button minimum height, in egui points.
pub const DEFAULT_MIN_HEIGHT: f32 = BUTTON_HEIGHT_DEFAULT;
/// Default button label font size, in egui points.
pub const DEFAULT_FONT_SIZE: f32 = BUTTON_FONT_SIZE_DEFAULT;
/// Default button icon size, in egui points.
pub const DEFAULT_ICON_SIZE: f32 = BUTTON_ICON_SIZE_DEFAULT_GLYPH;
/// Default button square fallback width, in egui points.
pub const DEFAULT_WIDTH: f32 = BUTTON_DEFAULT_WIDTH;
/// Large button minimum height, in egui points.
pub const LG_MIN_HEIGHT: f32 = BUTTON_HEIGHT_LG;
/// Large button label font size, in egui points.
pub const LG_FONT_SIZE: f32 = BUTTON_FONT_SIZE_LG;
/// Large button icon size, in egui points.
pub const LG_ICON_SIZE: f32 = BUTTON_ICON_SIZE_LG_GLYPH;
/// Large button square fallback width, in egui points.
pub const LG_DEFAULT_WIDTH: f32 = BUTTON_LG_WIDTH;
/// Icon button icon size, in egui points.
pub const ICON_SIZE: f32 = BUTTON_ICON_SIZE_GLYPH;
/// Small icon button minimum height and fallback width, in egui points.
pub const ICON_SM_DEFAULT_WIDTH: f32 = 26.0;
/// Small icon button label font size, in egui points, used only if callers add text.
pub const ICON_SM_FONT_SIZE: f32 = BUTTON_ICON_SM_FONT_SIZE;
/// Small icon button icon size, in egui points.
pub const ICON_SM_SIZE: f32 = BUTTON_ICON_SIZE_SM_GLYPH;

/// Small button padding, in egui points.
pub const SM_PADDING: Vec2 = BUTTON_SM_PADDING;
/// Default button padding, in egui points.
pub const DEFAULT_PADDING: Vec2 = BUTTON_DEFAULT_PADDING;
/// Large button padding, in egui points.
pub const LG_PADDING: Vec2 = BUTTON_LG_PADDING;
/// Icon button padding, in egui points.
pub const ICON_PADDING: Vec2 = BUTTON_ICON_PADDING;
/// Small icon button padding, in egui points.
pub const ICON_SM_PADDING: Vec2 = BUTTON_ICON_SM_PADDING;
