use super::*;
pub use crate::diagram::*;

pub(crate) struct DiagramViewContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) diagram: &'a mut DiagramState,
    pub(super) explorer: &'a SchemaExplorerState,
    pub(super) active_driver: &'a str,
    pub(super) connected: bool,
    /// Active connection id — scopes persisted layout snapshots (`None` for the
    /// disconnected/capture fixture paths, which simply never persist).
    pub(super) connection_id: Option<String>,
}

pub(crate) enum DiagramAction {
    OpenTable(String),
    ExecuteQuery(String),
}

pub(super) fn draw_diagram(ctx: &mut DiagramViewContext<'_>, ui: &mut egui::Ui) -> Option<DiagramAction> {
    let all_table_count = ctx.explorer.schema.table_details.len();
    let large_schema = all_table_count > ER_LARGE_SCHEMA_THRESHOLD;
    let search_query = ctx.diagram.search.trim().to_ascii_lowercase();
    let search_mode = diagram_search_mode(large_schema, ctx.diagram.show_all);

    // Poll background layout worker:
    if let Some(res) = ctx.diagram.layout_worker.poll_result() {
        if res.graph_version == ctx.diagram.schema_version && res.request_id == ctx.diagram.latest_layout_request {
            let mut graph = res.graph;
            let overrides_moved = ctx
                .connection_id
                .as_deref()
                .and_then(|id| ctx.diagram.saved_layouts.get(id))
                .is_some_and(|snapshot| graph.apply_position_overrides(&snapshot.positions));
            ctx.diagram.spatial_index = if overrides_moved {
                ErSpatialIndex::build(&graph.nodes, &graph.edges, DEFAULT_SPATIAL_CELL_SIZE)
            } else {
                res.spatial_index
            };
            ctx.diagram.graph = graph;
            ctx.diagram.layout_state = ErLayoutState::Ready;
            ctx.diagram.auto_fit_done = false;
            ctx.diagram.drag_node = None;
            ctx.diagram.selected_node = None;
        }
    }

    if ctx.explorer.schema.table_details.is_empty() {
        super::diagram_canvas_view::draw_diagram_empty_state(ctx, ui, 0, false, false);
        return None;
    }

    let render_limit = if !large_schema || ctx.diagram.show_all {
        all_table_count
    } else {
        ER_MAX_TABLES
    };

    let (_candidate_count, tables) = diagram_candidates(
        &ctx.explorer.schema.table_details,
        &search_query,
        search_mode,
        render_limit,
    );

    let grid_columns = if tables.len() <= 3 {
        tables.len().max(1)
    } else {
        ((all_table_count as f32).sqrt().ceil() as usize).clamp(3, 10)
    };

    let max_visible_columns = ctx
        .explorer
        .schema
        .table_details
        .iter()
        .take(50)
        .map(|table| table.columns.len().clamp(1, ER_MAX_COLUMNS))
        .max()
        .unwrap_or(1);
    let node_height = ER_HEADER_HEIGHT + ER_ROW_HEIGHT * max_visible_columns as f32;

    ensure_diagram_graph(ctx, grid_columns, node_height);

    // Determine active table subset:
    let active_node_indices: Option<Vec<usize>> = if search_mode && !search_query.is_empty() {
        let seed_indices: Vec<usize> = ctx
            .diagram
            .graph
            .nodes
            .iter()
            .filter(|node| matches_diagram_search(&node.table, &search_query))
            .map(|node| node.id)
            .collect();
        if seed_indices.is_empty() {
            draw_diagram_toolbar(ctx, ui, large_schema, all_table_count, &tables, render_limit);
            ui.add_space(10.0);
            super::diagram_canvas_view::draw_diagram_empty_state(ctx, ui, all_table_count, true, true);
            return None;
        }
        Some(
            ctx.diagram
                .graph
                .bfs_neighborhood(&seed_indices, ctx.diagram.neighborhood_depth, 100),
        )
    } else if search_mode && search_query.is_empty() {
        draw_diagram_toolbar(ctx, ui, large_schema, all_table_count, &tables, render_limit);
        ui.add_space(10.0);
        super::diagram_canvas_view::draw_diagram_empty_state(ctx, ui, all_table_count, true, false);
        return None;
    } else {
        None
    };

    draw_diagram_toolbar(ctx, ui, large_schema, all_table_count, &tables, render_limit);
    let design_action = super::diagram_design_panel_view::draw_er_design_panel(ctx, ui);
    ui.add_space(10.0);

    let canvas_action = super::diagram_canvas_view::draw_diagram_canvas(ctx, ui, active_node_indices.as_deref());
    design_action.or(canvas_action)
}

