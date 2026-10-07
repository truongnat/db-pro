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

/// Which card edge an edge attaches to (`right_side`) and how many column rows
/// are rendered at the current zoom (`max_columns`) — anchors land on the
/// column row when it is visible, otherwise the card center.
#[derive(Debug, Clone, Copy)]
pub struct ErAnchorSpec {
    pub right_side: bool,
    pub max_columns: usize,
}

impl ErAnchorSpec {
    pub fn right(max_columns: usize) -> Self {
        Self {
            right_side: true,
            max_columns,
        }
    }

    pub fn left(max_columns: usize) -> Self {
        Self {
            right_side: false,
            max_columns,
        }
    }
}

impl ErNode {
    pub fn column_anchor(&self, column_name: Option<&str>, spec: ErAnchorSpec) -> egui::Pos2 {
        let visible_columns = self.table.columns.len().min(spec.max_columns);
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
            if spec.right_side {
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

/// Grid geometry shared by the layout passes: columns for the isolate grid and
/// the uniform card height the LOD picked for this build.
#[derive(Debug, Clone, Copy)]
pub struct ErGridSpec {
    pub grid_columns: usize,
    pub node_height: f32,
}

impl From<(usize, f32)> for ErGridSpec {
    fn from((grid_columns, node_height): (usize, f32)) -> Self {
        Self {
            grid_columns,
            node_height,
        }
    }
}

/// Bounds for [`ErGraph::bfs_neighborhood`]: how many hops out to walk and the
/// total node cap that keeps search subsets usable on large schemas.
#[derive(Debug, Clone, Copy)]
pub struct ErBfsOptions {
    pub max_depth: usize,
    pub max_nodes: usize,
}

impl From<(usize, usize)> for ErBfsOptions {
    fn from((max_depth, max_nodes): (usize, usize)) -> Self {
        Self { max_depth, max_nodes }
    }
}

impl ErGraph {
    pub fn build(tables: &[UiTableSummary], schema_version: u64, grid: impl Into<ErGridSpec>) -> Self {
        let grid = grid.into();
        let (mut nodes, node_lookup) = collect_nodes(tables);
        assign_positions(&mut nodes, &node_lookup, grid.grid_columns, grid.node_height);
        let (edges, adjacency) = collect_edges(&nodes, &node_lookup);
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
                edge.world_bbox = edge_world_bbox(&self.nodes, edge);
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
                edge.world_bbox = edge_world_bbox(&self.nodes, edge);
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

    pub fn bfs_neighborhood(&self, seed_indices: &[usize], options: impl Into<ErBfsOptions>) -> Vec<usize> {
        let mut bfs = ErBfsTraversal::new(self, options.into());
        for &seed in seed_indices {
            if bfs.full() {
                break;
            }
            if seed < self.nodes.len() {
                bfs.push(seed, 0);
            }
        }
        while let Some((current, depth)) = bfs.queue.pop_front() {
            if bfs.full() || depth >= bfs.options.max_depth {
                continue;
            }
            bfs.expand(current, depth);
        }
        bfs.result
    }
}

/// Bookkeeping for one `bfs_neighborhood` run: visited set, frontier queue, and
/// the result order the caller sees. Bundled so the walk itself stays a flat
/// seed-then-expand loop.
struct ErBfsTraversal<'a> {
    adjacency: &'a HashMap<usize, Vec<usize>>,
    options: ErBfsOptions,
    visited: HashSet<usize>,
    queue: VecDeque<(usize, usize)>,
    result: Vec<usize>,
}

impl ErBfsTraversal<'_> {
    fn new(graph: &ErGraph, options: ErBfsOptions) -> ErBfsTraversal<'_> {
        ErBfsTraversal {
            adjacency: &graph.adjacency,
            options,
            visited: HashSet::new(),
            queue: VecDeque::new(),
            result: Vec::new(),
        }
    }

    fn full(&self) -> bool {
        self.result.len() >= self.options.max_nodes
    }

    fn push(&mut self, node: usize, depth: usize) {
        if self.visited.insert(node) {
            self.result.push(node);
            self.queue.push_back((node, depth));
        }
    }

    fn expand(&mut self, current: usize, depth: usize) {
        let Some(neighbors) = self.adjacency.get(&current) else {
            return;
        };
        for &neighbor in neighbors {
            if self.full() {
                return;
            }
            self.push(neighbor, depth + 1);
        }
    }
}

fn collect_nodes(tables: &[UiTableSummary]) -> (Vec<ErNode>, HashMap<(String, String), usize>) {
    let mut nodes = Vec::with_capacity(tables.len());
    let mut node_lookup = HashMap::with_capacity(tables.len());
    for (index, table) in tables.iter().enumerate() {
        node_lookup.insert((table.schema.clone(), table.name.clone()), index);
        nodes.push(ErNode {
            id: index,
            table: table.clone(),
            world_rect: egui::Rect::ZERO,
        });
    }
    (nodes, node_lookup)
}

fn collect_edges(
    nodes: &[ErNode],
    node_lookup: &HashMap<(String, String), usize>,
) -> (Vec<ErEdge>, HashMap<usize, Vec<usize>>) {
    let mut edges = Vec::new();
    let mut adjacency: HashMap<usize, Vec<usize>> = HashMap::new();
    for source_node in nodes {
        for fk in &source_node.table.foreign_keys {
            let Some(&target_index) = node_lookup.get(&(fk.to_schema.clone(), fk.to_table.clone())) else {
                continue;
            };
            adjacency.entry(source_node.id).or_default().push(target_index);
            adjacency.entry(target_index).or_default().push(source_node.id);
            let mut edge = ErEdge {
                id: edges.len(),
                source: source_node.id,
                target: target_index,
                foreign_key: fk.clone(),
                world_bbox: egui::Rect::ZERO,
            };
            edge.world_bbox = edge_world_bbox(nodes, &edge);
            edges.push(edge);
        }
    }
    for neighbors in adjacency.values_mut() {
        neighbors.sort_unstable();
        neighbors.dedup();
    }
    (edges, adjacency)
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
    let mut union_find = ErUnionFind::new(count);
    let mut linked = vec![false; count];
    for &(source, target) in &links {
        union_find.unite(source, target);
        linked[source] = true;
        linked[target] = true;
    }

    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for index in 0..count {
        groups.entry(union_find.find(index)).or_default().push(index);
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
        let mut layout = ErLayoutContext {
            nodes,
            links: &links,
            origin: egui::pos2(cursor_x, ER_CANVAS_MARGIN),
            step: egui::vec2(step_x, step_y),
            node_height,
        };
        let width = layout_component(&mut layout, &group);
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

/// Union-find over node ids with path halving and union by rank — groups the
/// graph into connected components before each is laid out as its own block.
struct ErUnionFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl ErUnionFind {
    fn new(count: usize) -> Self {
        Self {
            parent: (0..count).collect(),
            rank: vec![0; count],
        }
    }

    fn find(&mut self, mut index: usize) -> usize {
        while self.parent[index] != index {
            self.parent[index] = self.parent[self.parent[index]];
            index = self.parent[index];
        }
        index
    }

    fn unite(&mut self, mut left: usize, mut right: usize) {
        left = self.find(left);
        right = self.find(right);
        if left == right {
            return;
        }
        if self.rank[left] < self.rank[right] {
            std::mem::swap(&mut left, &mut right);
        }
        self.parent[right] = left;
        if self.rank[left] == self.rank[right] {
            self.rank[left] = self.rank[left].saturating_add(1);
        }
    }
}

/// Shared geometry inputs for [`layout_component`] — keeps the layering and
/// barycenter passes free of positional bookkeeping arguments.
struct ErLayoutContext<'a> {
    nodes: &'a mut [ErNode],
    links: &'a [(usize, usize)],
    origin: egui::Pos2,
    step: egui::Vec2,
    node_height: f32,
}

/// Returns the width occupied by the component.
fn layout_component(ctx: &mut ErLayoutContext<'_>, group: &[usize]) -> f32 {
    let layer = layer_ranks(ctx.links, group);
    let max_layer = layer.values().copied().max().unwrap_or(0);
    let mut rows: Vec<Vec<usize>> = vec![Vec::new(); max_layer + 1];
    for &id in group {
        rows[layer[&id]].push(id);
    }
    for row in &mut rows {
        row.sort_unstable();
    }
    let members: HashSet<usize> = group.iter().copied().collect();
    barycenter_rows(&mut rows, ctx.links, &members);

    let mut max_columns = 1usize;
    for (row_index, row) in rows.iter().enumerate() {
        max_columns = max_columns.max(row.len());
        for (column, &id) in row.iter().enumerate() {
            let position = egui::pos2(
                ctx.origin.x + column as f32 * ctx.step.x,
                ctx.origin.y + row_index as f32 * ctx.step.y,
            );
            ctx.nodes[id].world_rect = egui::Rect::from_min_size(position, egui::vec2(ER_NODE_WIDTH, ctx.node_height));
        }
    }
    max_columns as f32 * ctx.step.x
}

/// Kahn layering over the component's links: each member gets the row (depth)
/// it sits on. Members unreachable from the topological walk fall back to 0.
/// Incoming-degree and child lists for the links that stay inside `members`.
fn component_link_maps(
    links: &[(usize, usize)],
    group: &[usize],
) -> (HashMap<usize, usize>, HashMap<usize, Vec<usize>>) {
    let members: HashSet<usize> = group.iter().copied().collect();
    let mut indegree: HashMap<usize, usize> = group.iter().copied().map(|id| (id, 0)).collect();
    let mut children: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(source, target) in links {
        if members.contains(&source) && members.contains(&target) {
            *indegree.entry(source).or_default() += 1;
            children.entry(target).or_default().push(source);
        }
    }
    (indegree, children)
}

fn layer_ranks(links: &[(usize, usize)], group: &[usize]) -> HashMap<usize, usize> {
    let (mut indegree, children) = component_link_maps(links, group);
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
        let Some(kids) = children.get(&id) else {
            continue;
        };
        for &kid in kids {
            let next = my_layer + 1;
            let kid_layer = layer.entry(kid).or_insert(0);
            if *kid_layer < next {
                *kid_layer = next;
            }
            // kid is always a component member: children only ever collects
            // targets inside `members`. let-else keeps that invariant
            // branch-free instead of panicking on it.
            let Some(degree) = indegree.get_mut(&kid) else {
                continue;
            };
            *degree = degree.saturating_sub(1);
            if *degree == 0 {
                queue.push_back(kid);
            }
        }
    }
    for &id in group {
        layer.entry(id).or_insert(0);
    }
    layer
}

/// Mean column position of `id`'s upstream parents in the previous row,
/// scaled by 1024 so the sort key stays integral. Falls back to `id` itself
/// when the member has no parent in that row.
fn barycenter_of(id: usize, parents: &HashMap<usize, Vec<usize>>, previous: &HashMap<usize, usize>) -> usize {
    let (sum, count) = parents
        .get(&id)
        .into_iter()
        .flatten()
        .filter_map(|target| previous.get(target))
        .fold((0usize, 0usize), |(sum, count), position| (sum + position, count + 1));
    sum.saturating_mul(1024)
        .checked_div(count)
        .unwrap_or_else(|| id.saturating_mul(1024))
}

/// Reorders each row by the mean column position of its upstream parents
/// (barycenter heuristic) so layered rows settle into readable columns.
fn barycenter_rows(rows: &mut [Vec<usize>], links: &[(usize, usize)], members: &HashSet<usize>) {
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
            .map(|id| (barycenter_of(id, &parents, &previous), id))
            .collect();
        ranked.sort_unstable();
        rows[row_index] = ranked.into_iter().map(|(_, id)| id).collect();
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
fn edge_world_bbox(nodes: &[ErNode], edge: &ErEdge) -> egui::Rect {
    let (Some(source_node), Some(target_node)) = (nodes.get(edge.source), nodes.get(edge.target)) else {
        return egui::Rect::ZERO;
    };
    let anchor_columns = super::lod::ErLod::Detailed.max_columns();
    let fk = &edge.foreign_key;
    let source_anchor = source_node.column_anchor(
        fk.from_columns.first().map(String::as_str),
        ErAnchorSpec::right(anchor_columns),
    );
    let target_anchor = target_node.column_anchor(
        fk.to_columns.first().map(String::as_str),
        ErAnchorSpec::left(anchor_columns),
    );

    let xs = [
        source_anchor.x,
        target_anchor.x,
        source_node.world_rect.left(),
        target_node.world_rect.left(),
        source_node.world_rect.right(),
        target_node.world_rect.right(),
    ];
    let ys = [
        source_anchor.y,
        target_anchor.y,
        source_node.world_rect.top(),
        target_node.world_rect.top(),
        source_node.world_rect.bottom(),
        target_node.world_rect.bottom(),
    ];
    let min_x = xs.iter().copied().fold(f32::INFINITY, f32::min) - 40.0;
    let max_x = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max) + 40.0;
    let min_y = ys.iter().copied().fold(f32::INFINITY, f32::min) - 20.0;
    let max_y = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max) + 20.0;

    egui::Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y))
}
