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
    assert_eq!(menu_key_action(true, false), MenuKeyAction::Cancel);
    assert_eq!(menu_key_action(false, true), MenuKeyAction::Commit);
    assert_eq!(menu_key_action(false, false), MenuKeyAction::None);
    // Enter wins when both arrive in one frame; there is nothing to cancel after a commit.
    assert_eq!(menu_key_action(true, true), MenuKeyAction::Commit);
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
fn option_labels_stay_on_one_truncated_line() {
    let job = option_label_job(
        "a very long schema name that cannot fit".to_owned(),
        egui::Color32::WHITE,
        120.0,
    );
    assert_eq!(job.wrap.max_width, 120.0);
    assert_eq!(job.wrap.max_rows, 1);
    assert!(job.wrap.break_anywhere);
    assert_eq!(job.wrap.overflow_character, Some('…'));
}

#[test]
fn menu_and_trigger_geometry_helpers_compute_correct_values() {
    let margin = trigger_inner_margin(8.0, 4.0);
    assert_eq!(margin.left, 8);
    assert_eq!(margin.right, 8);
    assert_eq!(margin.top, (4.0 + crate::tokens::SPACE_XXS) as i8);

    let menu_margin = menu_surface_margin(12.0);
    assert_eq!(menu_margin.left, 12);
    assert_eq!(menu_margin.top, SPACE_SM as i8);

    let min_content_width = menu_min_content_width(150.0);
    assert_eq!(min_content_width, 150.0 - ICON_TEXT_GAP);
    assert_eq!(menu_min_content_width(2.0), 0.0);
}

#[test]
fn menu_row_width_matches_the_columns_paint_option_reserves() {
    let pad = 8.0;
    // Label only: symmetric padding around the text.
    assert_eq!(menu_row_width(90.0, false, false, pad), 90.0 + pad * 2.0);
    // Leading icon: 16px column plus the gap before the label.
    assert_eq!(
        menu_row_width(90.0, true, false, pad),
        90.0 + pad * 2.0 + OPTION_ICON_SIDE + SPACE_XS
    );
    // Selected row: the check indicator column is reserved so navigating never
    // resizes the popup.
    assert_eq!(
        menu_row_width(90.0, false, true, pad),
        90.0 + pad * 2.0 + SPACE_SM + ICON_LG
    );
}

#[test]
fn trigger_preferred_width_covers_label_chevron_padding_and_stroke() {
    let margin = trigger_inner_margin(8.0, 4.0);
    let outline = trigger_preferred_width(90.0, margin, 2.0);
    assert_eq!(
        outline,
        90.0 + TRIGGER_TEXT_RESERVED_WIDTH + margin.leftf() + margin.rightf() + 2.0
    );
    // A borderless ghost needs no stroke allowance.
    assert_eq!(trigger_preferred_width(90.0, margin, 0.0), outline - 2.0);
}

#[test]
fn menu_width_fits_the_widest_option_instead_of_a_magic_number() {
    let screen = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1440.0, 900.0));
    let trigger = egui::Rect::from_min_max(egui::Pos2::new(100.0, 100.0), egui::Pos2::new(208.0, 138.0));
    // Long option, short trigger: the popup grows so the choice is readable.
    let roomy = menu_row_width(400.0, false, true, 8.0);
    assert_eq!(
        calculate_menu_geometry(screen, trigger, 4, false, roomy).menu_width,
        roomy
    );
    // Short options: no 152px floor any more — the popup stops at the trigger.
    assert_eq!(
        calculate_menu_geometry(screen, trigger, 4, false, 40.0).menu_width,
        trigger.width()
    );
    // Empty list still gets a usable target.
    let tiny = egui::Rect::from_min_max(egui::Pos2::new(100.0, 100.0), egui::Pos2::new(120.0, 138.0));
    assert_eq!(
        calculate_menu_geometry(screen, tiny, 0, false, 0.0).menu_width,
        MIN_TRIGGER_WIDTH
    );
    // Never wider than the screen allows.
    assert!(calculate_menu_geometry(screen, trigger, 4, false, 5000.0).menu_width <= screen.width() - SPACE_SM * 2.0);
}

#[test]
fn menu_max_width_clamps_to_the_screen_with_a_usable_floor() {
    let wide = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1440.0, 900.0));
    assert_eq!(menu_max_width(wide), 1440.0 - SPACE_SM * DOUBLE_FACTOR);
    // A viewport narrower than the floor still gets a tappable popup.
    let narrow = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(40.0, 900.0));
    assert_eq!(menu_max_width(narrow), MENU_MIN_SCREEN_WIDTH);
}

#[test]
fn widest_label_width_stops_measuring_once_the_cap_is_reached() {
    let ctx = egui::Context::default();
    crate::DbProTheme::install_fonts(&ctx);
    let _ = crate::test_frame::frame(&ctx, egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let font = egui::FontId::proportional(13.0);
            let short = ["ab", "abcd", "abc"];
            let none: [&str; 0] = [];
            // Below the cap: the widest label is reported exactly.
            let measured = widest_label_width(ui, &short, &font, 500.0);
            assert!(measured > 0.0 && measured < 500.0);
            assert_eq!(measured, widest_label_width(ui, &["abcd"], &font, 500.0));
            // Above the cap: the scan returns the cap instead of the real width,
            // because every caller clamps to it anyway.
            assert_eq!(widest_label_width(ui, &short, &font, 1.0), 1.0);
            assert_eq!(widest_label_width(ui, &short, &font, 0.0), 0.0);
            assert_eq!(widest_label_width(ui, &none, &font, 120.0), 0.0);
        });
    });
}