fn ensure_diagram_graph(ctx: &mut DiagramViewContext<'_>, grid_columns: usize, node_height: f32) {
    let current_count = ctx.explorer.schema.table_details.len();
    let graph_dirty = ctx.diagram.graph.nodes.len() != current_count
        || ctx.diagram.graph.schema_version != ctx.diagram.schema_version;

    if graph_dirty && current_count > 0 {
        if ctx.diagram.graph.nodes.is_empty() {
            // First load: build immediately so canvas starts populated without blank frame
            let mut graph = ErGraph::build(
                &ctx.explorer.schema.table_details,
                ctx.diagram.schema_version,
                grid_columns,
                node_height,
            );
            if let Some(snapshot) = ctx
                .connection_id
                .as_deref()
                .and_then(|id| ctx.diagram.saved_layouts.get(id))
            {
                graph.apply_position_overrides(&snapshot.positions);
            }
            ctx.diagram.spatial_index = ErSpatialIndex::build(
                &graph.nodes,
                &graph.edges,
                DEFAULT_SPATIAL_CELL_SIZE,
            );
            ctx.diagram.graph = graph;
            ctx.diagram.layout_state = ErLayoutState::Ready;
            ctx.diagram.auto_fit_done = false;
            ctx.diagram.drag_node = None;
            ctx.diagram.selected_node = None;
        } else {
            // Background worker update: keep old graph renderable and dispatch async request.
            // saturating_add: at u64::MAX the version stays MAX; subsequent invalidations
            // all produce the same version but the graph node count check (graph_dirty)
            // prevents re-dispatch unless the actual table list changes.
            ctx.diagram.schema_version = ctx.diagram.schema_version.saturating_add(1);
            let request_id = ctx.diagram.layout_worker.request_layout(
                ctx.diagram.schema_version,
                ctx.explorer.schema.table_details.clone(),
                grid_columns,
                node_height,
            );
            ctx.diagram.latest_layout_request = request_id;

            // If the worker is in degraded mode (spawn failed), the request was
            // silently dropped. Transition to Failed state so the UI shows a
            // concise error while retaining the last valid graph.
            if ctx.diagram.layout_worker.dispatch_succeeded() {
                ctx.diagram.layout_state = ErLayoutState::Computing {
                    request_id,
                    graph_version: ctx.diagram.schema_version,
                };
            } else {
                ctx.diagram.layout_state = ErLayoutState::Failed("ER layout worker unavailable".to_owned());
            }
        }
    }
}

pub(super) fn diagram_candidates(
    all_tables: &[UiTableSummary],
    search_query: &str,
    search_mode: bool,
    render_limit: usize,
) -> (usize, Vec<UiTableSummary>) {
    let candidate_count = if search_mode && !search_query.is_empty() {
        all_tables
            .iter()
            .filter(|table| matches_diagram_search(table, search_query))
            .count()
    } else {
        all_tables.len()
    };
    let tables = if search_mode {
        all_tables
            .iter()
            .filter(|table| !search_query.is_empty() && matches_diagram_search(table, search_query))
            .take(render_limit + 1)
            .cloned()
            .collect()
    } else {
        all_tables
            .iter()
            .take(render_limit.saturating_add(1))
            .cloned()
            .collect()
    };
    (candidate_count, tables)
}

pub(super) fn diagram_search_mode(large_schema: bool, show_all: bool) -> bool {
    large_schema && !show_all
}

pub(super) fn diagram_show_all_after_search_edit(show_all: bool, search_query: &str, changed: bool) -> bool {
    if changed && !search_query.trim().is_empty() {
        false
    } else {
        show_all
    }
}

