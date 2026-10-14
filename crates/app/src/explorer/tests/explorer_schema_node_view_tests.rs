//! Tests for `../explorer_schema_node_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn schema_node_defaults_open_only_when_active() {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let theme = DbProTheme::light();
        for (is_active, expected_open) in [(false, false), (true, true)] {
            let mut seen_open = None;
            let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let render = SchemaNodeContext {
                        theme,
                        connection_id: "conn-1",
                        schema: "public",
                        is_active,
                        table_count: 0,
                    }
                    .draw(ui);
                    seen_open = Some(render.is_open);
                });
            });
            assert_eq!(seen_open, Some(expected_open));
        }
    }