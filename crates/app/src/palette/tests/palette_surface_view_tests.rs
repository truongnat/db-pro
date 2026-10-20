use super::*;

#[test]
fn empty_palette_keeps_keyboard_help_inside_the_dialog() {
    for theme in [DbProTheme::light(), DbProTheme::dark()] {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        theme.apply(&ctx);
        let mut palette = PaletteState::default();
        palette.open_with_scope(PaletteMode::QuickOpen, SearchScope::Schema);
        let mut help_visible = false;
        for frame in 0..5 {
            let output = crate::test_frame::frame(
                &ctx,
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0))),
                    time: Some(frame as f64),
                    ..Default::default()
                },
                |ui| {
                    ui.set_style(ui.ctx().global_style());
                    PaletteSurfaceContext {
                        theme,
                        palette: &mut palette,
                        items: &[],
                        title: "Quick Open",
                        description: "Switch workspaces",
                    }
                    .draw(ui.ctx());
                },
            );
            help_visible = output.shapes.iter().any(|shape| {
                if let egui::epaint::Shape::Text(text) = &shape.shape {
                    text.galley.text().contains("Navigate")
                        && shape
                            .clip_rect
                            .contains_rect(egui::Rect::from_min_size(text.pos, text.galley.size()))
                } else {
                    false
                }
            });
        }
        assert!(help_visible, "empty palette must not scroll keyboard help out of view");
    }
}