fn draw_diagram_toolbar(
    ctx: &mut DiagramViewContext<'_>,
    ui: &mut egui::Ui,
    large_schema: bool,
    all_table_count: usize,
    tables: &[UiTableSummary],
    render_limit: usize,
) {
    let visible_tables = if ctx.diagram.show_all || !large_schema {
        all_table_count
    } else {
        tables.len().min(render_limit)
    };
    let relationship_count: usize = ctx.diagram.graph.edges.len();

    ui.horizontal(|ui| {
        badge(
            ui,
            &plural_count(visible_tables, "table", "tables"),
            ctx.theme.accent_soft,
            ctx.theme.accent,
        );
        badge(
            ui,
            &plural_count(relationship_count, "relationship", "relationships"),
            ctx.theme.surface_hover,
            ctx.theme.text_secondary,
        );
        if matches!(ctx.diagram.layout_state, ErLayoutState::Computing { .. }) {
            badge(ui, "Arranging…", ctx.theme.surface_hover, ctx.theme.text_secondary);
        }

        if large_schema {
            ui.add_space(8.0);
            let search_changed =
                input(ui, &mut ctx.diagram.search, "Find table or column…", 200.0, ctx.theme).changed();
            if search_changed {
                ctx.diagram.search_changed_at = Some(std::time::Instant::now());
                ctx.diagram.fitted_search.clear();
            }
            ctx.diagram.show_all =
                diagram_show_all_after_search_edit(ctx.diagram.show_all, &ctx.diagram.search, search_changed);
            let search_mode = diagram_search_mode(large_schema, ctx.diagram.show_all);
            if search_mode {
                if ui
                    .selectable_label(ctx.diagram.neighborhood_depth == 1, "1 hop")
                    .on_hover_text("Show matching tables and their direct relationships")
                    .clicked()
                {
                    ctx.diagram.neighborhood_depth = 1;
                }
                if ui
                    .selectable_label(ctx.diagram.neighborhood_depth == 2, "2 hops")
                    .on_hover_text("Also include neighbors of neighbors")
                    .clicked()
                {
                    ctx.diagram.neighborhood_depth = 2;
                }
                if compact_button(ui, format!("Show all {all_table_count}"), ctx.theme)
                    .on_hover_text("Render every table — can be slow on very large schemas")
                    .clicked()
                {
                    ctx.diagram.show_all = true;
                }
            } else if compact_button(ui, "Focus search", ctx.theme)
                .on_hover_text("Back to the focused neighborhood map")
                .clicked()
            {
                ctx.diagram.show_all = false;
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if compact_icon_button_active(ui, Icon::PencilRuler, ctx.diagram.design.enabled, ctx.theme)
                .on_hover_text("Design Mode — draft schema edits, never mutates the database until Apply")
                .clicked()
            {
                ctx.diagram.design.enabled = !ctx.diagram.design.enabled;
                if ctx.diagram.design.enabled {
                    let names: Vec<String> = ctx
                        .explorer
                        .schema
                        .table_details
                        .iter()
                        .map(|t| format!("{}.{}", t.schema, t.name))
                        .collect();
                    ctx.diagram
                        .design
                        .set_schema_fingerprint(crate::diagram::design_mode::schema_fingerprint_from_names(&names));
                }
            }
            let has_manual_layout = ctx
                .connection_id
                .as_deref()
                .and_then(|id| ctx.diagram.saved_layouts.get(id))
                .is_some_and(|snapshot| !snapshot.positions.is_empty());
            if compact_icon_button_enabled(ui, Icon::RotateCcw, has_manual_layout, ctx.theme)
                .on_hover_text("Reset layout — clear dragged positions and refit the map")
                .clicked()
            {
                reset_diagram_layout(ctx);
            }
        });
    });
}

/// 24px icon button with an `active` fill — the compact-row toggle variant of
/// [`compact_icon_button`].
fn compact_icon_button_active(ui: &mut egui::Ui, icon: Icon, active: bool, theme: DbProTheme) -> egui::Response {
    let button = egui::Button::new(
        RichText::new(char::from(icon).to_string())
            .font(egui::FontId::new(14.0, egui::FontFamily::Name("lucide".into())))
            .color(if active { theme.accent } else { theme.text_muted }),
    )
    .min_size(egui::vec2(24.0, 24.0))
    .rounding(egui::Rounding::same(7.0))
    .stroke(egui::Stroke::NONE);
    if active {
        ui.add(button.fill(theme.surface_active))
    } else {
        ui.add(button)
    }
}

/// Drops every manual position for this connection and rebuilds the grid, then
/// lets the canvas refit on the next frame.
fn reset_diagram_layout(ctx: &mut DiagramViewContext<'_>) {
    if let Some(conn) = ctx.connection_id.as_deref() {
        if let Some(snapshot) = ctx.diagram.saved_layouts.get_mut(conn) {
            snapshot.positions.clear();
        }
    }
    ctx.diagram.graph = ErGraph::default();
    ctx.diagram.spatial_index = ErSpatialIndex::default();
    ctx.diagram.drag_node = None;
    ctx.diagram.selected_node = None;
    ctx.diagram.auto_fit_done = false;
}

