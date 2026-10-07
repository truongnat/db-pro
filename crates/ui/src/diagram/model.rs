use crate::runtime::{UiSchemaForeignKey, UiTableSummary};
use std::collections::{HashMap, HashSet, VecDeque};

/// Canonical storage key for one table node inside a persisted layout snapshot.
pub fn er_table_key(schema: &str, name: &str) -> String {
    format!("{schema}.{name}")
}

pub const ER_NODE_WIDTH: f32 = 280.0;
pub const ER_HEADER_HEIGHT: f32 = 40.0;
pub const ER_ROW_HEIGHT: f32 = 24.0;
pub const ER_GAP_X: f32 = 84.0;
pub const ER_GAP_Y: f32 = 76.0;
pub const ER_CANVAS_MARGIN: f32 = 48.0;
pub const ER_MAX_COLUMNS: usize = 8;
pub const ER_MAX_TABLES: usize = 5;
pub const ER_MAX_EDGES: usize = 6;
pub const ER_LARGE_SCHEMA_THRESHOLD: usize = 200;
pub const ER_MIN_ZOOM: f32 = 0.1;
pub const ER_MAX_ZOOM: f32 = 2.0;

#[derive(Debug, Clone)]
pub struct ErNode {
    pub id: usize,
    pub table: UiTableSummary,
    pub world_rect: egui::Rect,
}

impl ErNode {
    pub fn column_anchor(&self, column_name: Option<&str>, right_side: bool, max_columns: usize) -> egui::Pos2 {
        let visible_columns = self.table.columns.len().min(max_columns);
        let column_index = column_name
            .and_then(|name| {
                self.table
                    .columns
                    .iter()
                    .take(visible_columns)
                    .position(|item| item.name == name)
            })
            .unwrap_or(0);
        let y = if visible_columns == 0 {
            self.world_rect.center().y
        } else {
            self.world_rect.top() + ER_HEADER_HEIGHT + ER_ROW_HEIGHT * (column_index as f32 + 0.5)
        };
        egui::pos2(
            if right_side {
                self.world_rect.right()
            } else {
                self.world_rect.left()
            },
            y,
        )
    }
}

#[derive(Debug, Clone)]
pub struct ErEdge {
    pub id: usize,
    pub source: usize,
    pub target: usize,
    pub foreign_key: UiSchemaForeignKey,
    pub world_bbox: egui::Rect,
}

#[derive(Debug, Clone)]
pub struct ErGraph {
    pub nodes: Vec<ErNode>,
    pub edges: Vec<ErEdge>,
    pub node_lookup: HashMap<(String, String), usize>,
    pub adjacency: HashMap<usize, Vec<usize>>,
    pub world_bounds: egui::Rect,
    pub schema_version: u64,
}

impl Default for ErGraph {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_lookup: HashMap::new(),
            adjacency: HashMap::new(),
            world_bounds: egui::Rect::ZERO,
            schema_version: 0,
        }
    }
}

impl ErGraph {
    pub fn build(tables: &[UiTableSummary], schema_version: u64, grid_columns: usize, node_height: f32) -> Self {
        let mut nodes = Vec::with_capacity(tables.len());
        let mut node_lookup = HashMap::with_capacity(tables.len());
        let mut adjacency: HashMap<usize, Vec<usize>> = HashMap::new();

        let effective_columns = grid_columns.max(1);
        for (index, table) in tables.iter().enumerate() {
            let column = index % effective_columns;
            let row = index / effective_columns;
            let position = egui::pos2(
                ER_CANVAS_MARGIN + column as f32 * (ER_NODE_WIDTH + ER_GAP_X),
                ER_CANVAS_MARGIN + row as f32 * (node_height + ER_GAP_Y),
            );
            let world_rect = egui::Rect::from_min_size(position, egui::vec2(ER_NODE_WIDTH, node_height));
            node_lookup.insert((table.schema.clone(), table.name.clone()), index);
            nodes.push(ErNode {
                id: index,
                table: table.clone(),
                world_rect,
            });
        }

        let mut edges = Vec::new();
        let mut edge_id = 0;

        for source_node in &nodes {
            for fk in &source_node.table.foreign_keys {
                if let Some(&target_index) = node_lookup.get(&(fk.to_schema.clone(), fk.to_table.clone())) {
                    adjacency.entry(source_node.id).or_default().push(target_index);
                    adjacency.entry(target_index).or_default().push(source_node.id);

                    let world_bbox = edge_world_bbox(&nodes, source_node.id, target_index, fk);

                    edges.push(ErEdge {
                        id: edge_id,
                        source: source_node.id,
                        target: target_index,
                        foreign_key: fk.clone(),
                        world_bbox,
                    });
                    edge_id += 1;
                }
            }
        }

        for neighbors in adjacency.values_mut() {
            neighbors.sort_unstable();
            neighbors.dedup();
        }

        let world_bounds = nodes_world_bounds(&nodes);

        Self {
            nodes,
            edges,
            node_lookup,
            adjacency,
            world_bounds,
            schema_version,
        }
    }

