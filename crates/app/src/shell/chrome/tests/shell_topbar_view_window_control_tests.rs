//! Tests for `../shell_topbar_view.rs` (moved out of the source file).
    use super::*;
    use egui::{Context, Event, Pos2, RawInput, Rect, Vec2, ViewportId};

    const SCREEN: Vec2 = Vec2::new(800.0, 600.0);

    fn input(events: Vec<Event>) -> RawInput {
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
            events,
            ..Default::default()
        }
    }

    fn commands(ctx: &Context, events: Vec<Event>) -> Vec<egui::ViewportCommand> {
        crate::test_frame::frame(&ctx, input(events), |ui| {
            draw_window_resize_handles(ui.ctx());
        })
        .viewport_output
        .remove(&ViewportId::ROOT)
        .expect("root viewport output")
        .commands
    }

    fn render_controls(ctx: &Context, events: Vec<Event>, rects: &mut [Rect; 3]) -> egui::FullOutput {
        crate::test_frame::frame(&ctx, input(events), |ui| {
            egui::Panel::top("test_topbar")
                .exact_size(38.0)
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let ctx = ui.ctx().clone();
                        *rects = ShellTopbarContext::draw_window_controls(ui, &ctx, DbProTheme::light());
                    });
                });
            draw_window_resize_handles(ui.ctx());
        })
    }

    fn click_control(control_index: usize) -> Vec<egui::ViewportCommand> {
        let ctx = Context::default();
        DbProTheme::install_fonts(&ctx);
        let mut rects = [Rect::NOTHING; 3];
        for _ in 0..2 {
            let _ = render_controls(&ctx, Vec::new(), &mut rects);
        }
        let position = rects[control_index].center();
        let _ = render_controls(&ctx, vec![Event::PointerMoved(position)], &mut rects);
        let _ = render_controls(
            &ctx,
            vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            &mut rects,
        );
        render_controls(
            &ctx,
            vec![Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
            &mut rects,
        )
        .viewport_output
        .remove(&ViewportId::ROOT)
        .expect("root viewport output")
        .commands
    }

    #[test]
    fn frameless_window_resize_handles_only_start_native_resize_at_the_edge() {
        let ctx = Context::default();
        let _ = commands(&ctx, Vec::new());
        let _ = commands(&ctx, Vec::new());

        let press = |position| {
            vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ]
        };
        let drag = |position| vec![Event::PointerMoved(position)];

        let _ = commands(&ctx, press(Pos2::new(400.0, 2.0)));
        let edge_commands = commands(&ctx, drag(Pos2::new(410.0, 2.0)));
        assert!(edge_commands.contains(&egui::ViewportCommand::BeginResize(egui::ResizeDirection::North)));

        let ctx = Context::default();
        let _ = commands(&ctx, Vec::new());
        let _ = commands(&ctx, Vec::new());
        let _ = commands(&ctx, press(Pos2::new(400.0, 300.0)));
        let interior_commands = commands(&ctx, drag(Pos2::new(410.0, 300.0)));
        assert!(!interior_commands
            .iter()
            .any(|command| matches!(command, egui::ViewportCommand::BeginResize(_))));
    }

    #[test]
    fn linux_window_buttons_emit_viewport_commands_through_resize_overlay() {
        assert!(click_control(2).contains(&egui::ViewportCommand::Minimized(true)));
        assert!(click_control(1).contains(&egui::ViewportCommand::Maximized(true)));
        assert!(click_control(0).contains(&egui::ViewportCommand::Close));
    }