// Layout constants for the welcome surface. Named intent only — spacing values
// come from the shared scale (crate::tokens), not local pixel picks.

/// Vertical anchor for the hero block: the header lands at ~35% of the
/// available height instead of optical center, matching product landing pages.
pub(super) const HEADER_TOP_RATIO: f32 = 0.35;

/// Fixed height of each quick-action card. Chosen once for scanability: two
/// text lines (title + caption) with icon plus padding fit comfortably.
pub(super) const CARD_HEIGHT: f32 = 64.0;

/// Minimum width one card needs to stay readable; below it the grid collapses
/// to a single stacked column. Also doubles as the container floor on very
/// narrow viewports.
pub(super) const MIN_CARD_WIDTH: f32 = 280.0;

/// Width at which the 2-column grid becomes viable: two minimum cards plus the
/// gap between them. Derived, not tuned per-eyeball.
pub(super) const WIDE_GRID_MIN_WIDTH: f32 = 2.0 * MIN_CARD_WIDTH + crate::tokens::SPACE_MD;

/// Total width the 2-column grid caps at — wider viewports keep the grid
/// centered instead of stretching cards into banners.
pub(super) const WIDE_GRID_WIDTH: f32 = 620.0;

/// Lower bound for the description wrap width so degenerate narrow cards can
/// never request a zero-width layout.
pub(super) const MIN_DESC_WRAP_WIDTH: f32 = 10.0;
