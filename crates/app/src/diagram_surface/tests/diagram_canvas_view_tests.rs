//! Tests for `../diagram_canvas_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn dragging_canvas_updates_pan() {
        let context = egui::Context::default();
        DbProTheme::install_fonts(&context);
        let explorer = SchemaExplorerState::default();
        let mut diagram = DiagramState::default();
        let start = egui::pos2(160.0, 160.0);
        run_canvas_frame(&context, &explorer, &mut diagram, vec![egui::Event::PointerMoved(start)]);
        run_canvas_frame(&context, &explorer, &mut diagram, vec![press_event(start)]);
        // Two separate move frames: a per-frame delta accumulation bug would
        // leave pan at the last frame's (10, 5) instead of the full (74, 53).
        run_canvas_frame(&context, &explorer, &mut diagram, vec![moved_to(start + egui::vec2(64.0, 48.0))]);
        run_canvas_frame(&context, &explorer, &mut diagram, vec![moved_to(start + egui::vec2(74.0, 53.0))]);
        run_canvas_frame(&context, &explorer, &mut diagram, vec![release_event(start + egui::vec2(74.0, 53.0))]);

        assert_eq!(
            diagram.pan,
            egui::vec2(74.0, 53.0),
            "pan must track the full distance from the press point, not per-frame motion"
        );
    }

    fn run_canvas_frame(
        context: &egui::Context,
        explorer: &SchemaExplorerState,
        diagram: &mut DiagramState,
        events: Vec<egui::Event>,
    ) {
        let mut view = DiagramViewContext {
            theme: DbProTheme::dark(),
            diagram,
            explorer,
            active_driver: "SQLite",
            connected: true,
            connection_id: None,
        };
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900.0, 600.0));
        let _ = crate::test_frame::frame(&context, egui::RawInput {
            screen_rect: Some(screen),
            events,
            ..Default::default()
        }, |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                let (response, _) =
                    ui.allocate_painter(egui::vec2(880.0, 560.0), egui::Sense::click_and_drag());
                let viewport = ErViewport::new(view.diagram.pan, view.diagram.zoom, response.rect.min);
                handle_drag_input(&mut view, &response, viewport, &[]);
            });
        });
    }

    fn moved_to(pos: egui::Pos2) -> egui::Event {
        egui::Event::PointerMoved(pos)
    }

    fn press_event(pos: egui::Pos2) -> egui::Event {
        button_event(pos, true)
    }

    fn release_event(pos: egui::Pos2) -> egui::Event {
        button_event(pos, false)
    }

    fn button_event(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }