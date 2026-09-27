// Vertical pitch reserved for each database-tree entry, in egui points.
pub(crate) const ROW_HEIGHT: f32 = 26.0;
// Horizontal offset added at each nested tree depth, in egui points.
pub(crate) const INDENT_STEP: f32 = 14.0;
// Leading inset before tree-row controls, in egui points.
pub(crate) const LEFT_INSET: f32 = 8.0;
// Width reserved for an expand/collapse chevron, in egui points.
pub(crate) const CHEVRON_SLOT: f32 = 14.0;
// Offset applied when centering the row icon in its slot, in egui points.
pub(crate) const ICON_OFFSET: f32 = 7.0;
// Width reserved for a database-object icon, in egui points.
pub(crate) const ICON_SLOT: f32 = 17.0;
// Inset before secondary row details, in egui points.
pub(crate) const DETAIL_INSET: f32 = 10.0;
// Content height used when the tree has no measured child rows, in egui points.
pub(crate) const DEFAULT_CONTENT_HEIGHT: f32 = 48.0;
// Smallest positive clip height used to keep egui clipping geometry valid, in egui points.
pub(crate) const MIN_CLIP_HEIGHT: f32 = 1.0;
