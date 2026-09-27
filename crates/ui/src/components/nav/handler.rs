use crate::DbProTheme;
use egui::Color32;
use lucide_icons::Icon;

/// Clamps the active page number within the 1-based range [1, page_count].
pub fn clamp_page(page: usize, page_count: usize) -> usize {
    let max_page = page_count.max(1);
    page.clamp(1, max_page)
}

/// Checks whether backward pagination navigation is possible.
pub fn can_navigate_prev(enabled: bool, page: usize) -> bool {
    enabled && page > 1
}

/// Checks whether forward pagination navigation is possible.
pub fn can_navigate_next(enabled: bool, page: usize, page_count: usize) -> bool {
    enabled && page < page_count.max(1)
}

/// Decrements the page index by 1 (bounded to 1).
pub fn prev_page(page: usize) -> usize {
    page.saturating_sub(1).max(1)
}

/// Increments the page index by 1 (bounded to page_count).
pub fn next_page(page: usize, page_count: usize) -> usize {
    let max_page = page_count.max(1);
    page.saturating_add(1).min(max_page)
}

/// Formats the current and total page count label (e.g., "1 / 10").
pub fn format_page_label(page: usize, page_count: usize) -> String {
    format!("{} / {}", page, page_count.max(1))
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
mod tests {
    use super::*;

    #[test]
    fn clamp_page_handles_zero_and_overflow() {
        assert_eq!(clamp_page(0, 5), 1);
        assert_eq!(clamp_page(3, 5), 3);
        assert_eq!(clamp_page(10, 5), 5);
        assert_eq!(clamp_page(0, 0), 1);
    }

    #[test]
    fn navigation_predicates_check_boundaries_and_enabled_flag() {
        assert!(!can_navigate_prev(true, 1));
        assert!(can_navigate_prev(true, 2));
        assert!(!can_navigate_prev(false, 2));

        assert!(can_navigate_next(true, 1, 5));
        assert!(!can_navigate_next(true, 5, 5));
        assert!(!can_navigate_next(false, 1, 5));
    }

    #[test]
    fn prev_and_next_page_bounds() {
        assert_eq!(prev_page(1), 1);
        assert_eq!(prev_page(5), 4);

        assert_eq!(next_page(1, 5), 2);
        assert_eq!(next_page(5, 5), 5);
        assert_eq!(next_page(usize::MAX, usize::MAX), usize::MAX);
    }

    #[test]
    fn format_page_label_renders_ratios() {
        assert_eq!(format_page_label(2, 8), "2 / 8");
        assert_eq!(format_page_label(1, 0), "1 / 1");
    }

    #[test]
    fn pagination_buttons_have_directional_accessible_names() {
        assert_eq!(page_button_accessible_label(Icon::ChevronLeft), "Previous page");
        assert_eq!(page_button_accessible_label(Icon::ChevronRight), "Next page");
    }

    #[test]
    fn breadcrumb_item_color_picks_primary_or_secondary() {
        let theme = DbProTheme::light();
        assert_eq!(breadcrumb_item_color(true, &theme), theme.text_primary);
        assert_eq!(breadcrumb_item_color(false, &theme), theme.text_secondary);
    }

    #[test]
    fn page_button_colors_computes_palette() {
        let theme = DbProTheme::light();
        let (fill_enabled_hover, text_enabled_hover) = page_button_colors(true, true, &theme);
        assert_eq!(fill_enabled_hover, theme.surface_hover);
        assert_eq!(text_enabled_hover, theme.text_primary);

        let (fill_disabled, text_disabled) = page_button_colors(false, false, &theme);
        assert_eq!(fill_disabled, theme.surface_panel);
        assert_eq!(text_disabled, theme.text_disabled);
    }
}