/// World-space grid: lines live in world coordinates so they pan and zoom with
/// the diagram. The world step doubles while the projected spacing drops below
/// a readable minimum instead of collapsing into a solid fill at low zoom.
pub(super) fn paint_diagram_grid(
    painter: &egui::Painter,
    viewport: &ErViewport,
    screen_rect: egui::Rect,
    theme: DbProTheme,
) {
    let visible = painter.clip_rect().intersect(screen_rect);
    if visible.is_negative() {
        return;
    }
    let mut step_world = 48.0_f32;
    while step_world * viewport.zoom < 14.0 {
        step_world *= 4.0;
    }
    let world_view = viewport.visible_world_rect(visible, 0.0);
    let grid_color = theme.border_subtle.linear_multiply(0.55);
    let start_x = (world_view.left() / step_world).floor() * step_world;
    let mut grid_x = start_x;
    while grid_x <= world_view.right() {
        let sx = viewport.world_to_screen_pos(egui::pos2(grid_x, 0.0)).x;
        painter.line_segment(
            [egui::pos2(sx, visible.top()), egui::pos2(sx, visible.bottom())],
            egui::Stroke::new(1.0, grid_color),
        );
        grid_x += step_world;
    }
    let start_y = (world_view.top() / step_world).floor() * step_world;
    let mut grid_y = start_y;
    while grid_y <= world_view.bottom() {
        let sy = viewport.world_to_screen_pos(egui::pos2(0.0, grid_y)).y;
        painter.line_segment(
            [egui::pos2(visible.left(), sy), egui::pos2(visible.right(), sy)],
            egui::Stroke::new(1.0, grid_color),
        );
        grid_y += step_world;
    }
}

pub(super) fn diagram_foreign_key_label(foreign_key: &UiSchemaForeignKey) -> String {
    if foreign_key.from_columns.len() > 1 || foreign_key.to_columns.len() > 1 {
        format!(
            "[{}] → [{}]",
            foreign_key.from_columns.join(", "),
            foreign_key.to_columns.join(", ")
        )
    } else {
        format!(
            "{} → {}",
            foreign_key.from_columns.first().map(String::as_str).unwrap_or("key"),
            foreign_key.to_columns.first().map(String::as_str).unwrap_or("key"),
        )
    }
}

