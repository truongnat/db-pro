use super::*;

#[test]
fn ddl_actions_and_confirmation_disclosure_fit_narrow_workspace() {
    let context = egui::Context::default();
    DbProTheme::install_fonts(&context);
    for width in [360.0, 560.0, 1024.0] {
        let mut ddl = "CREATE TABLE customers (id integer);".to_owned();
        let script = DdlScriptContext {
            theme: DbProTheme::dark(),
            writable: true,
            executing: false,
            error: None,
            ddl: &mut ddl,
        };
        let _ = crate::test_frame::frame(
            &context,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 400.0))),
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    let right = ui.max_rect().right();
                    assert!(draw_toolbar(
                        &DdlToolbarContext {
                            theme: DbProTheme::dark()
                        },
                        ui
                    )
                    .is_none());
                    assert!(draw_script_header(&script, ui).is_none());
                    assert!(
                        ui.min_rect().right() <= right + 1.0,
                        "DDL actions escape {width}-point workspace: {:?}",
                        ui.min_rect()
                    );
                });
            },
        );
    }
}