    /// Moves a node to a new top-left position, keeping its size. Incident edge
    /// bboxes are recomputed so viewport culling stays correct while dragging.
    /// `world_bounds` is only ever expanded here — call
    /// [`Self::recompute_world_bounds`] when the drag settles to shrink it back.
    pub fn move_node(&mut self, node_id: usize, new_min: egui::Pos2) {
        let Some(node) = self.nodes.get_mut(node_id) else {
            return;
        };
        node.world_rect = egui::Rect::from_min_size(new_min, node.world_rect.size());
        for edge in &mut self.edges {
            if edge.source == node_id || edge.target == node_id {
                edge.world_bbox = edge_world_bbox(&self.nodes, edge.source, edge.target, &edge.foreign_key);
            }
        }
        self.world_bounds = self.world_bounds.union(self.nodes[node_id].world_rect);
    }

    /// Recomputes the union of all node rects plus the canvas margin.
    pub fn recompute_world_bounds(&mut self) {
        self.world_bounds = nodes_world_bounds(&self.nodes);
    }

    /// Applies persisted manual positions (keyed by `schema.name`) onto a freshly
    /// built graph, then refreshes every edge bbox and the world bounds.
    /// Returns `true` when at least one node moved.
    pub fn apply_position_overrides(&mut self, positions: &HashMap<String, [f32; 2]>) -> bool {
        if positions.is_empty() {
            return false;
        }
        let mut moved = false;
        for node in &mut self.nodes {
            let key = er_table_key(&node.table.schema, &node.table.name);
            if let Some(&[x, y]) = positions.get(&key) {
                node.world_rect = egui::Rect::from_min_size(egui::pos2(x, y), node.world_rect.size());
                moved = true;
            }
        }
        if moved {
            for edge in &mut self.edges {
                edge.world_bbox = edge_world_bbox(&self.nodes, edge.source, edge.target, &edge.foreign_key);
            }
            self.recompute_world_bounds();
        }
        moved
    }

    pub fn active_subset_bounds(&self, subset: &[usize]) -> egui::Rect {
        let mut bounds: Option<egui::Rect> = None;
        for &id in subset {
            if let Some(node) = self.nodes.get(id) {
                bounds = match bounds {
                    Some(b) => Some(b.union(node.world_rect)),
                    None => Some(node.world_rect),
                };
            }
        }
        bounds.map_or(self.world_bounds, |b| b.expand(ER_CANVAS_MARGIN))
    }

    pub fn bfs_neighborhood(&self, seed_indices: &[usize], max_depth: usize, max_nodes: usize) -> Vec<usize> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        for &seed in seed_indices {
            if seed < self.nodes.len() && visited.insert(seed) {
                queue.push_back((seed, 0));
                result.push(seed);
                if result.len() >= max_nodes {
                    return result;
                }
            }
        }

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth || result.len() >= max_nodes {
                continue;
            }
            if let Some(neighbors) = self.adjacency.get(&current) {
                for &neighbor in neighbors {
                    if visited.insert(neighbor) {
                        result.push(neighbor);
                        queue.push_back((neighbor, depth + 1));
                        if result.len() >= max_nodes {
                            break;
                        }
                    }
                }
            }
        }

        result
    }
}

fn nodes_world_bounds(nodes: &[ErNode]) -> egui::Rect {
    if nodes.is_empty() {
        return egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(640.0, 360.0));
    }
    let mut bounds = nodes[0].world_rect;
    for node in &nodes[1..] {
        bounds = bounds.union(node.world_rect);
    }
    bounds.expand(ER_CANVAS_MARGIN)
}

/// Bounding box covering an edge's anchors and both endpoint nodes, used for
/// viewport culling. Shared by `ErGraph::build` and post-drag refresh so the
/// margin math never diverges.
fn edge_world_bbox(nodes: &[ErNode], source: usize, target: usize, fk: &UiSchemaForeignKey) -> egui::Rect {
    let (Some(source_node), Some(target_node)) = (nodes.get(source), nodes.get(target)) else {
        return egui::Rect::ZERO;
    };
    let source_anchor = source_node.column_anchor(fk.from_columns.first().map(String::as_str), true, ER_MAX_COLUMNS);
    let target_anchor = target_node.column_anchor(fk.to_columns.first().map(String::as_str), false, ER_MAX_COLUMNS);

    let min_x = source_anchor
        .x
        .min(target_anchor.x)
        .min(source_node.world_rect.left())
        .min(target_node.world_rect.left())
        - 40.0;
    let max_x = source_anchor
        .x
        .max(target_anchor.x)
        .max(source_node.world_rect.right())
        .max(target_node.world_rect.right())
        + 40.0;
    let min_y = source_anchor
        .y
        .min(target_anchor.y)
        .min(source_node.world_rect.top())
        .min(target_node.world_rect.top())
        - 20.0;
    let max_y = source_anchor
        .y
        .max(target_anchor.y)
        .max(source_node.world_rect.bottom())
        .max(target_node.world_rect.bottom())
        + 20.0;

    egui::Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y))
}