/// Orthogonal three-segment edges. Parallel edges between the same column pair
/// used to share one `bend_x` and stack their labels — each edge now gets a
/// small deterministic lane offset so they fan out. When `highlight` names a
/// node (drag/hover/selection), only its incident edges keep full opacity.
pub(super) fn paint_scene_edges(
    painter: &egui::Painter,
    graph: &ErGraph,
    scene: &ErRenderScene,
    viewport: &ErViewport,
    theme: DbProTheme,
    highlight: Option<usize>,
) {
    let zoom = viewport.zoom;
    for &edge_id in &scene.visible_edges {
        let Some(edge) = graph.edges.get(edge_id) else {
            continue;
        };
        let Some(source_node) = graph.nodes.get(edge.source) else {
            continue;
        };
        let Some(target_node) = graph.nodes.get(edge.target) else {
            continue;
        };

        let source_on_right = source_node.world_rect.center().x < target_node.world_rect.center().x;
        let from_world = source_node.column_anchor(
            edge.foreign_key.from_columns.first().map(String::as_str),
            source_on_right,
            scene.lod.max_columns(),
        );
        let to_world = target_node.column_anchor(
            edge.foreign_key.to_columns.first().map(String::as_str),
            !source_on_right,
            scene.lod.max_columns(),
        );

        let from = viewport.world_to_screen_pos(from_world);
        let to = viewport.world_to_screen_pos(to_world);

        // Per-edge lane keeps parallel edges from drawing on top of each other.
        let lane = ((edge.id % 7) as f32 - 3.0) * 9.0 * zoom;
        let bend_x = (from.x + to.x) / 2.0 + lane;
        let bend_a = egui::pos2(bend_x, from.y);
        let bend_b = egui::pos2(bend_x, to.y);
        let emphasized = highlight.is_none_or(|id| edge.source == id || edge.target == id);
        // Edges stay readable at rest; dimming kicks in only while a node is
        // actively highlighted so unrelated routes fall back.
        let (edge_color, width) = if highlight.is_some() && !emphasized {
            (theme.accent.linear_multiply(0.18), 1.0)
        } else if emphasized {
            (theme.accent, (1.6 * zoom).clamp(1.2, 2.4))
        } else {
            (theme.accent.linear_multiply(0.7), (1.1 * zoom).clamp(0.8, 1.5))
        };
        let stroke = egui::Stroke::new(width, edge_color);
        painter.line_segment([from, bend_a], stroke);
        painter.line_segment([bend_a, bend_b], stroke);
        painter.line_segment([bend_b, to], stroke);

        // Crow's-foot notation: the FK (source, "many") side fans out into
        // three prongs; the referenced PK (target, "one") side gets a bar.
        // Both endpoints leave their nodes horizontally so `dir` is ±x.
        let dir = if source_on_right { 1.0 } else { -1.0 };
        let foot_len = (9.0 * zoom).clamp(4.5, 11.0);
        let foot_spread = foot_len * 0.8;
        let apex = egui::pos2(from.x + dir * foot_len, from.y);
        for dy in [-foot_spread, 0.0, foot_spread] {
            painter.line_segment([apex, egui::pos2(from.x, from.y + dy)], stroke);
        }
        let bar_half = foot_len * 0.65;
        painter.line_segment(
            [egui::pos2(to.x, to.y - bar_half), egui::pos2(to.x, to.y + bar_half)],
            stroke,
        );

        // `fk_col → pk_col` is the clearest statement of what a line means.
        // Detailed labels every edge; Standard labels an edge only while its
        // node is highlighted (hover/drag/click-pin) so dense maps stay clean.
        if scene.lod.shows_edge_labels() || (emphasized && highlight.is_some()) {
            let label = diagram_foreign_key_label(&edge.foreign_key);
            let display_label = if label.len() > 36 {
                format!("{}…", &label[..34])
            } else {
                label
            };
            let label_color = if emphasized {
                theme.text_secondary
            } else {
                theme.text_muted.linear_multiply(0.7)
            };
            let label_galley =
                painter.layout_no_wrap(display_label.clone(), FontId::proportional(10.0), label_color);
            // Right of the vertical segment so the text never sits on the line.
            let label_position = egui::pos2(
                bend_x + label_galley.size().x / 2.0 + 7.0,
                (from.y + to.y) / 2.0,
            );
            let label_rect = egui::Rect::from_center_size(label_position, label_galley.size() + egui::vec2(10.0, 6.0));
            painter.rect_filled(label_rect, egui::Rounding::same(4.0), theme.surface_panel);
            painter.rect_stroke(
                label_rect,
                egui::Rounding::same(4.0),
                egui::Stroke::new(1.0, theme.border_subtle),
            );
            painter.text(
                label_position,
                egui::Align2::CENTER_CENTER,
                display_label,
                FontId::proportional(10.0),
                label_color,
            );
        }
    }
}

/// Truncates `text` so its rendered width fits `max_width`, measuring with the
/// real font instead of a fixed char count — `schema.table` names otherwise
/// clipped at ~18 chars while the card had room for ~36.
fn truncate_to_width(painter: &egui::Painter, text: &str, font: FontId, max_width: f32) -> String {
    let galley = painter.layout_no_wrap(text.to_owned(), font.clone(), egui::Color32::WHITE);
    if galley.size().x <= max_width {
        return text.to_owned();
    }
    let char_count = text.chars().count().max(1) as f32;
    let budget = ((max_width / galley.size().x) * char_count).floor() as usize;
    crate::components::truncate_ellipsis(text, budget.max(4))
}

/// `to_table.to_column` (or just the table for composite keys) — the FK target
/// shown on column rows at Detailed LOD.
fn fk_target_label(foreign_key: &UiSchemaForeignKey) -> String {
    foreign_key
        .to_columns
        .first()
        .map(|col| format!("{}.{}", foreign_key.to_table, col))
        .unwrap_or_else(|| foreign_key.to_table.clone())
}

/// Compact row-count for header subtitles: `950`, `12.4K`, `1.2M`.
fn compact_count(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1e6)
    } else if count >= 10_000 {
        format!("{:.1}K", count as f64 / 1e3)
    } else {
        count.to_string()
    }
}

