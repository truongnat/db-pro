//! Tests for `../query_layout_surface_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn closed_dock_keeps_editor_above_minimum_height() {
        let layout = calculate(QueryPanelLayoutContext {
            available_width: 800.0,
            available_height: 400.0,
            bottom_panel_open: false,
            output_dock_maximized: false,
            bottom_panel_height: 500.0,
            right_dock_width: 400.0,
            dock_position: OutputDockPosition::Bottom,
        });
        assert!(!layout.dock_open);
        assert_eq!(layout.dock_height, 0.0);
        assert!(layout.editor_height >= 120.0);
    }

    #[test]
    fn maximized_dock_reserves_compact_editor_strip() {
        let layout = calculate(QueryPanelLayoutContext {
            available_width: 1000.0,
            available_height: 900.0,
            bottom_panel_open: true,
            output_dock_maximized: true,
            bottom_panel_height: 320.0,
            right_dock_width: 400.0,
            dock_position: OutputDockPosition::Bottom,
        });
        assert_eq!(layout.editor_height, 80.0);
        assert!(layout.dock_height >= OUTPUT_MIN_HEIGHT);
    }

    #[test]
    fn right_dock_calculates_horizontal_widths() {
        let layout = calculate(QueryPanelLayoutContext {
            available_width: 1200.0,
            available_height: 800.0,
            bottom_panel_open: true,
            output_dock_maximized: false,
            bottom_panel_height: 300.0,
            right_dock_width: 500.0,
            dock_position: OutputDockPosition::Right,
        });
        assert!(layout.dock_open);
        assert_eq!(layout.dock_position, OutputDockPosition::Right);
        assert_eq!(layout.dock_width, 500.0);
        assert_eq!(layout.editor_width, 700.0);
    }