//! Tests for `../diagram_canvas_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn dragging_canvas_updates_pan() {
        let context = egui::Context::default();
        DbProTheme::install_fonts(&context);
        let explorer = SchemaExplorerState::default();
        let mut diagram = DiagramState::default();
        let start = egui::pos2(160.0, 160.0);
        run_canvas_frame(&context, &explorer, &mut diagram, vec![egui::Event::PointerMoved(start)]);
        run_canvas_frame(&context, &explorer, &mut diagram, vec![press_event(start)]);
        // Two separate move frames: a per-frame delta accumulation bug would
        // leave pan at the last frame's (10, 5) instead of the full (74, 53).
        run_canvas_frame(&context, &explorer, &mut diagram, vec![moved_to(start + egui::vec2(64.0, 48.0))]);
        run_canvas_frame(&context, &explorer, &mut diagram, vec![moved_to(start + egui::vec2(74.0, 53.0))]);
        run_canvas_frame(&context, &explorer, &mut diagram, vec![release_event(start + egui::vec2(74.0, 53.0))]);

        assert_eq!(
            diagram.pan,
            egui::vec2(74.0, 53.0),
            "pan must track the full distance from the press point, not per-frame motion"
        );
    }

    fn run_canvas_frame(
        context: &egui::Context,
        explorer: &SchemaExplorerState,
        diagram: &mut DiagramState,
        events: Vec<egui::Event>,
    ) {
        let mut view = DiagramViewContext {
            theme: DbProTheme::dark(),
            diagram,
            explorer,
            active_driver: "SQLite",
            connected: true,
            connection_id: None,
        };
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900.0, 600.0));
        let _ = crate::test_frame::frame(&context, egui::RawInput {
            screen_rect: Some(screen),
            events,
            ..Default::default()
        }, |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                let (response, _) =
                    ui.allocate_painter(egui::vec2(880.0, 560.0), egui::Sense::click_and_drag());
                let viewport = ErViewport::new(view.diagram.pan, view.diagram.zoom, response.rect.min);
                handle_drag_input(&mut view, &response, viewport, &[]);
            });
        });
    }

    fn moved_to(pos: egui::Pos2) -> egui::Event {
        egui::Event::PointerMoved(pos)
    }

    fn press_event(pos: egui::Pos2) -> egui::Event {
        button_event(pos, true)
    }

    fn release_event(pos: egui::Pos2) -> egui::Event {
        button_event(pos, false)
    }

    fn button_event(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }

    /// Four unlinked tables in two grid columns at the Compact card height:
    /// 740 x 252 world units, small enough to fit above the column threshold.
    fn compact_graph() -> ErGraph {
        let table = |name: &str| UiTableSummary {
            schema: "main".to_owned(),
            name: name.to_owned(),
            row_count: None,
            columns: vec![crate::runtime::UiSchemaColumn {
                name: "id".to_owned(),
                data_type: "INTEGER".to_owned(),
                nullable: false,
                is_primary_key: true,
            }],
            foreign_keys: Vec::new(),
        };
        let tables = vec![table("customers"), table("orders"), table("payments"), table("reviews")];
        ErGraph::build(&tables, 1, (2usize, 40.0))
    }

    fn compact_diagram() -> DiagramState {
        let graph = compact_graph();
        DiagramState {
            spatial_index: ErSpatialIndex::build(&graph.nodes, &graph.edges, DEFAULT_SPATIAL_CELL_SIZE),
            graph,
            ..DiagramState::default()
        }
    }

    fn run_canvas_draw(
        context: &egui::Context,
        explorer: &SchemaExplorerState,
        diagram: &mut DiagramState,
        connection_id: Option<&str>,
    ) {
        let mut view = DiagramViewContext {
            theme: DbProTheme::dark(),
            diagram,
            explorer,
            active_driver: "SQLite",
            connected: true,
            connection_id: connection_id.map(str::to_owned),
        };
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900.0, 600.0));
        let _ = crate::test_frame::frame(
            context,
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    let _ = draw_diagram_canvas(&mut view, ui, None);
                });
            },
        );
    }

    #[test]
    fn refit_never_drops_a_level_of_detail() {
        // Climbing is always safe.
        assert!(refit_keeps_detail(0.30, 0.94));
        // Staying in the band is what a settled diagram does.
        assert!(refit_keeps_detail(0.70, 0.72));
        // Dropping to Compact would rebuild shorter cards, which fit back to a
        // higher zoom and would refit each other forever.
        assert!(!refit_keeps_detail(0.94, 0.43));
        assert!(!refit_keeps_detail(1.20, 0.80));
    }

    #[test]
    fn stale_strip_zoom_is_refitted() {
        // 0.44 renders name-only cards; the bounds fit at 0.94.
        assert!(strip_zoom_is_stale(0.44, 0.94));
        // A viewport that already shows columns is kept.
        assert!(!strip_zoom_is_stale(0.94, 0.94));
        // Nothing better to offer when the bounds cannot show columns either.
        assert!(!strip_zoom_is_stale(0.44, 0.30));
    }

    #[test]
    fn committed_bounds_refit_a_stale_fit() {
        // The pre-fix state: a fit taken against the taller Standard build,
        // which the Compact cards committed afterwards. The viewport is still
        // the fit's own output, so it is refitted against the current bounds
        // instead of being restored as if the user had chosen it.
        let context = egui::Context::default();
        DbProTheme::install_fonts(&context);
        let explorer = SchemaExplorerState::default();
        let mut diagram = compact_diagram();
        diagram.zoom = 0.44;
        diagram.pan = egui::vec2(-120.0, -40.0);
        diagram.auto_fit_viewport = Some((diagram.zoom, diagram.pan));
        diagram.auto_fit_done = false;

        run_canvas_draw(&context, &explorer, &mut diagram, None);

        assert!(
            diagram.zoom > 0.45,
            "a committed layout must refit above the Compact threshold, got {}",
            diagram.zoom
        );
        assert_eq!(diagram.auto_fit_viewport, Some((diagram.zoom, diagram.pan)));
    }

    #[test]
    fn user_viewport_survives_a_layout_commit() {
        // Same rebuild, but the user zoomed after the last fit: the marker no
        // longer matches, so the stored viewport is restored untouched.
        let context = egui::Context::default();
        DbProTheme::install_fonts(&context);
        let explorer = SchemaExplorerState::default();
        let mut diagram = compact_diagram();
        diagram.zoom = 0.30;
        diagram.pan = egui::vec2(12.0, 24.0);
        diagram.auto_fit_viewport = Some((0.9, egui::Vec2::ZERO));
        diagram.snapshot_live = true;
        diagram.saved_layouts.insert(
            "conn".to_owned(),
            ErLayoutSnapshot {
                positions: Default::default(),
                zoom: Some(diagram.zoom),
                pan: Some([diagram.pan.x, diagram.pan.y]),
            },
        );
        diagram.auto_fit_done = false;

        run_canvas_draw(&context, &explorer, &mut diagram, Some("conn"));

        assert_eq!(diagram.zoom, 0.30, "a zoom the user chose must survive the rebuild");
        assert_eq!(diagram.pan, egui::vec2(12.0, 24.0));
    }
