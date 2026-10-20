use super::*;

#[test]
fn compact_query_controls_stay_inside_remaining_workspace() {
    for width in [320.0, 420.0, 572.0, 619.0, 860.0] {
        for running in [false, true] {
            let ctx = egui::Context::default();
            DbProTheme::install_fonts(&ctx);
            let mut editor = QueryEditorState::default();
            let mut execution = QueryExecutionPolicyState::default();
            let _ = crate::test_frame::frame(
                &ctx,
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 400.0))),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let available = ui.available_rect_before_wrap();
                        let mut context = QueryContextViewContext {
                            theme: DbProTheme::dark(),
                            editor: &mut editor,
                            execution: &mut execution,
                            file_path: None,
                            connected: true,
                            connection_name: "Regional analytics warehouse with a very long connection name",
                            schema: "customer_retention_reporting_and_revenue_operations",
                            environment: db_pro_core::domain::connection::ConnectionEnvironment::Development,
                            active_request_id: running.then_some(RequestId(9)),
                            execution_started_at: running.then(std::time::Instant::now),
                            cancel_supported: true,
                            cancel_reason: None,
                            modifier: "Cmd",
                        };
                        let output = draw_context_strip(&mut context, ui);
                        for anchor in [output.anchors.context_anchor, output.anchors.more_anchor] {
                            let rect = anchor.expect("context and More remain present");
                            assert!(rect.left() >= available.left() - 0.5, "{width}: {rect:?}");
                            assert!(rect.right() <= available.right() + 0.5, "{width}: {rect:?}");
                        }
                        assert!(
                            ui.min_rect().right() <= available.right() + 0.5,
                            "all run, row-cap and menu controls must fit at {width}"
                        );
                    });
                },
            );
        }
    }
}
