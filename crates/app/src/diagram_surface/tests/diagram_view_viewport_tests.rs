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