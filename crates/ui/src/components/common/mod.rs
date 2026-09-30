//! Shared component support that is not a standalone widget.
//!
//! The common layer owns reusable layout calculations and display formatting. It is kept
//! separate from component folders so components depend on a small, named support surface
//! instead of a catch-all utility implementation.

mod format;
mod layout;

pub use format::{
    format_bytes, format_count_with_suffix, format_duration_millis, format_page_range, format_percentage,
    truncate_ellipsis,
};
pub use layout::{calculate_dialog_layout, calculate_sheet_layout, clamp_popup_to_screen, DialogLayout};
