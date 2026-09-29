/// Vertical pitch reserved for each database-tree entry, in egui points.
pub(crate) const ROW_HEIGHT: f32 = 26.0;
/// Horizontal offset added at each nested tree depth, in egui points.
pub(crate) const INDENT_STEP: f32 = 14.0;
/// Leading inset before tree-row controls, in egui points.
pub(crate) const LEFT_INSET: f32 = 8.0;
/// Width reserved for an expand/collapse chevron, in egui points.
pub(crate) const CHEVRON_SLOT: f32 = 14.0;
/// Horizontal center used by chevron and node-icon slots, in egui points.
pub(crate) const CONTROL_CENTER_OFFSET: f32 = 7.0;
/// Legacy nudge that visually centers the chevron glyph in the narrower chevron slot.
pub(crate) const CHEVRON_HORIZONTAL_NUDGE: f32 = -1.0;
/// Width reserved for a database-object icon, in egui points.
pub(crate) const ICON_SLOT: f32 = 17.0;
/// Inset before secondary row details, in egui points.
pub(crate) const DETAIL_INSET: f32 = 10.0;
/// Corner radius for the selected and hovered row background, in egui points.
pub(crate) const ROW_ROUNDING: f32 = 4.0;
/// Width of the selected-row accent rail, in egui points.
pub(crate) const SELECTED_ACCENT_WIDTH: f32 = 2.5;
/// Corner radius for the selected-row accent rail, in egui points.
pub(crate) const SELECTED_ACCENT_ROUNDING: f32 = 1.0;
/// Hover opacity values at or below this threshold are visually skipped to avoid redundant painting.
pub(crate) const HOVER_VISIBLE_THRESHOLD: f32 = 0.001;
/// Lucide font size used for database-object icons.
pub(crate) const NODE_ICON_FONT_SIZE: f32 = 13.0;
/// Lucide font size used for expand/collapse chevrons.
pub(crate) const CHEVRON_ICON_FONT_SIZE: f32 = 10.5;
/// Primary row label font size, in egui points.
pub(crate) const LABEL_FONT_SIZE: f32 = 12.5;
/// Secondary detail label font size, in egui points.
pub(crate) const DETAIL_FONT_SIZE: f32 = 11.0;
/// Collapsed chevron fades out once expansion animation reaches this progress.
pub(crate) const CHEVRON_COLLAPSED_HIDE_PROGRESS: f32 = 0.99;
/// Expanded chevron fades in after this expansion progress.
pub(crate) const CHEVRON_EXPANDED_SHOW_PROGRESS: f32 = 0.01;
/// Maximum alpha channel value used when crossfading chevron glyphs.
pub(crate) const ALPHA_CHANNEL_MAX: f32 = 255.0;
/// Progress values at or below this threshold keep nested rows unallocated.
pub(crate) const REVEAL_VISIBLE_THRESHOLD: f32 = 0.01;
/// Content height used when the tree has no measured child rows, in egui points.
pub(crate) const DEFAULT_CONTENT_HEIGHT: f32 = 48.0;
/// Smallest positive clip height used to keep egui clipping geometry valid, in egui points.
pub(crate) const MIN_CLIP_HEIGHT: f32 = 1.0;
/// Animation state salt for hover fade; value is legacy state and must stay stable.
pub(crate) const HOVER_ANIMATION_ID_SALT: &str = "hover";
/// Animation state salt for chevron rotation/crossfade; value is legacy state and must stay stable.
pub(crate) const CHEVRON_ANIMATION_ID_SALT: &str = "chev_anim";
/// Temporary egui data salt for measured child height; value is legacy state and must stay stable.
pub(crate) const CHILDREN_HEIGHT_ID_SALT: &str = "content_h";
