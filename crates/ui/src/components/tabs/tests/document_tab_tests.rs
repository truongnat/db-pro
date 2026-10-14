//! Tests for `../document_tab.rs` (moved out of the source file).
use super::*;

#[test]
// cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
fn tab_padding_keeps_title_and_close_target_separate() {
    for title in [
        "Query 1",
        "Component Gallery",
        "A very long query document title that needs truncation",
    ] {
        let ctx = egui::Context::default();
        let theme = DbProTheme::light();
        DbProTheme::install_fonts(&ctx);
        theme.apply(&ctx);
        let output = crate::test_frame::frame(&ctx, egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                draw_document_tab(
                    ui,
                    theme,
                    DocumentTabItem {
                        selected: true,
                        icon: Icon::FileCode,
                        title,
                        unsaved: true,
                        show_close: true,
                    },
                    |_, _| {},
                );
            });
        });
        let tab = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect) if rect.fill == theme.surface_app => Some(rect.rect),
                _ => None,
            })
            .expect("selected tab background");
        assert_eq!(tab.height(), DOCUMENT_TAB_HEIGHT);
        assert!(tab.width() <= TAB_MAX_WIDTH);
        let label = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.text().starts_with(title.chars().next().unwrap()) => Some(text),
                _ => None,
            })
            .expect("tab title");
        assert_eq!(label.pos.x - tab.left(), TAB_TITLE_INSET);
        let close_left = tab.right() - TAB_PAD_X - TAB_CLOSE_SIZE;
        assert!(
            label.pos.x + label.galley.size().x + SPACE_MD <= close_left,
            "title overlaps dirty/close slot"
        );
        assert!(label.pos.y >= tab.top());
        assert!(label.pos.y + label.galley.size().y <= tab.bottom());
    }
}
