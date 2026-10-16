//! Tests for `../diagram_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn fit_view_uses_visible_viewport_instead_of_scroll_content_size() {
        let viewport_rect = egui::Rect::from_min_size(egui::pos2(40.0, 70.0), egui::vec2(900.0, 600.0));
        let bounds = egui::Rect::from_min_size(egui::pos2(48.0, 48.0), egui::vec2(3200.0, 2400.0));

        let (zoom, pan) = fit_diagram_viewport(bounds, viewport_rect).expect("non-empty diagram");
        let fitted = ErViewport::new(pan, zoom, viewport_rect.min).world_to_screen_rect(bounds);

        assert!(fitted.left() >= viewport_rect.left() - 1.0);
        assert!(fitted.right() <= viewport_rect.right() + 1.0);
        assert!(fitted.top() >= viewport_rect.top() - 1.0);
        assert!(fitted.bottom() <= viewport_rect.bottom() + 1.0);
        assert!(zoom < 0.5, "large diagram should be allowed to zoom out to fit: {zoom}");
    }

#[test]
fn self_reference_edge_bend_stays_outside_card_across_zoom_and_lanes() {
    let card_right = 420.0;
    for zoom in [0.1, 0.5, 1.0, 2.0] {
        for lane in [-27.0 * zoom, 0.0, 27.0 * zoom] {
            let bend_x = diagram_edge_bend_x(420.0, 420.0, card_right, true, lane, zoom);
            assert!(bend_x > card_right, "self-loop bend {bend_x} must clear card at zoom {zoom}");
        }
    }
}

#[test]
fn regular_edge_bend_keeps_midpoint_routing() {
    assert_eq!(diagram_edge_bend_x(100.0, 300.0, 500.0, false, 12.0, 1.0), 212.0);
}


#[test]
fn self_reference_anchors_share_the_same_card_side() {
    assert_eq!(diagram_edge_anchor_sides(3, 3, 420.0, 420.0), (true, true));
    assert_eq!(diagram_edge_anchor_sides(4, 2, 500.0, 300.0), (false, true));
}
