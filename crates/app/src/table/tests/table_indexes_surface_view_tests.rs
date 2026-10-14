//! Tests for `../table_indexes_surface_view.rs` (moved out of the source file).
    use super::*;
    use egui::{CentralPanel, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};

    fn test_index() -> UiTableIndex {
        UiTableIndex {
            name: "users_email_idx".to_owned(),
            columns: vec!["email".to_owned()],
            unique: true,
            method: "btree".to_owned(),
            primary: false,
            include_columns: Vec::new(),
            predicate: None,
            definition: "CREATE UNIQUE INDEX users_email_idx ON users (email)".to_owned(),
        }
    }

    #[test]
    fn index_name_does_not_activate_on_click() {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let index = test_index();
        let theme = DbProTheme::dark();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(420.0, 120.0));
        let mut label_rect = None;
        let _ = crate::test_frame::frame(&ctx, RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        }, |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                label_rect = Some(index_name_label(ui, &index, theme).rect);
            });
        });
        let click_pos = label_rect.expect("index label is laid out").center();

        let mut clicked = false;
        for pressed in [true, false] {
            let _ = crate::test_frame::frame(&ctx, RawInput {
                screen_rect: Some(screen),
                events: vec![Event::PointerButton {
                    pos: click_pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::default(),
                }],
                ..Default::default()
            }, |ctx| {
                CentralPanel::default().show(ctx, |ui| {
                    clicked = index_name_label(ui, &index, theme).clicked();
                });
            });
        }
        assert!(!clicked, "the index name is metadata, not a modal action");
    }

    #[test]
    fn index_columns_fit_the_standard_table_detail_viewport() {
        let available_width = 928.0;
        let columns = index_columns(available_width);
        let requested_width = columns.iter().map(|column| column.width.unwrap_or(80.0)).sum::<f32>();

        assert!(
            requested_width <= available_width,
            "index columns request {requested_width} pt from a {available_width} pt viewport"
        );
        assert!(
            columns[5].width.unwrap_or_default() >= 150.0,
            "Status needs room for its labels"
        );
    }