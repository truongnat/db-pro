use egui::{Margin, Pos2, Rect};

use super::config::{
    DOUBLE_FACTOR, MAX_VISIBLE_ITEMS, MENU_MIN_SCREEN_WIDTH, MENU_MIN_WIDTH, MIN_TRIGGER_TEXT_WIDTH, MIN_TRIGGER_WIDTH,
    TRIGGER_TEXT_RESERVED_WIDTH, VERTICAL_CENTER_FACTOR,
};
use crate::tokens::{ICON_LG, ICON_TEXT_GAP, RADIUS_DROPDOWN, SPACE_SM, SPACE_XL, SPACE_XS, TABLE_ROW_HEIGHT_COMPACT};

pub fn dropdown_should_open_above(space_below: f32, space_above: f32, menu_h: f32) -> bool {
    space_below < menu_h && space_above > space_below
}

pub struct DropdownGeometry {
    pub menu_pos: Pos2,
    pub menu_width: f32,
    pub max_height: f32,
}

pub fn calculate_menu_geometry(
    screen: Rect,
    parent_rect: Rect,
    option_count: usize,
    has_more: bool,
) -> DropdownGeometry {
    // Count real options plus the optional action row, saturate overflow, then cap the visible row count.
    let extra = usize::from(has_more);
    let visible = option_count.saturating_add(extra).min(MAX_VISIBLE_ITEMS);
    let menu_h = visible as f32 * TABLE_ROW_HEIGHT_COMPACT + RADIUS_DROPDOWN * DOUBLE_FACTOR;
    // Measure nonnegative space from the trigger to each screen edge before choosing a direction.
    let space_below = (screen.bottom() - parent_rect.bottom()).max(0.0);
    let space_above = (parent_rect.top() - screen.top()).max(0.0);
    // Flip upward only if the menu does not fit below and above has more room; otherwise keep downward placement.
    let open_up = dropdown_should_open_above(space_below, space_above, menu_h);
    // Clamp width in order: at least trigger/design minimum, at most usable screen width (with minimum screen fallback).
    let menu_width = parent_rect
        .width()
        .max(MENU_MIN_WIDTH)
        .min((screen.width() - SPACE_SM * DOUBLE_FACTOR).max(MENU_MIN_SCREEN_WIDTH));
    let menu_left = parent_rect.left().clamp(
        screen.left() + SPACE_SM,
        (screen.right() - menu_width - SPACE_SM).max(screen.left() + SPACE_SM),
    );
    let menu_pos = if open_up {
        Pos2::new(menu_left, parent_rect.top() - SPACE_XS - menu_h)
    } else {
        Pos2::new(menu_left, parent_rect.bottom() + SPACE_XS)
    };

    DropdownGeometry {
        menu_pos,
        menu_width,
        max_height: MAX_VISIBLE_ITEMS as f32 * TABLE_ROW_HEIGHT_COMPACT,
    }
}

pub fn selected_label(options: &[String], selected: usize) -> &str {
    options
        .get(selected)
        .map(String::as_str)
        .unwrap_or("Select an option...")
}

pub fn trigger_accessibility_label(label: Option<&str>, selected: &str) -> String {
    label.map_or_else(|| selected.to_owned(), |label| format!("{label}: {selected}"))
}

pub fn trigger_content_width(width: f32, available: f32) -> f32 {
    (width - SPACE_XL).min(available).max(MIN_TRIGGER_WIDTH)
}

pub fn trigger_text_width(available: f32) -> f32 {
    (available - TRIGGER_TEXT_RESERVED_WIDTH).max(MIN_TRIGGER_TEXT_WIDTH)
}

pub fn trigger_inner_margin(padding_x: f32, padding_y: f32) -> Margin {
    Margin::symmetric(padding_x, padding_y + crate::tokens::SPACE_XXS)
}

pub fn menu_surface_margin(menu_margin_left: f32) -> Margin {
    Margin::symmetric(menu_margin_left, RADIUS_DROPDOWN)
}

pub fn menu_min_content_width(menu_width: f32) -> f32 {
    (menu_width - ICON_TEXT_GAP).max(0.0)
}

pub fn option_text_rect(rect: Rect, button_padding_x: f32, selected: bool) -> Rect {
    // Reserve trailing space only for selected rows, preventing text from colliding with the checkmark.
    let indicator_offset = if selected { SPACE_SM + ICON_LG } else { 0.0 };
    let text_left = rect.left() + button_padding_x;
    let text_right = rect.right() - button_padding_x - indicator_offset;
    Rect::from_min_max(Pos2::new(text_left, rect.top()), Pos2::new(text_right, rect.bottom()))
}

pub fn option_galley_pos(label_left: f32, vertical_center: f32, galley_height: f32) -> Pos2 {
    Pos2::new(label_left, vertical_center - galley_height * VERTICAL_CENTER_FACTOR)
}

pub fn option_check_icon_pos(rect_right: f32, vertical_center: f32) -> Pos2 {
    Pos2::new(rect_right - SPACE_SM, vertical_center)
}

