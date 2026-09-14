use super::model::{ErEdge, ErNode};
use std::collections::{HashMap, HashSet};

pub const DEFAULT_SPATIAL_CELL_SIZE: f32 = 256.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErSpatialMetrics {
    pub node_bucket_count: usize,
    pub edge_bucket_count: usize,
    pub node_references: usize,
    pub edge_references: usize,
    pub max_node_bucket_size: usize,
    pub max_edge_bucket_size: usize,
}

#[derive(Debug, Clone)]
pub struct ErSpatialIndex {
    pub cell_size: f32,
    pub node_cells: HashMap<(i32, i32), Vec<usize>>,
    pub edge_cells: HashMap<(i32, i32), Vec<usize>>,
}

impl Default for ErSpatialIndex {
    fn default() -> Self {
        Self::new(DEFAULT_SPATIAL_CELL_SIZE)
    }
}

impl ErSpatialIndex {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size: cell_size.max(32.0),
            node_cells: HashMap::new(),
            edge_cells: HashMap::new(),
        }
    }

    pub fn build(nodes: &[ErNode], edges: &[ErEdge], cell_size: f32) -> Self {
        let mut index = Self::new(cell_size);
        for node in nodes {
            index.insert_node(node.id, node.world_rect);
        }
        for edge in edges {
            index.insert_edge(edge.id, edge.world_bbox);
        }
        for node_list in index.node_cells.values_mut() {
            node_list.sort_unstable();
            node_list.dedup();
        }
        for edge_list in index.edge_cells.values_mut() {
            edge_list.sort_unstable();
            edge_list.dedup();
        }
        index
    }

    pub fn metrics(&self) -> ErSpatialMetrics {
        let node_references = self.node_cells.values().map(|v| v.len()).sum();
        let edge_references = self.edge_cells.values().map(|v| v.len()).sum();
        let max_node_bucket_size = self.node_cells.values().map(|v| v.len()).max().unwrap_or(0);
        let max_edge_bucket_size = self.edge_cells.values().map(|v| v.len()).max().unwrap_or(0);

        ErSpatialMetrics {
            node_bucket_count: self.node_cells.len(),
            edge_bucket_count: self.edge_cells.len(),
            node_references,
            edge_references,
            max_node_bucket_size,
            max_edge_bucket_size,
        }
    }

    fn cell_range(&self, rect: egui::Rect) -> (i32, i32, i32, i32) {
        let min_x = (rect.min.x / self.cell_size).floor() as i32;
        let max_x = (rect.max.x / self.cell_size).floor() as i32;
        let min_y = (rect.min.y / self.cell_size).floor() as i32;
        let max_y = (rect.max.y / self.cell_size).floor() as i32;
        (min_x, max_x, min_y, max_y)
    }

    pub fn insert_node(&mut self, node_id: usize, rect: egui::Rect) {
        let (min_x, max_x, min_y, max_y) = self.cell_range(rect);
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                self.node_cells.entry((x, y)).or_default().push(node_id);
            }
        }
    }

    pub fn insert_edge(&mut self, edge_id: usize, bbox: egui::Rect) {
        let (min_x, max_x, min_y, max_y) = self.cell_range(bbox);
        // Bound the cell expansion for pathological long edges
        let max_span = 32;
        let bounded_max_x = max_x.min(min_x + max_span);
        let bounded_max_y = max_y.min(min_y + max_span);
        for x in min_x..=bounded_max_x {
            for y in min_y..=bounded_max_y {
                self.edge_cells.entry((x, y)).or_default().push(edge_id);
            }
        }
    }

    pub fn query_nodes(&self, world_rect: egui::Rect) -> Vec<usize> {
        let (min_x, max_x, min_y, max_y) = self.cell_range(world_rect);
        let mut seen = HashSet::new();
        let mut results = Vec::new();
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                if let Some(cell_nodes) = self.node_cells.get(&(x, y)) {
                    for &id in cell_nodes {
                        if seen.insert(id) {
                            results.push(id);
                        }
                    }
                }
            }
        }
        results
    }

    pub fn query_edges(&self, world_rect: egui::Rect) -> Vec<usize> {
        let (min_x, max_x, min_y, max_y) = self.cell_range(world_rect);
        let mut seen = HashSet::new();
        let mut results = Vec::new();
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                if let Some(cell_edges) = self.edge_cells.get(&(x, y)) {
                    for &id in cell_edges {
                        if seen.insert(id) {
                            results.push(id);
                        }
                    }
                }
            }
        }
        results
    }

    pub fn hit_test_node(&self, world_pos: egui::Pos2, nodes: &[ErNode]) -> Option<usize> {
        let query_rect = egui::Rect::from_center_size(world_pos, egui::vec2(4.0, 4.0));
        let candidates = self.query_nodes(query_rect);
        candidates.into_iter().find(|&id| {
            if let Some(node) = nodes.get(id) {
                node.world_rect.contains(world_pos)
            } else {
                false
            }
        })
    }
}
