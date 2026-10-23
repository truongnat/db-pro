// Layout constants for the welcome surface. Named intent only — spacing values
// come from the shared scale (crate::tokens), not local pixel picks.

/// Vertical anchor for the hero block, capped by the surface's compact spacing.
pub(super) const HEADER_TOP_RATIO: f32 = 0.35;

/// Fixed height of each quick-action card. Chosen once for scanability: two
/// text lines (title + caption) with icon plus padding fit comfortably.
pub(super) const CARD_HEIGHT: f32 = 64.0;

/// Minimum readable card width before the grid switches to one stacked column.
pub(super) const MIN_CARD_WIDTH: f32 = 280.0;

/// Widest a single card grows before the grid adds a column instead. A card's
/// text starts at `SPACE_MD + ICON_DEFAULT + SPACE_SM` and ends `SPACE_MD`
/// before its edge, so this keeps the caption near a 60-character measure
/// while the grid still spends the surface's width.
pub(super) const CARD_MAX_WIDTH: f32 = 420.0;

/// Width at which the 2-column grid becomes viable: two minimum cards plus the
/// gap between them. Derived, not tuned per-eyeball.
pub(super) const WIDE_GRID_MIN_WIDTH: f32 = 2.0 * MIN_CARD_WIDTH + crate::tokens::SPACE_MD;

/// Width at which all four quick actions fit on one row without any card
/// dropping below its minimum readable width.
pub(super) const QUAD_GRID_MIN_WIDTH: f32 = 4.0 * MIN_CARD_WIDTH + 3.0 * crate::tokens::SPACE_MD;

/// Quick-action intents the welcome cards emit; the workspace shell maps each
/// to a navigation/mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WelcomeAction {
    NewConnection,
    NewQuery,
    ToggleAgent,
    OpenDiagram,
}
