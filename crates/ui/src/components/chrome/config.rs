/// Small avatar edge length, in egui points.
pub(super) const AVATAR_SM: f32 = 24.0;
/// Medium avatar edge length, in egui points.
pub(super) const AVATAR_MD: f32 = 32.0;
/// Large avatar edge length, in egui points.
pub(super) const AVATAR_LG: f32 = 40.0;
/// Small avatar status dot radius, in egui points.
pub(super) const AVATAR_STATUS_DOT_SM: f32 = 3.5;
/// Medium avatar status dot radius, in egui points.
pub(super) const AVATAR_STATUS_DOT_MD: f32 = 4.5;
/// Large avatar status dot radius, in egui points.
pub(super) const AVATAR_STATUS_DOT_LG: f32 = 5.5;
/// Status dot inset multiplier relative to its radius.
pub(super) const AVATAR_STATUS_DOT_INSET_FACTOR: f32 = 0.7;
/// Status dot backing ring expansion, in egui points.
pub(super) const AVATAR_STATUS_RING_PADDING: f32 = 1.5;
/// Avatar initials font scale relative to avatar edge length.
pub(super) const AVATAR_INITIALS_FONT_SCALE: f32 = 0.38;
/// Minimum readable initials font size, in egui points.
pub(super) const AVATAR_INITIALS_MIN_FONT_SIZE: f32 = 10.0;
/// Avatar icon font scale relative to avatar edge length.
pub(super) const AVATAR_ICON_FONT_SCALE: f32 = 0.45;
/// Default skeleton placeholder width, in egui points.
pub(super) const SKELETON_DEFAULT_WIDTH: f32 = 160.0;
/// Default skeleton placeholder height, in egui points.
pub(super) const SKELETON_DEFAULT_HEIGHT: f32 = 12.0;
/// Minimum pulse alpha used by skeleton placeholders.
pub(super) const SKELETON_MIN_ALPHA: f32 = 0.35;
/// Maximum pulse alpha used by skeleton placeholders.
pub(super) const SKELETON_MAX_ALPHA: f32 = 0.72;
/// Skeleton shimmer movement speed in cycles per second.
pub(super) const SKELETON_SHIMMER_CYCLES_PER_SECOND: f64 = 0.8;
/// Skeleton shimmer band width relative to the placeholder width.
pub(super) const SKELETON_SHIMMER_WIDTH_FACTOR: f32 = 0.3;
/// Minimum skeleton shimmer band width, in egui points.
pub(super) const SKELETON_SHIMMER_MIN_WIDTH: f32 = 20.0;
/// Skeleton shimmer overlay alpha multiplier.
pub(super) const SKELETON_SHIMMER_ALPHA: f32 = 0.25;
/// Empty state top spacer, in egui points.
pub(super) const EMPTY_STATE_TOP_SPACE: f32 = 12.0;
/// Empty state icon-to-title spacer, in egui points.
pub(super) const EMPTY_STATE_ICON_GAP: f32 = 8.0;
/// Empty state title-to-description spacer, in egui points.
pub(super) const EMPTY_STATE_TITLE_GAP: f32 = 4.0;
/// Empty state action top spacer, in egui points.
pub(super) const EMPTY_STATE_ACTION_GAP: f32 = 12.0;
/// Empty state bottom spacer, in egui points.
pub(super) const EMPTY_STATE_BOTTOM_SPACE: f32 = 8.0;
/// Empty state Lucide icon font size, in egui points.
pub(super) const EMPTY_STATE_ICON_SIZE: f32 = 20.0;
/// Empty state title font size, in egui points.
pub(super) const EMPTY_STATE_TITLE_SIZE: f32 = 15.0;
/// Empty state description font size, in egui points.
pub(super) const EMPTY_STATE_DESCRIPTION_SIZE: f32 = 13.0;
/// Toolbar horizontal inner padding, in egui points.
pub(super) const TOOLBAR_MARGIN_X: f32 = 8.0;
/// Toolbar vertical inner padding, in egui points.
pub(super) const TOOLBAR_MARGIN_Y: f32 = 4.0;
/// Gap between adjacent toolbar controls, in egui points.
pub(super) const TOOLBAR_ITEM_GAP: f32 = 4.0;
