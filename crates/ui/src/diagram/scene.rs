use super::lod::ErLod;
use super::model::ErGraph;
use super::spatial::ErSpatialIndex;
use super::viewport::ErViewport;
use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct ErPerfMetrics {
    pub total_nodes: usize,
    pub visible_nodes: usize,
    pub total_edges: usize,
    pub visible_edges: usize,
    pub spatial_query_micros: u64,
    pub prepare_micros: u64,
}

#[derive(Debug, Clone)]
pub struct ErRenderScene {
    pub visible_nodes: Vec<usize>,
    pub visible_edges: Vec<usize>,
    pub lod: ErLod,
    pub viewport: ErViewport,
    pub metrics: ErPerfMetrics,
}

pub fn prepare_render_scene(
    graph: &ErGraph,
    spatial_index: &ErSpatialIndex,
    viewport: &ErViewport,
    screen_clip: egui::Rect,
    filter_set: Option<&[usize]>,
) -> ErRenderScene {
    let start_time = Instant::now();
    let lod = ErLod::from_zoom(viewport.zoom);
    let world_viewport = viewport.visible_world_rect(screen_clip, 32.0);

    let query_start = Instant::now();
    let candidate_nodes = spatial_index.query_nodes(world_viewport);
    let candidate_edges = spatial_index.query_edges(world_viewport);
    let spatial_query_micros = query_start.elapsed().as_micros() as u64;

    let visible_nodes: Vec<usize> = if let Some(filter) = filter_set {
        let filter_lookup: std::collections::HashSet<usize> = filter.iter().copied().collect();
        candidate_nodes
            .into_iter()
            .filter(|id| {
                filter_lookup.contains(id)
                    && graph
                        .nodes
                        .get(*id)
                        .is_some_and(|n| world_viewport.intersects(n.world_rect))
            })
            .collect()
    } else {
        candidate_nodes
            .into_iter()
            .filter(|id| {
                graph
                    .nodes
                    .get(*id)
                    .is_some_and(|n| world_viewport.intersects(n.world_rect))
            })
            .collect()
    };

    let visible_edges: Vec<usize> = if let Some(filter) = filter_set {
        let filter_lookup: std::collections::HashSet<usize> = filter.iter().copied().collect();
        candidate_edges
            .into_iter()
            .filter(|id| {
                if let Some(edge) = graph.edges.get(*id) {
                    filter_lookup.contains(&edge.source)
                        && filter_lookup.contains(&edge.target)
                        && world_viewport.intersects(edge.world_bbox)
                } else {
                    false
                }
            })
            .collect()
    } else {
        candidate_edges
            .into_iter()
            .filter(|id| {
                graph
                    .edges
                    .get(*id)
                    .is_some_and(|e| world_viewport.intersects(e.world_bbox))
            })
            .collect()
    };

    let prepare_micros = start_time.elapsed().as_micros() as u64;

    ErRenderScene {
        visible_nodes: visible_nodes.clone(),
        visible_edges: visible_edges.clone(),
        lod,
        viewport: *viewport,
        metrics: ErPerfMetrics {
            total_nodes: graph.nodes.len(),
            visible_nodes: visible_nodes.len(),
            total_edges: graph.edges.len(),
            visible_edges: visible_edges.len(),
            spatial_query_micros,
            prepare_micros,
        },
    }
}
