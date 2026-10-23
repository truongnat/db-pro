use super::{config::*, layout::screen_rect_fallback};

#[test]
fn dialog_constants_follow_design_tokens() {
    assert_eq!(DIALOG_RADIUS, crate::tokens::RADIUS_DIALOG);
    assert_eq!(SHEET_CORNER_RADIUS, crate::tokens::RADIUS_DIALOG);
    assert_eq!(DIALOG_WIDTH, 420.0);
    assert_eq!(SHEET_WIDTH, 360.0);
    assert_eq!(DIALOG_TRANSLATE_PX, 8.0);
    assert_eq!(SHEET_TRANSLATE_PX, 16.0);
}

#[test]
fn screen_rect_fallback_returns_safe_size() {
    let ctx = egui::Context::default();
    let screen = screen_rect_fallback(&ctx);
    assert!(screen.width() >= 1280.0);
    assert!(screen.height() >= 800.0);
}

#[test]
fn public_dialog_and_sheet_compatibility_paths_remain_available() {
    let theme = crate::DbProTheme::light();
    let mut open = true;

    {
        let _modal_dialog = super::modal::Dialog::new(&mut open, "Modal", theme);
    }
    {
        let _root_dialog = super::Dialog::new(&mut open, "Modal", theme);
    }
    {
        let _sheet_module = super::sheet::Sheet::new(&mut open, "Sheet", theme);
    }
    {
        let _root_sheet = super::Sheet::new(&mut open, "Sheet", theme);
    }

    let _close_button: fn(&mut egui::Ui, crate::DbProTheme) -> egui::Response = super::close_icon_button;
    let _actions: fn(&mut egui::Ui, crate::DbProTheme, super::DialogActionLabels<'_>) -> (bool, bool) =
        super::dialog_actions;
}

fn run_sheet_frame(ctx: &egui::Context, input: egui::RawInput, open: &mut bool, theme: crate::DbProTheme) {
    let _ = crate::test_frame::frame(ctx, input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            super::sheet::Sheet::new(open, "Panel", theme).show(ui, |ui| {
                ui.label("content");
            });
        });
    });
}

fn left_click(pos: egui::Pos2) -> Vec<egui::Event> {
    vec![
        egui::Event::PointerMoved(pos),
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        },
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::default(),
        },
    ]
}

#[test]
fn sheet_dismisses_on_backdrop_click_but_not_on_panel_click() {
    let ctx = egui::Context::default();
    crate::DbProTheme::install_fonts(&ctx);
    let theme = crate::DbProTheme::dark();

    // Warm up until the sheet is fully open.
    let mut open = true;
    for _ in 0..12 {
        run_sheet_frame(&ctx, Default::default(), &mut open, theme);
    }
    assert!(open);

    // Click the dim area on the left — outside the right-side panel.
    run_sheet_frame(
        &ctx,
        egui::RawInput {
            events: left_click(egui::pos2(100.0, 400.0)),
            ..Default::default()
        },
        &mut open,
        theme,
    );
    assert!(!open, "backdrop click should dismiss the sheet");

    // Reopen, then click inside the panel — it must stay open. `content_rect`
    // is unbounded (10k) in this harness, so the 360pt sheet hugs the right edge;
    // click just left of it.
    open = true;
    for _ in 0..12 {
        run_sheet_frame(&ctx, Default::default(), &mut open, theme);
    }
    run_sheet_frame(
        &ctx,
        egui::RawInput {
            events: left_click(egui::pos2(9900.0, 400.0)),
            ..Default::default()
        },
        &mut open,
        theme,
    );
    assert!(open, "a click inside the sheet must not dismiss it");
}
