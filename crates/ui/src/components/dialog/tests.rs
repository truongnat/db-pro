use super::config::*;
use super::layout::screen_rect_fallback;

#[test]
fn dialog_constants_follow_design_tokens() {
    assert_eq!(DIALOG_RADIUS, 16.0);
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
