use egui::Color32;
use lucide_icons::Icon;

use crate::DbProTheme;

fn effective_page_count(page_count: usize) -> usize {
    page_count.max(1)
}

/// Clamps the active page number within the 1-based range [1, page_count].
pub fn clamp_page(page: usize, page_count: usize) -> usize {
    let max_page = effective_page_count(page_count);
    page.clamp(1, max_page)
}

/// Checks whether backward pagination navigation is possible.
pub fn can_navigate_prev(enabled: bool, page: usize) -> bool {
    enabled && page > 1
}

/// Checks whether forward pagination navigation is possible.
pub fn can_navigate_next(enabled: bool, page: usize, page_count: usize) -> bool {
    enabled && page < effective_page_count(page_count)
}

/// Decrements the page index by 1 (bounded to 1).
pub fn prev_page(page: usize) -> usize {
    page.saturating_sub(1).max(1)
}

/// Increments the page index by 1 (bounded to page_count).
pub fn next_page(page: usize, page_count: usize) -> usize {
    let max_page = effective_page_count(page_count);
    page.saturating_add(1).min(max_page)
}

/// Formats the current and total page count label (e.g., "1 / 10").
pub fn format_page_label(page: usize, page_count: usize) -> String {
    format!("{} / {}", page, effective_page_count(page_count))
}

/// Returns an accessible action label for the pagination chevron.
pub fn page_button_accessible_label(icon: Icon) -> &'static str {
    match icon {
        Icon::ChevronLeft => "Previous page",
        Icon::ChevronRight => "Next page",
        _ => "Pagination action",
    }
}

/// Returns the text color for a breadcrumb segment depending on whether it is the active/current item.
pub fn breadcrumb_item_color(current: bool, theme: &DbProTheme) -> Color32 {
    if current {
        theme.text_primary
    } else {
        theme.text_secondary
    }
}

/// Determines the background fill and icon text color for pagination buttons based on enabled/hover states.
pub fn page_button_colors(enabled: bool, hovered: bool, theme: &DbProTheme) -> (Color32, Color32) {
    let fill = if enabled && hovered {
        theme.surface_hover
    } else {
        theme.surface_panel
    };
    let color = if enabled {
        theme.text_primary
    } else {
        theme.text_disabled
    };
    (fill, color)
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