pub(super) fn paint_er_node_lod(
    painter: &egui::Painter,
    node: &ErNode,
    screen_rect: egui::Rect,
    selected: bool,
    hovered: bool,
    lod: ErLod,
    theme: DbProTheme,
) {
    // The node rect is already screen-scaled, so derive zoom back from its width.
    let zoom = screen_rect.width() / ER_NODE_WIDTH;
    match lod {
        ErLod::Compact if !selected => {
            // Compact pill card: only header with table name and PK count
            painter.rect_filled(screen_rect, egui::Rounding::same(6.0), theme.surface_panel);
            painter.rect_stroke(
                screen_rect,
                egui::Rounding::same(6.0),
                egui::Stroke::new(if hovered { 1.4 } else { 1.0 }, theme.border_default),
            );
            painter.rect_filled(screen_rect, egui::Rounding::same(6.0), theme.surface_hover);
            let title = truncate_to_width(
                painter,
                &node.table.name,
                FontId::proportional((12.0 * zoom).clamp(8.0, 14.0)),
                screen_rect.width() - 18.0 * zoom,
            );
            painter.text(
                screen_rect.center_top() + egui::vec2(0.0, 14.0 * zoom),
                egui::Align2::CENTER_CENTER,
                title,
                FontId::proportional((12.0 * zoom).clamp(8.0, 14.0)),
                theme.text_primary,
            );
            let pk_count = node.table.columns.iter().filter(|c| c.is_primary_key).count();
            if pk_count > 0 {
                painter.text(
                    egui::pos2(screen_rect.right() - 8.0 * zoom, screen_rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    format!("{pk_count} PK"),
                    FontId::proportional((9.0 * zoom).clamp(7.0, 11.0)),
                    theme.warning,
                );
            }
        }
        _ => {
            // Header:
            painter.rect_filled(screen_rect, egui::Rounding::same(8.0), theme.surface_panel);
            let border_stroke = if selected {
                egui::Stroke::new(1.5, theme.accent)
            } else if hovered {
                egui::Stroke::new(1.4, theme.accent.linear_multiply(0.8))
            } else {
                egui::Stroke::new(1.0, theme.border_default)
            };
            painter.rect_stroke(screen_rect, egui::Rounding::same(8.0), border_stroke);

            let header_height = ER_HEADER_HEIGHT * zoom;
            let header_rect = egui::Rect::from_min_max(
                screen_rect.min,
                egui::pos2(screen_rect.max.x, screen_rect.min.y + header_height),
            );
            let header_fill = if selected {
                theme.accent_soft
            } else {
                theme.surface_hover
            };
            painter.rect_filled(header_rect, egui::Rounding::same(8.0), header_fill);
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(header_rect.min.x, header_rect.max.y - 8.0 * zoom),
                    header_rect.max,
                ),
                egui::Rounding::ZERO,
                header_fill,
            );

            // Two-line header: table name gets the full width (measured, not a
            // fixed char cap); schema + column count sit muted underneath.
            let title_font = FontId::proportional(12.5 * zoom);
            let display_title = truncate_to_width(
                painter,
                &node.table.name,
                title_font.clone(),
                screen_rect.width() - 24.0 * zoom,
            );
            painter.text(
                screen_rect.min + egui::vec2(12.0 * zoom, 15.0 * zoom),
                egui::Align2::LEFT_CENTER,
                display_title,
                title_font,
                theme.text_primary,
            );
            let subtitle = {
                let cols = plural_count(node.table.columns.len(), "col", "cols");
                let mut parts = Vec::with_capacity(3);
                if !node.table.schema.is_empty() {
                    parts.push(node.table.schema.clone());
                }
                if let Some(rows) = node.table.row_count {
                    parts.push(format!("{} rows", compact_count(rows)));
                }
                parts.push(cols);
                parts.join(" · ")
            };
            let subtitle = truncate_to_width(
                painter,
                &subtitle,
                FontId::proportional(9.5 * zoom),
                screen_rect.width() - 24.0 * zoom,
            );
            painter.text(
                screen_rect.min + egui::vec2(12.0 * zoom, 30.0 * zoom),
                egui::Align2::LEFT_CENTER,
                subtitle,
                FontId::proportional(9.5 * zoom),
                if selected { theme.accent } else { theme.text_muted },
            );

            // Columns:
            let max_cols = if selected && lod == ErLod::Compact {
                3
            } else {
                lod.max_columns()
            };
            // The overflow hint takes the LAST row slot instead of painting on
            // top of a real column — `max_cols - 1` columns then "+ N more".
            let overflow = node.table.columns.len() > max_cols;
            let shown_cols = if overflow { max_cols.saturating_sub(1) } else { max_cols };
            for (index, column) in node.table.columns.iter().take(shown_cols).enumerate() {
                let row_top = screen_rect.min.y + (ER_HEADER_HEIGHT + index as f32 * ER_ROW_HEIGHT) * zoom;
                let row_rect = egui::Rect::from_min_max(
                    egui::pos2(screen_rect.min.x, row_top),
                    egui::pos2(screen_rect.max.x, row_top + ER_ROW_HEIGHT * zoom),
                );
                if index % 2 == 0 {
                    painter.rect_filled(row_rect, egui::Rounding::ZERO, theme.surface_app);
                }
                let foreign_key = node
                    .table
                    .foreign_keys
                    .iter()
                    .find(|foreign_key| foreign_key.from_columns.iter().any(|name| name == &column.name));
                let (marker_color, marker_icon) = if column.is_primary_key {
                    (theme.warning, Some(Icon::KeyRound))
                } else if foreign_key.is_some() {
                    (theme.accent, Some(Icon::Link))
                } else {
                    (theme.border_strong, None)
                };
                let marker_pos = egui::pos2(row_rect.min.x + 13.0 * zoom, row_rect.center().y);
                if let Some(icon) = marker_icon {
                    painter.text(
                        marker_pos,
                        egui::Align2::CENTER_CENTER,
                        char::from(icon),
                        FontId::new(10.0 * zoom, egui::FontFamily::Name("lucide".into())),
                        marker_color,
                    );
                } else {
                    painter.circle_filled(marker_pos, 2.0 * zoom, marker_color);
                }

                // FK columns carry the most useful fact in the diagram — where
                // they point — so at Detailed the right slot trades the type
                // for `FK → table.column`; a column that is both PK and FK
                // reads `PK·FK → target`.
                let type_display = if column.is_primary_key {
                    match (lod == ErLod::Detailed, foreign_key) {
                        (true, Some(foreign_key)) => format!("PK·FK → {}", fk_target_label(foreign_key)),
                        _ => format!("PK · {}", column.data_type),
                    }
                } else if let Some(foreign_key) = foreign_key {
                    if lod == ErLod::Detailed {
                        format!("FK → {}", fk_target_label(foreign_key))
                    } else {
                        format!("FK · {}", column.data_type)
                    }
                } else if !column.nullable {
                    format!("{} · NN", column.data_type)
                } else {
                    column.data_type.clone()
                };
                let type_display = truncate_to_width(
                    painter,
                    &type_display,
                    FontId::monospace(9.5 * zoom),
                    screen_rect.width() * 0.55,
                );
                let name_width = if lod.shows_data_types() {
                    // Leave room for the right-aligned type badge.
                    let type_galley = painter.layout_no_wrap(
                        type_display.clone(),
                        FontId::monospace(9.5 * zoom),
                        egui::Color32::WHITE,
                    );
                    screen_rect.width() - (23.0 + 12.0) * zoom - type_galley.size().x - 8.0 * zoom
                } else {
                    screen_rect.width() - (23.0 + 12.0) * zoom
                };
                let col_name = truncate_to_width(
                    painter,
                    &column.name,
                    FontId::proportional(11.0 * zoom),
                    name_width.max(24.0 * zoom),
                );
                painter.text(
                    egui::pos2(row_rect.min.x + 23.0 * zoom, row_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    col_name,
                    FontId::proportional(11.0 * zoom),
                    theme.text_primary,
                );
                if lod.shows_data_types() {
                    painter.text(
                        egui::pos2(row_rect.max.x - 12.0 * zoom, row_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        type_display,
                        FontId::monospace(9.5 * zoom),
                        if column.is_primary_key {
                            theme.warning
                        } else if foreign_key.is_some() {
                            theme.accent
                        } else {
                            theme.text_secondary
                        },
                    );
                }
            }
            if overflow {
                let row_top = screen_rect.min.y + (ER_HEADER_HEIGHT + shown_cols as f32 * ER_ROW_HEIGHT) * zoom;
                painter.text(
                    egui::pos2(screen_rect.min.x + 14.0 * zoom, row_top + ER_ROW_HEIGHT * zoom * 0.5),
                    egui::Align2::LEFT_CENTER,
                    format!("+ {} more columns", node.table.columns.len() - shown_cols),
                    FontId::proportional(10.0 * zoom),
                    theme.text_muted,
                );
            }
        }
    }
}

pub(super) fn draw_diagram_zoom_controls(
    ui: &mut egui::Ui,
    viewport_rect: egui::Rect,
    zoom: &mut f32,
    pan: &mut egui::Vec2,
    graph: &ErGraph,
    active_filter: Option<&[usize]>,
    theme: DbProTheme,
) {
    let controls_rect = zoom_controls_rect(viewport_rect);
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(controls_rect), |ui| {
        egui::Frame {
            fill: theme.surface_floating,
            inner_margin: egui::Margin::symmetric(5.0, 4.0),
            rounding: egui::Rounding::same(6.0),
            stroke: egui::Stroke::new(1.0, theme.border_subtle),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if compact_icon_button(ui, Icon::Minus, theme)
                    .on_hover_text("Zoom out")
                    .clicked()
                {
                    *zoom = (*zoom - 0.1).clamp(ER_MIN_ZOOM, ER_MAX_ZOOM);
                }
                if ui
                    .add(
                        egui::Label::new(
                            RichText::new(format!("{:.0}%", *zoom * 100.0))
                                .small()
                                .color(theme.text_secondary),
                        )
                        .sense(Sense::click()),
                    )
                    .on_hover_text("Click to reset zoom to 100%")
                    .clicked()
                {
                    *zoom = 1.0;
                }
                if compact_icon_button(ui, Icon::Plus, theme)
                    .on_hover_text("Zoom in")
                    .clicked()
                {
                    *zoom = (*zoom + 0.1).clamp(ER_MIN_ZOOM, ER_MAX_ZOOM);
                }
                if compact_icon_button(ui, Icon::RotateCcw, theme)
                    .on_hover_text("Reset zoom (100%)")
                    .clicked()
                {
                    *zoom = 1.0;
                    *pan = egui::Vec2::ZERO;
                }
                if compact_icon_button(ui, Icon::Maximize2, theme)
                    .on_hover_text("Fit diagram to viewport")
                    .clicked()
                {
                    let target_bounds = if let Some(filter) = active_filter {
                        graph.active_subset_bounds(filter)
                    } else {
                        graph.world_bounds
                    };
                    if let Some((fit_zoom, fit_pan)) = fit_diagram_viewport(target_bounds, viewport_rect) {
                        *zoom = fit_zoom;
                        *pan = fit_pan;
                    } else {
                        *zoom = 1.0;
                        *pan = egui::Vec2::ZERO;
                    }
                }
            });
        });
    });
}

