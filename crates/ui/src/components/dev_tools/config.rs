/// Height of the macOS-style top header bar in egui points.
pub const TERMINAL_HEADER_HEIGHT: f32 = 28.0;
/// Radius of the window control indicator dots (close, minimize, zoom).
pub const TERMINAL_DOT_RADIUS: f32 = 3.5;
/// Left inset for the first window control dot in egui points.
pub const TERMINAL_DOT_START_X: f32 = 12.0;
/// Horizontal spacing between window control dot centers in egui points.
pub const TERMINAL_DOT_SPACING: f32 = 12.0;
/// Left title inset to keep the decorative control dots clear.
pub const TERMINAL_TITLE_LEFT_INSET: f32 = 58.0;
/// Right title inset preserving balance with the left controls.
pub const TERMINAL_TITLE_RIGHT_INSET: f32 = 12.0;

/// Default stroke width for the circular progress ring track and arc.
pub const PROGRESS_RING_TRACK_WIDTH: f32 = 2.5;
/// Number of line segments used to approximate the smooth circular progress arc.
pub const PROGRESS_RING_ARC_POINTS: usize = 32;
/// Inset from the outer allocated boundary to the center of the ring stroke.
pub const PROGRESS_RING_INSET: f32 = 2.0;
/// Default radius used when a caller supplies a non-finite progress-ring radius.
pub const PROGRESS_RING_DEFAULT_RADIUS: f32 = 18.0;
/// Smallest supported progress-ring radius in egui points.
pub const PROGRESS_RING_MIN_RADIUS: f32 = 4.0;
