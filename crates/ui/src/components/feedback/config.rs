pub const PROGRESS_DEFAULT_HEIGHT: f32 = 6.0;
pub const PROGRESS_MIN_BEAM_WIDTH: f32 = 12.0;
pub(super) const PROGRESS_STATIC_BEAM_TAIL: f32 = 0.25;
pub(super) const PROGRESS_STATIC_BEAM_HEAD: f32 = 0.75;

pub const SPINNER_DEFAULT_SIZE: f32 = 18.0;
pub const SPINNER_MIN_SIZE: f32 = 5.0; // Safe allocation floor.
pub const SPINNER_MIN_RADIUS: f32 = 1.0;
pub const SPINNER_RADIUS_OFFSET: f32 = 3.0;

pub const KBD_FONT_SIZE: f32 = crate::tokens::FONT_SIZE_KBD;
pub const KBD_PAD_X: f32 = 6.0;
pub const KBD_PAD_Y: f32 = 2.5;
pub const KBD_COMBO_GAP: f32 = 3.0;
pub const KBD_PLUS_SIZE: f32 = 10.0;

pub const KBD_SEPARATOR_TEXT_SIZE: f32 = crate::tokens::FONT_SIZE_KBD;
pub const KBD_SEPARATOR_HEIGHT: f32 = 14.0;
pub const KBD_SEPARATOR_MIN_LINE_WIDTH: f32 = 10.0;
pub const KBD_SEPARATOR_PADDING_EXTRA: f32 = crate::tokens::SPACE_LG;

/// Allocated square size of the status dot hit area, in egui points.
pub const STATUS_DOT_SIZE: f32 = 12.0;
/// Radius of the solid core circle, in egui points.
pub const STATUS_DOT_CORE_RADIUS: f32 = 4.0;
/// Inner radius where the pulse ring starts, in egui points.
pub const STATUS_DOT_PULSE_MIN_RADIUS: f32 = 3.0;
/// Outer radius the pulse ring expands to over one cycle, in egui points.
pub const STATUS_DOT_PULSE_MAX_RADIUS: f32 = 6.0;
/// Pulse ring alpha at the start of the cycle (fades to 0).
pub const STATUS_DOT_PULSE_MAX_ALPHA: f32 = 120.0;
/// Duration of one pulse cycle, in seconds.
pub const STATUS_DOT_PULSE_SECS: f32 = 1.6;
