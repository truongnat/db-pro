//! Query workspace panel geometry.
use super::super::*;

pub(super) struct QueryPanelLayoutContext {
    pub(super) available_height: f32,
    pub(super) bottom_panel_open: bool,
    pub(super) output_dock_maximized: bool,
    pub(super) bottom_panel_height: f32,
}

pub(super) struct QueryPanelLayout {
    pub(super) dock_open: bool,
    pub(super) dock_height: f32,
    pub(super) editor_height: f32,
}

pub(super) fn calculate(context: QueryPanelLayoutContext) -> QueryPanelLayout {
    let status_height = query_status_bar_surface_view::QUERY_STATUS_HEIGHT;
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
        dock_height,
        editor_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_dock_keeps_editor_above_minimum_height() {
        let layout = calculate(QueryPanelLayoutContext {
            available_height: 400.0,
            bottom_panel_open: false,
            output_dock_maximized: false,
            bottom_panel_height: 500.0,
        });
        assert!(!layout.dock_open);
        assert_eq!(layout.dock_height, 0.0);
        assert!(layout.editor_height >= 120.0);
    }

    #[test]
    fn maximized_dock_reserves_compact_editor_strip() {
        let layout = calculate(QueryPanelLayoutContext {
            available_height: 900.0,
            bottom_panel_open: true,
            output_dock_maximized: true,
            bottom_panel_height: 320.0,
        });
        assert_eq!(layout.editor_height, 80.0);
        assert!(layout.dock_height >= OUTPUT_MIN_HEIGHT);
    }
}
