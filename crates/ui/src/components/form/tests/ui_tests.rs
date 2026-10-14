//! Tests for `../ui.rs` (moved out of the source file).
use super::*;

#[test]
fn field_label_marker_and_helper_gaps_have_one_spacing_owner() {
    for spacing in [egui::vec2(0.0, 0.0), egui::vec2(8.0, 4.0), egui::vec2(16.0, 16.0)] {
        let ctx = egui::Context::default();
        let theme = DbProTheme::light();
        DbProTheme::install_fonts(&ctx);
        theme.apply(&ctx);
        let mut value = String::from("database");
        let output = crate::test_frame::frame(&ctx, egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.set_width(300.0);
                ui.spacing_mut().item_spacing = spacing;
                FormField::new("Host", &mut value, "", theme)
                    .required(true)
                    .helper_text("Helper")
                    .show(ui);
                assert_eq!(ui.spacing().item_spacing, spacing);
            });
        });
        let text = |name: &str| {
            output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.text() == name => Some(text),
                    _ => None,
                })
                .expect("painted text")
        };
        let label = text("Host");
        let star = text("*");
        let helper = text("Helper");
        let frame = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect) if rect.fill == theme.surface_editor => Some(rect.rect),
                _ => None,
            })
            .expect("field frame");
        assert_eq!(star.pos.x - label.pos.x - label.galley.size().x, LABEL_HELPER_GAP);
        assert_eq!(frame.top() - label.pos.y - label.galley.size().y, LABEL_HELPER_GAP);
        assert_eq!(helper.pos.y - frame.bottom(), LABEL_HELPER_GAP);
    }
}
