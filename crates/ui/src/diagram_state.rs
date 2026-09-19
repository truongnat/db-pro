use super::*;

/// Presentation and worker state for the ER diagram surface.
pub(crate) struct DiagramState {
    pub(super) zoom: f32,
    pub(super) pan: egui::Vec2,
    pub(super) pan_origin: Option<egui::Vec2>,
    pub(super) search: String,
    pub(super) show_all: bool,
    pub(super) design: crate::diagram::design_mode::DesignModeState,
    pub(super) new_table: String,
    pub(super) new_schema: String,
    pub(super) column_name: String,
    pub(super) column_type: String,
    pub(super) foreign_key_name: String,
    pub(super) foreign_key_from: String,
    pub(super) foreign_key_to: String,
    pub(super) neighborhood_depth: usize,
    pub(super) graph: ErGraph,
    pub(super) spatial_index: ErSpatialIndex,
    pub(super) schema_version: u64,
    pub(super) layout_worker: crate::diagram::ErLayoutWorker,
    pub(super) layout_state: crate::diagram::ErLayoutState,
    pub(super) latest_layout_request: u64,
}

impl Default for DiagramState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            pan_origin: None,
            search: String::new(),
            show_all: false,
            design: crate::diagram::design_mode::DesignModeState::default(),
            new_table: String::new(),
            new_schema: "public".to_owned(),
            column_name: String::new(),
            column_type: "text".to_owned(),
            foreign_key_name: String::new(),
            foreign_key_from: String::new(),
            foreign_key_to: String::new(),
            neighborhood_depth: 1,
            graph: ErGraph::default(),
            spatial_index: ErSpatialIndex::default(),
            // Start at one so a stale worker result cannot match the initial state.
            schema_version: 1,
            layout_worker: crate::diagram::ErLayoutWorker::default(),
            layout_state: crate::diagram::ErLayoutState::Idle,
            latest_layout_request: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DiagramState;

    #[test]
    fn default_diagram_state_starts_at_safe_viewport_defaults() {
        let state = DiagramState::default();

        assert_eq!(state.zoom, 1.0);
        assert_eq!(state.neighborhood_depth, 1);
        assert_eq!(state.schema_version, 1);
        assert!(state.graph.nodes.is_empty());
        assert!(state.search.is_empty());
    }
}
