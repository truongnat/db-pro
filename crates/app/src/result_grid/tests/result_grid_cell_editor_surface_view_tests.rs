//! Tests for `../result_grid_cell_editor_surface_view.rs` (moved out of the source file).
    use super::*;
    use egui::{Pos2, RawInput, Sense, Vec2};

    /// The editor draws inside a row's horizontal layout; its allocated
    /// footprint must stay within the 28pt cell or the whole row resizes.
    fn row_height_for(kind: CellEditorKind) -> f32 {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let mut value = "2026-09-30 14:22:11".to_owned();
        let mut error = None;
        let mut height = 0.0;
        let _ = crate::test_frame::frame(&ctx, RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 300.0))),
            ..Default::default()
        }, |ui| {
            height = egui::CentralPanel::default()
                .show(ui, |ui| {
                    let row = ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                        ui.allocate_exact_size(Vec2::new(40.0, 28.0), Sense::hover());
                        let (cell_rect, _) = ui.allocate_exact_size(Vec2::new(220.0, 28.0), Sense::hover());
                        let mut context = CellEditorContext {
                            theme: DbProTheme::dark(),
                            kind: kind.clone(),
                            value: &mut value,
                            error: &mut error,
                        };
                        draw(&mut context, ui, cell_rect);
                        ui.allocate_exact_size(Vec2::new(120.0, 28.0), Sense::hover());
                    });
                    row.response.rect.height()
                })
                .inner;
        });
        height
    }

    #[test]
    fn temporal_editor_keeps_the_row_height() {
        let height = row_height_for(CellEditorKind::Temporal { date_only: false });
        assert!(height <= 28.5, "temporal editor grew the row to {height}");
    }

    #[test]
    fn select_editor_keeps_the_row_height() {
        let height = row_height_for(CellEditorKind::Select(vec!["pending".to_owned()]));
        assert!(height <= 28.5, "select editor grew the row to {height}");
    }

    #[test]
    fn text_editor_keeps_the_row_height() {
        let height = row_height_for(CellEditorKind::Text);
        assert!(height <= 28.5, "text editor grew the row to {height}");
    }

    /// The calendar button must actually open the picker: a click inside the
    // cell must not be treated as a "click outside" that closes the popup in
    /// the same frame it was toggled open.
    #[test]
    fn calendar_button_opens_the_picker() {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let mut value = "2026-09-30 14:22:11".to_owned();
        let mut error = None;
        let screen = egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 300.0));
        let cell = egui::Rect::from_min_size(Pos2::new(60.0, 0.0), Vec2::new(220.0, 28.0));
        // The calendar button occupies the trailing 24px of the editor.
        let click = Pos2::new(cell.max.x - 12.0, 14.0);
        // A warm-up frame registers the widget rects; pointer containment is
        // resolved from last frame's rects, so the first frame cannot click.
        for event in [None, Some(true), Some(false)] {
            let events = match event {
                None => vec![],
                Some(pressed) => vec![
                    egui::Event::PointerMoved(click),
                    egui::Event::PointerButton {
                        pos: click,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    },
                ],
            };
            let _ = crate::test_frame::frame(&ctx, RawInput {
                screen_rect: Some(screen),
                events,
                ..Default::default()
            }, |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    let mut context = CellEditorContext {
                        theme: DbProTheme::dark(),
                        kind: CellEditorKind::Temporal { date_only: false },
                        value: &mut value,
                        error: &mut error,
                    };
                    draw(&mut context, ui, cell);
                });
            });
        }
        assert!(
            egui::Popup::is_any_open(&ctx),
            "clicking the calendar button did not open the picker"
        );
    }