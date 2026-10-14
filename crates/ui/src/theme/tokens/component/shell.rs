//! Shell (window chrome) token contract.
//!
//! The application frame is not a reusable widget, so its dimensions live in
//! one contract instead of being spread across views. Colors still come from
//! `DbProTheme`; the values below are the shell-owned geometry.
//!
//! Owned values: activity rail width and the reserved block for the trailing
//! settings button. Anything per-view (tab bar height, status bar height)
//! belongs to that view's own config, not here.

use super::super::primitive::SPACE_4XL;
use super::super::primitive::SPACE_5XL;
use super::super::primitive::SPACE_XS;

/// Width of the left activity rail. Sized to fit one `ButtonSize::IconSm`
/// button (26px) centered with breathing room on both sides — the rail is a
/// fixed strip, not a resizable panel.
pub const ACTIVITY_RAIL_WIDTH: f32 = SPACE_5XL;

/// Vertical space reserved at the bottom of the activity rail for the settings
/// button plus its margin: `IconSm` height (26) does not cover hover ring +
/// spacing, so the reserve is `SPACE_4XL` (40) + `SPACE_XS` (4) rather than the
/// raw button height.
pub const ACTIVITY_RAIL_SETTINGS_RESERVE: f32 = SPACE_4XL + SPACE_XS;

#[cfg(test)]
#[path = "tests/shell_tests.rs"]
mod tests;