pub fn should_toggle_popup(clicked: bool, focused: bool, keyboard_activation: bool) -> bool {
    clicked || (focused && keyboard_activation)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationKeys {
    pub arrow_down: bool,
    pub arrow_up: bool,
}

pub fn navigate_selection(selected: usize, option_count: usize, keys: NavigationKeys) -> usize {
    // Clamp movement at both ends; the optional load-more action is deliberately outside option_count.
    let selected = if keys.arrow_down && selected.saturating_add(1) < option_count {
        selected + 1
    } else {
        selected
    };

    if keys.arrow_up && selected > 0 {
        selected - 1
    } else {
        selected
    }
}

pub fn should_close_for_key(escape_pressed: bool, enter_pressed: bool) -> bool {
    escape_pressed || enter_pressed
}

pub fn selection_from_click(clicked: bool, option_index: usize) -> Option<usize> {
    clicked.then_some(option_index)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopupBounds {
    pub trigger: Rect,
    pub menu: Rect,
}

pub fn should_close_on_outside_click(clicked: bool, pos: Pos2, bounds: PopupBounds) -> bool {
    clicked && !bounds.trigger.contains(pos) && !bounds.menu.contains(pos)
}

pub fn should_request_load_more(clicked: bool, has_more: bool) -> bool {
    clicked && has_more
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_label_and_accessibility_label_have_fallbacks() {
        let options = vec!["Primary".to_owned()];
        assert_eq!(selected_label(&options, 0), "Primary");
        assert_eq!(selected_label(&options, 1), "Select an option...");
        assert_eq!(
            trigger_accessibility_label(Some("Cluster pool"), "Primary"),
            "Cluster pool: Primary"
        );
        assert_eq!(trigger_accessibility_label(None, "Primary"), "Primary");
    }

    #[test]
    fn keyboard_input_maps_to_selection_and_popup_actions() {
        assert!(should_toggle_popup(false, true, true));
        assert!(!should_toggle_popup(false, false, true));
        assert_eq!(
            navigate_selection(
                0,
                3,
                NavigationKeys {
                    arrow_down: true,
                    arrow_up: false,
                },
            ),
            1
        );
        assert_eq!(
            navigate_selection(
                2,
                3,
                NavigationKeys {
                    arrow_down: true,
                    arrow_up: false,
                },
            ),
            2
        );
        assert_eq!(
            navigate_selection(
                2,
                3,
                NavigationKeys {
                    arrow_down: false,
                    arrow_up: true,
                },
            ),
            1
        );
        assert!(should_close_for_key(true, false));
    }

    #[test]
    fn click_actions_require_valid_ui_signals() {
        let parent = Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 30.0));
        let popup = Rect::from_min_size(Pos2::new(0.0, 40.0), egui::vec2(100.0, 60.0));
        assert_eq!(selection_from_click(true, 2), Some(2));
        assert_eq!(selection_from_click(false, 2), None);
        let bounds = PopupBounds {
            trigger: parent,
            menu: popup,
        };
        assert!(!should_close_on_outside_click(true, Pos2::new(10.0, 10.0), bounds));
        assert!(should_close_on_outside_click(true, Pos2::new(200.0, 200.0), bounds));
        assert!(should_request_load_more(true, true));
        assert!(!should_request_load_more(false, true));
    }

    #[test]
    fn option_geometry_helpers_compute_correct_positions_and_bounds() {
        let row_rect = Rect::from_min_size(Pos2::new(10.0, 20.0), egui::vec2(200.0, 30.0));
        let padding_x = 8.0;

        let unselected_rect = option_text_rect(row_rect, padding_x, false);
        assert_eq!(unselected_rect.left(), 18.0);
        assert_eq!(unselected_rect.right(), 202.0);
        assert_eq!(unselected_rect.top(), 20.0);
        assert_eq!(unselected_rect.bottom(), 50.0);

        let selected_rect = option_text_rect(row_rect, padding_x, true);
        let expected_indicator_offset = SPACE_SM + ICON_LG;
        assert_eq!(selected_rect.left(), 18.0);
        assert_eq!(selected_rect.right(), 202.0 - expected_indicator_offset);

        let vertical_center = 35.0;
        let galley_height = 14.0;
        let label_pos = option_galley_pos(18.0, vertical_center, galley_height);
        assert_eq!(
            label_pos,
            Pos2::new(18.0, vertical_center - galley_height * VERTICAL_CENTER_FACTOR)
        );

        let check_pos = option_check_icon_pos(row_rect.right(), vertical_center);
        assert_eq!(check_pos, Pos2::new(row_rect.right() - SPACE_SM, vertical_center));
    }

    #[test]
    fn menu_and_trigger_geometry_helpers_compute_correct_values() {
        let margin = trigger_inner_margin(8.0, 4.0);
        assert_eq!(margin.left, 8.0);
        assert_eq!(margin.right, 8.0);
        assert_eq!(margin.top, 4.0 + crate::tokens::SPACE_XXS);

        let menu_margin = menu_surface_margin(12.0);
        assert_eq!(menu_margin.left, 12.0);
        assert_eq!(menu_margin.top, RADIUS_DROPDOWN);

        let min_content_width = menu_min_content_width(150.0);
        assert_eq!(min_content_width, 150.0 - ICON_TEXT_GAP);
        assert_eq!(menu_min_content_width(2.0), 0.0);
    }
}
