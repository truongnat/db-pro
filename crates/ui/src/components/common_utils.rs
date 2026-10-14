//! Backwards-compatible path for the shared component support layer.
//!
//! New code should import from [`super::common`]. This facade keeps existing callers on
//! `components::common_utils::*` source-compatible while the implementation lives in the
//! named common submodules.

pub use super::common::{
    calculate_dialog_layout, calculate_sheet_layout, clamp_popup_to_screen, format_bytes, format_count_with_suffix,
    format_duration_millis, format_page_range, format_percentage, truncate_ellipsis, DialogLayout,
};

#[cfg(test)]
#[path = "tests/common_utils_tests.rs"]
mod tests;
