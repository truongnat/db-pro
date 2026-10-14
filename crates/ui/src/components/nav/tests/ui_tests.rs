//! Tests for `../ui.rs` (moved out of the source file).
use super::*;

fn run_ui(mut on_ui: impl FnMut(&mut egui::Ui)) {
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| on_ui(ui));
    });
}

#[test]
fn pagination_renders_and_updates_page_state() {
    let theme = DbProTheme::light();
    let mut page = 2;
    run_ui(|ui| {
        let resp = Pagination::new(&mut page, 5, theme).show(ui);
        assert!(resp.rect.is_finite());
        assert!(resp.rect.width() > 0.0);
    });
    assert_eq!(page, 2);
}

#[test]
fn breadcrumbs_render_items() {
    let theme = DbProTheme::light();
    let items = [
        BreadcrumbItem::new("Databases"),
        BreadcrumbItem::new("Production").current(true),
    ];
    run_ui(|ui| {
        let clicked = Breadcrumb::new(&items, theme).show(ui);
        assert_eq!(clicked, None);
    });
}

#[test]
fn page_and_section_headers_render_with_descriptions() {
    let theme = DbProTheme::light();
    run_ui(|ui| {
        let page_h = PageHeader::new("Users", theme)
            .description("Manage system users")
            .show(ui);
        assert!(page_h.rect.is_finite());

        let section_h = SectionHeader::new("Permissions", theme)
            .description("Role assignments")
            .show(ui);
        assert!(section_h.rect.is_finite());
    });
}
