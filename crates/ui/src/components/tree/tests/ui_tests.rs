//! Tests for `../ui.rs` (moved out of the source file).
use super::*;

#[test]
fn tree_icons_layout() {
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        let font = FontId::new(NODE_ICON_FONT_SIZE, FontFamily::Name("lucide".into()));
        for kind in [
            TreeNodeKind::Server,
            TreeNodeKind::Database,
            TreeNodeKind::Schema,
            TreeNodeKind::Table,
            TreeNodeKind::View,
            TreeNodeKind::Column,
            TreeNodeKind::PrimaryKey,
            TreeNodeKind::ForeignKey,
            TreeNodeKind::Index,
        ] {
            let galley = ctx.fonts_mut(|fonts| {
                fonts.layout_no_wrap(char::from(kind.icon()).to_string(), font.clone(), Color32::WHITE)
            });
            assert!(!galley.rows.is_empty());
            assert!(galley.size().x > 0.0);
        }
    });
}
