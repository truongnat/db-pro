//! Tests for `../handler.rs` (moved out of the source file).
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
fn stale_selection_index_clamps_when_options_shrink() {
    // Issue #480: options shrinking 5→2 left ArrowDown stuck at index 5.
    let down = NavigationKeys {
        arrow_down: true,
        arrow_up: false,
    };
    let up = NavigationKeys {
        arrow_down: false,
        arrow_up: true,
    };
    let none = NavigationKeys {
        arrow_down: false,
        arrow_up: false,
    };
    assert_eq!(navigate_selection(5, 2, down), 1);
    assert_eq!(navigate_selection(5, 2, none), 1);
    assert_eq!(navigate_selection(5, 3, up), 1);
    // Empty list: clamps to 0; render paths read via `options.get`, so it is inert.
    assert_eq!(navigate_selection(5, 0, down), 0);
}

#[test]
fn keyboard_input_maps_to_selection_and_popup_actions() {
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
    assert_eq!(selection_from_click(true, 2), Some(2));
    assert_eq!(selection_from_click(false, 2), None);
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
    assert_eq!(margin.left, 8);
    assert_eq!(margin.right, 8);
    assert_eq!(margin.top, (4.0 + crate::tokens::SPACE_XXS) as i8);

    let menu_margin = menu_surface_margin(12.0);
    assert_eq!(menu_margin.left, 12);
    assert_eq!(menu_margin.top, RADIUS_DROPDOWN as i8);

    let min_content_width = menu_min_content_width(150.0);
    assert_eq!(min_content_width, 150.0 - ICON_TEXT_GAP);
    assert_eq!(menu_min_content_width(2.0), 0.0);
}
