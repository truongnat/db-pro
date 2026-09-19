use super::*;

/// Presentation and worker state for the ER diagram surface.
pub(crate) struct DiagramState {
    pub(crate) zoom: f32,
    pub(crate) pan: egui::Vec2,
    pub(crate) pan_origin: Option<egui::Vec2>,
    pub(crate) search: String,
    pub(crate) show_all: bool,
    pub(crate) design: crate::diagram::design_mode::DesignModeState,
    pub(crate) new_table: String,
    pub(crate) new_schema: String,
    pub(crate) column_name: String,
    pub(crate) column_type: String,
    pub(crate) foreign_key_name: String,
    pub(crate) foreign_key_from: String,
    pub(crate) foreign_key_to: String,
    pub(crate) neighborhood_depth: usize,
    pub(crate) graph: ErGraph,
    pub(crate) spatial_index: ErSpatialIndex,
    pub(crate) schema_version: u64,
    pub(crate) layout_worker: crate::diagram::ErLayoutWorker,
    pub(crate) layout_state: crate::diagram::ErLayoutState,
    pub(crate) latest_layout_request: u64,
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
