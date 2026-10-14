//! Tests for `../ui.rs` (moved out of the source file).
use super::*;
use crate::DbProTheme;

#[test]
fn inline_code_renders_accessible_response() {
    let theme = DbProTheme::light();
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let resp = InlineCode::new("SELECT 1;", theme).show(ui);
            assert!(resp.rect.width() > 0.0);
            assert!(resp.rect.height() > 0.0);
        });
    });
}

#[test]
fn code_block_renders_with_and_without_line_numbers() {
    let theme = DbProTheme::dark();
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let resp1 = CodeBlock::new("SELECT * FROM users;\nWHERE id = 1;", theme)
                .language("sql")
                .show(ui);
            assert!(resp1.rect.height() > 32.0);

            let resp2 = CodeBlock::new("const x = 42;", theme)
                .language("ts")
                .show_line_numbers(false)
                .show(ui);
            assert!(resp2.rect.height() > 32.0);
        });
    });
}
