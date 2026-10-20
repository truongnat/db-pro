// Layout constants for the welcome surface. Named intent only — spacing values
// come from the shared scale (crate::tokens), not local pixel picks.

/// Vertical anchor for the hero block, capped by the surface's compact spacing.
pub(super) const HEADER_TOP_RATIO: f32 = 0.35;

/// Fixed height of each quick-action card. Chosen once for scanability: two
/// text lines (title + caption) with icon plus padding fit comfortably.
pub(super) const CARD_HEIGHT: f32 = 64.0;

/// Minimum readable card width before the grid switches to one stacked column.
pub(super) const MIN_CARD_WIDTH: f32 = 280.0;

/// Width at which the 2-column grid becomes viable: two minimum cards plus the
/// gap between them. Derived, not tuned per-eyeball.
pub(super) const WIDE_GRID_MIN_WIDTH: f32 = 2.0 * MIN_CARD_WIDTH + crate::tokens::SPACE_MD;

/// Total width the 2-column grid caps at — wider viewports keep the grid
/// centered instead of stretching cards into banners.
pub(super) const WIDE_GRID_WIDTH: f32 = 620.0;

/// Quick-action intents the welcome cards emit; the workspace shell maps each
/// to a navigation/mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WelcomeAction {
    NewConnection,
    NewQuery,
    ToggleAgent,
    OpenDiagram,
}
