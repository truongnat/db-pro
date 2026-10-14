use super::*;

/// Presentation and worker state for the ER diagram surface.
pub(crate) struct DiagramState {
    pub(super) zoom: f32,
    pub(super) pan: egui::Vec2,
    /// Canvas pan currently in progress: the pan value at grab plus the screen
    /// position where the press landed. `drag_delta` is per-frame in egui, so
    /// panning is recomputed from the press point, not accumulated.
    pub(super) pan_origin: Option<(egui::Vec2, egui::Pos2)>,
    /// Node currently dragged by the pointer, plus the grab offset between the
    /// pointer (world space) and the node's top-left corner.
    pub(super) drag_node: Option<(usize, egui::Vec2)>,
    /// Node under the pointer this frame — drives edge highlighting and the
    /// cursor affordance.
    pub(super) hovered_node: Option<usize>,
    /// Node pinned by a single click — keeps its relationships highlighted
    /// until the user clicks empty canvas or the graph is rebuilt. Double
    /// click opens the table; single click no longer does.
    pub(super) selected_node: Option<usize>,
    /// `true` once the initial fit-to-viewport (or a restored snapshot) ran for
    /// the current graph; reset whenever the graph is rebuilt.
    pub(super) auto_fit_done: bool,
    /// Capture-only knob (`DB_PRO_CAPTURE_DIAGRAM_ZOOM`): after the initial fit,
    /// re-center on the world at this zoom so evidence shots can document the
    /// Standard/Detailed levels of detail deterministically.
    pub(super) capture_zoom_override: Option<f32>,
    /// Timestamp of the last search edit — the canvas fits the result subset
    /// once typing pauses instead of jumping per keystroke.
    pub(super) search_changed_at: Option<std::time::Instant>,
    /// Last search query that a fit was performed for.
    pub(super) fitted_search: String,
    /// Per-connection persisted layouts (`connection_id` → snapshot), hydrated
    /// from `dbpro.native.er-layouts-v1` and written back on app save.
    pub(super) saved_layouts: std::collections::HashMap<String, crate::diagram::ErLayoutSnapshot>,
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
            drag_node: None,
            hovered_node: None,
            selected_node: None,
            auto_fit_done: false,
            capture_zoom_override: None,
            search_changed_at: None,
            fitted_search: String::new(),
            saved_layouts: std::collections::HashMap::new(),
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
