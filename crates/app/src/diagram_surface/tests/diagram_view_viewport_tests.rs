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
fn large_schema_search_and_design_inputs_fit_narrow_workspace() {
    let context = egui::Context::default();
    DbProTheme::install_fonts(&context);
    for width in [420.0, 680.0, 1200.0] {
        let (bridge, _commands, _events) = TaskBridge::with_channels();
        let mut app = DbProApp::with_task_bridge(bridge);
        app.open_data_cleanup_for_capture("diagram-design", false);
        let mut view = DiagramViewContext {
            theme: app.theme,
            diagram: &mut app.schema.diagram,
            explorer: &app.schema.explorer,
            active_driver: "postgresql",
            connected: true,
            connection_id: None,
        };
        let frame = measure_diagram_frame(&view, "customers");
        let _ = crate::test_frame::frame(
            &context,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 1000.0))),
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    let right = ui.max_rect().right();
                    draw_diagram_toolbar(&mut view, ui, &frame);
                    assert!(
                        ui.min_rect().right() <= right + 1.0,
                        "large-schema toolbar must fit at {width}"
                    );
                    super::super::diagram_design_panel_view::draw_er_design_panel(&mut view, ui);
                    assert!(
                        ui.min_rect().right() <= right + 1.0,
                        "large-schema controls escape {width}-point workspace: {:?}",
                        ui.min_rect()
                    );
                });
            },
        );
    }
}
