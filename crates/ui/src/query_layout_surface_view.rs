//! Query workspace panel geometry.
use super::super::*;

pub(super) struct QueryPanelLayoutContext {
    pub(super) available_width: f32,
    pub(super) available_height: f32,
    pub(super) bottom_panel_open: bool,
    pub(super) output_dock_maximized: bool,
    pub(super) bottom_panel_height: f32,
    pub(super) right_dock_width: f32,
    pub(super) dock_position: OutputDockPosition,
}

pub(super) struct QueryPanelLayout {
    pub(super) dock_open: bool,
    pub(super) dock_position: OutputDockPosition,
    pub(super) dock_height: f32,
    pub(super) dock_width: f32,
    pub(super) editor_height: f32,
    pub(super) editor_width: f32,
}

pub(super) fn calculate(context: QueryPanelLayoutContext) -> QueryPanelLayout {
    let status_height = query_status_bar_surface_view::QUERY_STATUS_HEIGHT;
    match context.dock_position {
        OutputDockPosition::Bottom => {
            let dock_height = if !context.bottom_panel_open {
                0.0
            } else if context.output_dock_maximized {
                (context.available_height - status_height - 80.0).max(OUTPUT_MIN_HEIGHT)
            } else {
                context.bottom_panel_height.clamp(OUTPUT_MIN_HEIGHT, OUTPUT_MAX_HEIGHT)
            };
            let editor_height = if context.output_dock_maximized && context.bottom_panel_open {
                80.0
            } else {
                (context.available_height - dock_height - status_height).max(120.0)
            };
            QueryPanelLayout {
                dock_open: context.bottom_panel_open,
                dock_position: OutputDockPosition::Bottom,
                dock_height,
                dock_width: context.available_width,
                editor_height,
                editor_width: context.available_width,
            }
        }
        OutputDockPosition::Right => {
            let full_height = (context.available_height - status_height).max(120.0);
            let dock_width = if !context.bottom_panel_open {
                0.0
            } else if context.output_dock_maximized {
                (context.available_width - 120.0).max(280.0)
            } else {
                context.right_dock_width.clamp(240.0, (context.available_width - 160.0).max(240.0))
            };
            let editor_width = if context.output_dock_maximized && context.bottom_panel_open {
                120.0
            } else {
                (context.available_width - dock_width).max(160.0)
            };
            QueryPanelLayout {
                dock_open: context.bottom_panel_open,
                dock_position: OutputDockPosition::Right,
                dock_height: full_height,
                dock_width,
                editor_height: full_height,
                editor_width,
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
}
