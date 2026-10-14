//! Tests for `../button.rs` (moved out of the source file).
use super::*;

#[test]
// cc-scan:allow DEEP_NESTING,LONG_FUNCTION — nesting mirrors structure depth
fn press_scales_background_icon_text_and_underline_together() {
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let button = Button::new(DbProTheme::light()).icon(Icon::Play).text("Run");
            let tokens = SizeTokens::from_size(button.size);
            let galley =
                ui.fonts_mut(|fonts| fonts.layout_job(button_label_layout_job(&button, &tokens, Color32::WHITE)));
            let original = galley.rows[0].visuals.mesh.vertices.clone();
            let rect = Rect::from_min_size(Pos2::new(20.0, 20.0), Vec2::new(140.0, 40.0));
            let scale = press_scale(1.0);
            for left_aligned in [false, true] {
                let index = ui.painter().add(Shape::Noop);
                paint_interactive_surface(
                    ui,
                    InteractiveLayout {
                        paint_rect: rect,
                        scale,
                        fill: Color32::BLUE,
                        stroke: Stroke::NONE,
                        galley: &galley,
                        text_color: Color32::WHITE,
                        left_aligned,
                        horizontal_padding: 12.0,
                        underline: true,
                    },
                );
                ui.ctx().graphics(|graphics| {
                    let list = graphics.get(ui.layer_id()).unwrap();
                    let shape = &list.all_entries().nth(index.0 + 1).unwrap().shape;
                    let Shape::Vec(shapes) = shape else {
                        panic!("button shapes")
                    };
                    let Shape::Rect(background) = &shapes[0] else {
                        panic!("background")
                    };
                    assert_eq!(background.rect.center(), rect.center());
                    assert!((background.rect.size() - rect.size() * scale).length() < 0.001);
                    let Shape::Text(text) = &shapes[1] else { panic!("text") };
                    for (before, after) in original.iter().zip(&text.galley.rows[0].visuals.mesh.vertices) {
                        assert!((after.pos.to_vec2() - before.pos.to_vec2() * scale).length() < 0.001);
                        assert_eq!(after.uv, before.uv);
                    }
                    let Shape::LineSegment { points, .. } = &shapes[2] else {
                        panic!("underline")
                    };
                    assert!(((points[1].x - points[0].x) - galley.size().x * scale).abs() < 0.001);
                });
            }
            assert_eq!(galley.rows[0].visuals.mesh.vertices, original);
        });
    });
}

/// Visible or explicit labels provide stable accessibility names; a tooltip is only a fallback.
#[test]
fn accessible_name_prefers_access_label_then_label_then_tooltip() {
    let theme = DbProTheme::dark();

    let icon_only_with_tooltip = Button::new(theme).icon(Icon::X).tooltip("Close Agent");
    assert_eq!(icon_only_with_tooltip.accessible_name(), "Close Agent");

    let labelled = Button::new(theme).icon(Icon::X).text("Close").tooltip("Close Agent");
    assert_eq!(labelled.accessible_name(), "Close");

    let explicit = Button::new(theme)
        .icon(Icon::X)
        .text("Close")
        .access_label("Dismiss the agent panel")
        .tooltip("Close Agent");
    assert_eq!(explicit.accessible_name(), "Dismiss the agent panel");
}

#[test]
fn a_button_with_no_name_at_all_still_has_one() {
    let theme = DbProTheme::dark();
    assert_eq!(Button::new(theme).accessible_name(), "Button");
}

#[test]
fn icon_only_button_requires_an_explicit_accessible_name_when_rendered() {
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let theme = DbProTheme::dark();
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let response = Button::new(theme).icon(Icon::X).access_label("Close panel").show(ui);
            assert!(response.rect.is_positive());
        });
    });
}

#[test]
#[should_panic(expected = "Icon-only buttons require an explicit access_label")]
fn icon_only_button_without_access_label_is_rejected_in_all_builds() {
    let ctx = egui::Context::default();
    let theme = DbProTheme::dark();
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            Button::new(theme).icon(Icon::X).tooltip("Close panel").show(ui);
        });
    });
}
