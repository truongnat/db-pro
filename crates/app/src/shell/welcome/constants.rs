// Layout constants for the welcome surface. Kept in a dedicated module so the
// view reads named intent instead of bare numbers.

/// Fraction of the leftover height (after reserving the content block) used as
/// top padding so the header sits slightly above vertical center.
pub(super) const TOP_SPACE_RATIO: f32 = 0.4;
/// Approximate height the welcome content occupies; the space above it is
/// distributed via TOP_SPACE_RATIO.
pub(super) const WELCOME_BLOCK_HEIGHT: f32 = 400.0;

pub(super) const CARD_GAP: f32 = crate::tokens::SPACE_MD;
pub(super) const CARD_HEIGHT: f32 = 64.0;
/// Width above which the four cards lay out in a 2x2 grid instead of stacking.
pub(super) const WIDE_BREAKPOINT: f32 = 620.0;
pub(super) const STACKED_SIDE_INSET: f32 = 32.0;
pub(super) const MIN_CONTAINER_WIDTH: f32 = 280.0;
pub(super) const TITLE_DESC_GAP: f32 = 2.0;
/// Floor for the description wrap width so narrow cards never wrap to zero.
pub(super) const MIN_DESC_WRAP_WIDTH: f32 = 10.0;
