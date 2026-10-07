use super::handler::dropdown_should_open_above;

#[test]
fn dropdown_flips_above_when_there_is_no_room_below() {
    assert!(dropdown_should_open_above(40.0, 280.0, 260.0));
    assert!(!dropdown_should_open_above(300.0, 40.0, 260.0));
    assert!(!dropdown_should_open_above(200.0, 200.0, 180.0));
}

#[test]
fn compact_select_matches_toolbar_button_height_and_alignment() {
    use crate::components::{Button, ButtonSize, Select, SelectSize, SelectVariant};
    let ctx = egui::Context::default();
    crate::DbProTheme::install_fonts(&ctx);
    let theme = crate::DbProTheme::light();
    let options = vec!["500 rows".to_owned()];
    let mut selected = 0;
    for variant in [SelectVariant::Outline, SelectVariant::Ghost] {
        for _ in 0..2 {
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let button = Button::new(theme).text("Run").size(ButtonSize::Sm).show(ui);
                        let select = Select::new("compact_height", &mut selected, &options)
                            .theme(theme)
                            .width(108.0)
                            .size(SelectSize::Sm)
                            .variant(variant)
                            .show(ui);
                        assert!((select.rect.height() - button.rect.height()).abs() < 0.1);
                        assert!((select.rect.center().y - button.rect.center().y).abs() < 0.1);
                    });
                });
            });
        }
    }
}

#[test]
fn default_select_retains_form_height() {
    let ctx = egui::Context::default();
    crate::DbProTheme::install_fonts(&ctx);
    let options = vec!["500 rows".to_owned()];
    let mut selected = 0;
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let response = super::Select::new("form_height", &mut selected, &options)
                .width(120.0)
                .show(ui);
            assert!((response.rect.height() - crate::tokens::component::input::INPUT_HEIGHT_DEFAULT).abs() < 0.1);
        });
    });
}
