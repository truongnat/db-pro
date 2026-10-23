use super::*;

/// Keyed cache of the live-catalog search pass (#544). The answer depends on
/// exactly three inputs — the catalog generation (`schema_version`), the table
/// count, and the normalized query — so validity is decided by comparing those
/// on use, never by manual invalidation: any mismatch is a miss and recomputes.
/// Within one `(generation, count)` pair the table list is stable — the same
/// invariant `ensure_diagram_graph` already uses to decide the graph needs no
/// rebuild.
#[derive(Debug, Default)]
pub(crate) struct ErMatchCountCache {
    generation: u64,
    table_count: usize,
    query: String,
    matched_tables: usize,
    /// Recompute count — evidence for the "no per-frame rescanning" acceptance (#544).
    recomputes: u64,
}

impl ErMatchCountCache {
    /// Number of live tables matching `query` (ASCII-lowercased), rescanned
    /// only when the key changed.
    pub(crate) fn matched_tables(
        &mut self,
        tables: &[UiTableSummary],
        generation: u64,
        query: &str,
    ) -> usize {
        if self.generation != generation || self.table_count != tables.len() || self.query != query {
            self.matched_tables = tables
                .iter()
                .filter(|table| matches_diagram_search(table, query))
                .count();
            self.generation = generation;
            self.table_count = tables.len();
            self.query = query.to_owned();
            self.recomputes += 1;
        }
        self.matched_tables
    }

    #[cfg(test)]
    pub(crate) fn recomputes(&self) -> u64 {
        self.recomputes
    }
}

/// Keyed cache of the graph seed scan + BFS neighborhood pass (#544). Keyed by
/// the graph's identity (`schema_version` + node count — every committed graph
/// gets a fresh generation, and manual-layout resets install an empty graph),
/// plus the query and hop depth the BFS was run at. The BFS result is a set of
/// node ids, so persisted position overrides do not affect validity.
#[derive(Debug, Default)]
pub(crate) struct ErSubsetCache {
    graph_version: u64,
    node_count: usize,
    query: String,
    neighborhood_depth: usize,
    seeds: Vec<usize>,
    subset: Vec<usize>,
    /// Recompute count — evidence for the "no per-frame rescanning" acceptance (#544).
    recomputes: u64,
}

/// Borrowed view of one cached subset resolution.
pub(crate) struct ErSubsetMatch<'a> {
    pub(super) seeds: &'a [usize],
    pub(super) subset: &'a [usize],
}

impl ErSubsetCache {
    /// Seeds + BFS neighborhood for `(graph, query, depth)`, recomputed only on
    /// key change. `empty_seed_cap`/`match_seed_cap` mirror the two caps the
    /// uncached pass used (40 for the no-query neighborhood, 100 for matches).
    pub(crate) fn resolve(
        &mut self,
        graph: &ErGraph,
        query: &str,
        neighborhood_depth: usize,
    ) -> ErSubsetMatch<'_> {
        if self.graph_version != graph.schema_version
            || self.node_count != graph.nodes.len()
            || self.query != query
            || self.neighborhood_depth != neighborhood_depth
        {
            let (seeds, cap): (Vec<usize>, usize) = if query.is_empty() {
                (
                    graph.nodes.iter().take(3).map(|node| node.id).collect(),
                    40,
                )
            } else {
                (
                    graph
                        .nodes
                        .iter()
                        .filter(|node| matches_diagram_search(&node.table, query))
                        .map(|node| node.id)
                        .collect(),
                    100,
                )
            };
            let subset = graph.bfs_neighborhood(&seeds, (neighborhood_depth, cap));
            self.graph_version = graph.schema_version;
            self.node_count = graph.nodes.len();
            self.query = query.to_owned();
            self.neighborhood_depth = neighborhood_depth;
            self.seeds = seeds;
            self.subset = subset;
            self.recomputes += 1;
        }
        ErSubsetMatch {
            seeds: &self.seeds,
            subset: &self.subset,
        }
    }

    #[cfg(test)]
    pub(crate) fn recomputes(&self) -> u64 {
        self.recomputes
    }
}

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
    /// The zoom/pan the auto-fit produced. A rebuilt graph refits the viewport
    /// only while it still equals this value: once the user zooms or pans, the
    /// cards can change height (the LOD follows the zoom) without stealing
    /// their view.
    pub(super) auto_fit_viewport: Option<(f32, egui::Vec2)>,
    /// `true` once this session has written the live viewport back into the
    /// connection's snapshot, i.e. the stored zoom/pan is no longer the value
    /// loaded from storage. Only the loaded value may be refitted for being a
    /// name-strip view — an in-session viewport is the user's.
    pub(super) snapshot_live: bool,
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
    /// #544: per-key caches for the search passes — live-catalog match count
    /// and graph seed/BFS subset — so redraws stop rescanning 39k names.
    pub(super) match_count_cache: ErMatchCountCache,
    pub(super) subset_cache: ErSubsetCache,
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
            auto_fit_viewport: None,
            snapshot_live: false,
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
            match_count_cache: ErMatchCountCache::default(),
            subset_cache: ErSubsetCache::default(),
            layout_worker: crate::diagram::ErLayoutWorker::default(),
            layout_state: crate::diagram::ErLayoutState::Idle,
            latest_layout_request: 0,
        }
    }
}

#[cfg(test)]
#[path = "tests/diagram_state_tests.rs"]
mod tests;

