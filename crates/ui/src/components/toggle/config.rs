/// Shared radius for standalone toggles and the outer ends of grouped toggles.
pub const TOGGLE_ROUNDING: f32 = 6.0;
/// Gap between an icon and label when both are present.
pub const ICON_TEXT_GAP: f32 = 6.0;
/// Border width used by outline and grouped toggle controls.
pub const OUTLINE_STROKE_WIDTH: f32 = 1.0;
/// Minimum animated hover value that changes the rendered rest state.
pub const HOVER_THRESHOLD: f32 = 0.001;
/// Opacity multiplier used by the outline variant's hover fill.
pub const OUTLINE_FILL_FACTOR: f32 = 0.5;
/// Multiplier for applying horizontal padding on both sides of the content.
pub const HORIZONTAL_PADDING_FACTOR: f32 = 2.0;
/// Factor used to center measured toggle content inside its allocated rectangle.
pub const CONTENT_CENTER_FACTOR: f32 = 0.5;

/// Small toggle height, in egui points.
pub const SM_HEIGHT: f32 = 28.0;
/// Default toggle height, in egui points.
pub const DEFAULT_HEIGHT: f32 = 32.0;
/// Large toggle height, in egui points.
pub const LG_HEIGHT: f32 = 38.0;

/// Small toggle label font size, in egui points.
pub const SM_FONT_SIZE: f32 = 11.5;
/// Default toggle label font size, in egui points.
pub const DEFAULT_FONT_SIZE: f32 = 12.5;
/// Large toggle label font size, in egui points.
pub const LG_FONT_SIZE: f32 = 13.5;

/// Small toggle icon size, in egui points.
pub const SM_ICON_SIZE: f32 = 13.0;
/// Default toggle icon size, in egui points.
pub const DEFAULT_ICON_SIZE: f32 = 14.5;
/// Large toggle icon size, in egui points.
pub const LG_ICON_SIZE: f32 = 16.0;

/// Small toggle horizontal padding, in egui points.
pub const SM_PADDING_X: f32 = 8.0;
/// Default toggle horizontal padding, in egui points.
pub const DEFAULT_PADDING_X: f32 = 11.0;
/// Large toggle horizontal padding, in egui points.
pub const LG_PADDING_X: f32 = 14.0;