/// Screen rect of the floating zoom strip — shared with the canvas so pointer
/// presses on the strip never fall through to pan/drag gestures.
pub(super) fn zoom_controls_rect(viewport_rect: egui::Rect) -> egui::Rect {
    let control_width = 164.0_f32.min((viewport_rect.width() - 16.0).max(0.0));
    egui::Rect::from_min_size(
        egui::pos2(viewport_rect.right() - control_width - 8.0, viewport_rect.top() + 8.0),
        egui::vec2(control_width, 32.0),
    )
}

pub(super) fn fit_diagram_viewport(bounds: egui::Rect, viewport_rect: egui::Rect) -> Option<(f32, egui::Vec2)> {
    let bounds_size = bounds.size();
    if bounds_size.x <= 10.0 || bounds_size.y <= 10.0 {
        return None;
    }

    let zoom_x = (viewport_rect.width() - 80.0) / bounds_size.x;
    let zoom_y = (viewport_rect.height() - 80.0) / bounds_size.y;
    let zoom = zoom_x.min(zoom_y).clamp(ER_MIN_ZOOM, 1.5);
    let pan = egui::vec2(
        viewport_rect.width() / 2.0 - bounds.center().x * zoom,
        viewport_rect.height() / 2.0 - bounds.center().y * zoom,
    );

    Some((zoom, pan))
}

#[cfg(test)]
mod viewport_tests {
    use super::*;

    #[test]
    fn fit_view_uses_visible_viewport_instead_of_scroll_content_size() {
        let viewport_rect = egui::Rect::from_min_size(egui::pos2(40.0, 70.0), egui::vec2(900.0, 600.0));
        let bounds = egui::Rect::from_min_size(egui::pos2(48.0, 48.0), egui::vec2(3200.0, 2400.0));

        let (zoom, pan) = fit_diagram_viewport(bounds, viewport_rect).expect("non-empty diagram");
        let fitted = ErViewport::new(pan, zoom, viewport_rect.min).world_to_screen_rect(bounds);

        assert!(fitted.left() >= viewport_rect.left() - 1.0);
        assert!(fitted.right() <= viewport_rect.right() + 1.0);
        assert!(fitted.top() >= viewport_rect.top() - 1.0);
        assert!(fitted.bottom() <= viewport_rect.bottom() + 1.0);
        assert!(zoom < 0.5, "large diagram should be allowed to zoom out to fit: {zoom}");
    }
}
