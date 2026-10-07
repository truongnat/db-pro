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

        for (index, table) in tables.iter().enumerate() {
            node_lookup.insert((table.schema.clone(), table.name.clone()), index);
            nodes.push(ErNode {
                id: index,
                table: table.clone(),
                world_rect: egui::Rect::ZERO,
            });
        }
        assign_positions(&mut nodes, &node_lookup, grid_columns, node_height);

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

/// Connected components sit in their own block. A component with foreign keys
/// is layered parent-above-child (one barycenter pass). Tables with no
/// relationship stay in the original grid, to the right of those blocks.
fn assign_positions(
    nodes: &mut [ErNode],
    lookup: &HashMap<(String, String), usize>,
    grid_columns: usize,
    node_height: f32,
) {
    let count = nodes.len();
    if count == 0 {
        return;
    }
    // Resolve names once. Later passes only see node ids.
    let mut links: Vec<(usize, usize)> = Vec::new();
    for node in nodes.iter() {
        for foreign_key in &node.table.foreign_keys {
            let Some(&target) = lookup.get(&(foreign_key.to_schema.clone(), foreign_key.to_table.clone())) else {
                continue;
            };
            if target != node.id {
                links.push((node.id, target));
            }
        }
    }
    links.sort_unstable();
    links.dedup();
    let mut parent: Vec<usize> = (0..count).collect();
    let mut rank = vec![0u8; count];
    let mut linked = vec![false; count];
    for &(source, target) in &links {
        unite(&mut parent, &mut rank, source, target);
        linked[source] = true;
        linked[target] = true;
    }

    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for index in 0..count {
        groups.entry(find(&mut parent, index)).or_default().push(index);
    }
    let mut components: Vec<Vec<usize>> = groups.into_values().collect();
    components.sort_by_key(|group| group.iter().copied().min().unwrap_or(0));

    let step_x = ER_NODE_WIDTH + ER_GAP_X;
    let step_y = node_height + ER_GAP_Y;
    let mut cursor_x = ER_CANVAS_MARGIN;
    let mut isolates = Vec::new();
    for group in components {
        let connected = group.len() >= 2 && group.iter().any(|id| linked[*id]);
        if !connected {
            isolates.extend(group);
            continue;
        }
        let width = layout_component(
            nodes,
            &links,
            &group,
            cursor_x,
            ER_CANVAS_MARGIN,
            step_x,
            step_y,
            node_height,
        );
        cursor_x += width + ER_GAP_X;
    }

    let columns = grid_columns.max(1);
    for (index, id) in isolates.into_iter().enumerate() {
        let column = index % columns;
        let row = index / columns;
        let position = egui::pos2(
            cursor_x + column as f32 * step_x,
            ER_CANVAS_MARGIN + row as f32 * step_y,
        );
        nodes[id].world_rect = egui::Rect::from_min_size(position, egui::vec2(ER_NODE_WIDTH, node_height));
    }
}

fn find(parent: &mut [usize], mut index: usize) -> usize {
    while parent[index] != index {
        parent[index] = parent[parent[index]];
        index = parent[index];
    }
    index
}

fn unite(parent: &mut [usize], rank: &mut [u8], mut left: usize, mut right: usize) {
    left = find(parent, left);
    right = find(parent, right);
    if left == right {
        return;
    }
    if rank[left] < rank[right] {
        std::mem::swap(&mut left, &mut right);
    }
    parent[right] = left;
    if rank[left] == rank[right] {
        rank[left] = rank[left].saturating_add(1);
    }
}

/// Returns the width occupied by the component.
fn layout_component(
    nodes: &mut [ErNode],
    links: &[(usize, usize)],
    group: &[usize],
    origin_x: f32,
    origin_y: f32,
    step_x: f32,
    step_y: f32,
    node_height: f32,
) -> f32 {
    let members: HashSet<usize> = group.iter().copied().collect();
    let mut indegree: HashMap<usize, usize> = group.iter().copied().map(|id| (id, 0)).collect();
    let mut children: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(source, target) in links {
        if members.contains(&source) && members.contains(&target) {
            *indegree.entry(source).or_default() += 1;
            children.entry(target).or_default().push(source);
        }
    }
    let mut queue: Vec<usize> = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(id, _)| *id)
        .collect();
    queue.sort_unstable();
    let mut queue = VecDeque::from(queue);
    let mut layer: HashMap<usize, usize> = HashMap::new();
    while let Some(id) = queue.pop_front() {
        let my_layer = layer.get(&id).copied().unwrap_or(0);
        layer.entry(id).or_insert(my_layer);
        if let Some(kids) = children.get(&id) {
            for &kid in kids {
                let next = my_layer + 1;
                let kid_layer = layer.entry(kid).or_insert(0);
                if *kid_layer < next {
                    *kid_layer = next;
                }
                let degree = indegree.get_mut(&kid).expect("component member");
                *degree = degree.saturating_sub(1);
                if *degree == 0 {
                    queue.push_back(kid);
                }
            }
        }
    }
    for &id in group {
        layer.entry(id).or_insert(0);
    }
    let max_layer = layer.values().copied().max().unwrap_or(0);
    let mut rows: Vec<Vec<usize>> = vec![Vec::new(); max_layer + 1];
    for &id in group {
        rows[layer[&id]].push(id);
    }
    for row in &mut rows {
        row.sort_unstable();
    }
    // Parents of each child, built once for the barycenter pass.
    let mut parents: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(source, target) in links {
        if members.contains(&source) && members.contains(&target) {
            parents.entry(source).or_default().push(target);
        }
    }
    for row_index in 1..rows.len() {
        let previous: HashMap<usize, usize> = rows[row_index - 1]
            .iter()
            .enumerate()
            .map(|(position, id)| (*id, position))
            .collect();
        let mut ranked: Vec<(usize, usize)> = rows[row_index]
            .iter()
            .copied()
            .map(|id| {
                let mut sum = 0usize;
                let mut count = 0usize;
                if let Some(targets) = parents.get(&id) {
                    for target in targets {
                        if let Some(position) = previous.get(target) {
                            sum += *position;
                            count += 1;
                        }
                    }
                }
                let barycenter = if count == 0 {
                    id.saturating_mul(1024)
                } else {
                    sum.saturating_mul(1024) / count
                };
                (barycenter, id)
            })
            .collect();
        ranked.sort_unstable();
        rows[row_index] = ranked.into_iter().map(|(_, id)| id).collect();
    }

    let mut max_columns = 1usize;
    for (row_index, row) in rows.iter().enumerate() {
        max_columns = max_columns.max(row.len());
        for (column, &id) in row.iter().enumerate() {
            let position = egui::pos2(origin_x + column as f32 * step_x, origin_y + row_index as f32 * step_y);
            nodes[id].world_rect = egui::Rect::from_min_size(position, egui::vec2(ER_NODE_WIDTH, node_height));
        }
    }
    max_columns as f32 * step_x
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
    let anchor_columns = super::lod::ErLod::Detailed.max_columns();
    let source_anchor = source_node.column_anchor(fk.from_columns.first().map(String::as_str), true, anchor_columns);
    let target_anchor = target_node.column_anchor(fk.to_columns.first().map(String::as_str), false, anchor_columns);

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
