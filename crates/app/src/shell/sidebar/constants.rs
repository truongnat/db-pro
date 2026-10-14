//! Sidebar layout + option geometry. Values are derived from the shared scale
//! (`crate::tokens`) or component contracts; comments name intent, not pixels.

use super::{ProblemsSeverityFilter, ProblemsSourceFilter};

/// Extra pixels the sidebar clip rect bleeds past its edge so the resize grip
/// hover ring and button strokes are not clipped at the rail boundary.
pub(super) const SIDEBAR_CLIP_BLEED: f32 = 1.0;

/// IconSm button (26) plus the XS gap reserve — the plus slot stays fixed so
/// the connection selector never jumps when the sidebar resizes.
pub(super) const PLUS_SLOT_W: f32 = 28.0;

/// Selector strip height: one text line at BODY_SM in a compact rail.
pub(super) const SELECTOR_H: f32 = crate::tokens::SPACE_2XL;

/// Right-side search glyph slot inside the selector strip.
pub(super) const SEARCH_SLOT_W: f32 = crate::tokens::SPACE_LG + crate::tokens::SPACE_XXS;

/// Below this width the connection label would show a single glyph + ellipsis;
/// collapsing further is unreadable, so the floor wins over shrinking.
pub(super) const MIN_SELECTOR_W: f32 = crate::tokens::SPACE_6XL + crate::tokens::SPACE_SM;

/// The "New query" strip matches the small button contract height.
pub(super) const NEW_QUERY_HEIGHT: f32 = crate::tokens::component::button::BUTTON_HEIGHT_SM;

/// Problems severity filter order; labels are locale keys, resolved at draw.
pub(super) const SEVERITY_OPTIONS: [(ProblemsSeverityFilter, &str); 3] = [
    (ProblemsSeverityFilter::All, "sidebar.filter_all"),
    (ProblemsSeverityFilter::Errors, "sidebar.filter_errors"),
    (ProblemsSeverityFilter::Warnings, "sidebar.filter_warnings"),
];

/// Problems source filter order; labels are locale keys, resolved at draw.
pub(super) const SOURCE_OPTIONS: [(ProblemsSourceFilter, &str); 5] = [
    (ProblemsSourceFilter::All, "sidebar.filter_all_sources"),
    (ProblemsSourceFilter::Parser, "sidebar.filter_parser"),
    (ProblemsSourceFilter::Lint, "sidebar.filter_lint"),
    (ProblemsSourceFilter::Delimiter, "sidebar.filter_delimiter"),
    (ProblemsSourceFilter::Database, "sidebar.filter_database"),
];
