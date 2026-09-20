use super::*;
pub use crate::diagram::*;

impl DbProApp {
    pub(super) fn draw_diagram(&mut self, ui: &mut egui::Ui) {
        let all_table_count = self.schema_explorer.schema.table_details.len();
        let large_schema = all_table_count > ER_LARGE_SCHEMA_THRESHOLD;
        let search_query = self.diagram.search.trim().to_ascii_lowercase();
        let search_mode = diagram_search_mode(large_schema, self.diagram.show_all);

        // Poll background layout worker:
        if let Some(res) = self.diagram.layout_worker.poll_result() {
            if res.graph_version == self.diagram.schema_version && res.request_id == self.diagram.latest_layout_request
            {
                self.diagram.graph = res.graph;
                self.diagram.spatial_index = res.spatial_index;
                self.diagram.layout_state = ErLayoutState::Ready;
            }
        }

        if self.schema_explorer.schema.table_details.is_empty() {
            self.draw_diagram_empty_state(ui, 0, false, false);
            return;
        }

        let render_limit = if !large_schema || self.diagram.show_all {
            all_table_count
        } else {
            ER_MAX_TABLES
        };

        let (_candidate_count, tables) = diagram_candidates(
            &self.schema_explorer.schema.table_details,
            &search_query,
            search_mode,
            render_limit,
        );

        let grid_columns = if tables.len() <= 3 {
            tables.len().max(1)
        } else {
            ((all_table_count as f32).sqrt().ceil() as usize).clamp(3, 10)
        };

        let max_visible_columns = self
            .schema_explorer
            .schema
            .table_details
            .iter()
            .take(50)
            .map(|table| table.columns.len().clamp(1, ER_MAX_COLUMNS))
            .max()
            .unwrap_or(1);
        let node_height = ER_HEADER_HEIGHT + ER_ROW_HEIGHT * max_visible_columns as f32;

        self.ensure_diagram_graph(grid_columns, node_height);

        // Determine active table subset:
        let active_node_indices: Option<Vec<usize>> = if search_mode && !search_query.is_empty() {
            let seed_indices: Vec<usize> = self
                .diagram
                .graph
                .nodes
                .iter()
                .filter(|node| matches_diagram_search(&node.table, &search_query))
                .map(|node| node.id)
                .collect();
            if seed_indices.is_empty() {
                self.draw_diagram_toolbar(ui, large_schema, all_table_count, &tables, render_limit);
                ui.add_space(10.0);
                self.draw_diagram_empty_state(ui, all_table_count, true, true);
                return;
            }
            Some(
                self.diagram
                    .graph
                    .bfs_neighborhood(&seed_indices, self.diagram.neighborhood_depth, 100),
            )
        } else if search_mode && search_query.is_empty() {
            self.draw_diagram_toolbar(ui, large_schema, all_table_count, &tables, render_limit);
            ui.add_space(10.0);
            self.draw_diagram_empty_state(ui, all_table_count, true, false);
            return;
        } else {
            None
        };

        self.draw_diagram_toolbar(ui, large_schema, all_table_count, &tables, render_limit);
        self.draw_er_design_panel(ui);
        ui.add_space(10.0);

        self.draw_diagram_canvas(ui, active_node_indices.as_deref());
    }

    fn ensure_diagram_graph(&mut self, grid_columns: usize, node_height: f32) {
        let current_count = self.schema_explorer.schema.table_details.len();
        let graph_dirty = self.diagram.graph.nodes.len() != current_count
            || self.diagram.graph.schema_version != self.diagram.schema_version;

        if graph_dirty && current_count > 0 {
            if self.diagram.graph.nodes.is_empty() {
                // First load: build immediately so canvas starts populated without blank frame
                self.diagram.graph = ErGraph::build(
                    &self.schema_explorer.schema.table_details,
                    self.diagram.schema_version,
                    grid_columns,
                    node_height,
                );
                self.diagram.spatial_index = ErSpatialIndex::build(
                    &self.diagram.graph.nodes,
                    &self.diagram.graph.edges,
                    DEFAULT_SPATIAL_CELL_SIZE,
                );
                self.diagram.layout_state = ErLayoutState::Ready;
            } else {
                // Background worker update: keep old graph renderable and dispatch async request.
                // saturating_add: at u64::MAX the version stays MAX; subsequent invalidations
                // all produce the same version but the graph node count check (graph_dirty)
                // prevents re-dispatch unless the actual table list changes.
                self.diagram.schema_version = self.diagram.schema_version.saturating_add(1);
                let request_id = self.diagram.layout_worker.request_layout(
                    self.diagram.schema_version,
                    self.schema_explorer.schema.table_details.clone(),
                    grid_columns,
                    node_height,
                );
                self.diagram.latest_layout_request = request_id;

                // If the worker is in degraded mode (spawn failed), the request was
                // silently dropped. Transition to Failed state so the UI shows a
                // concise error while retaining the last valid graph.
                if self.diagram.layout_worker.dispatch_succeeded() {
                    self.diagram.layout_state = ErLayoutState::Computing {
                        request_id,
                        graph_version: self.diagram.schema_version,
                    };
                } else {
                    self.diagram.layout_state = ErLayoutState::Failed("ER layout worker unavailable".to_owned());
                }
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

impl DbProApp {
    fn draw_diagram_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        large_schema: bool,
        all_table_count: usize,
        tables: &[UiTableSummary],
        render_limit: usize,
    ) {
        let visible_tables = if self.diagram.show_all || !large_schema {
            all_table_count
        } else {
            tables.len().min(render_limit)
        };
        let relationship_count: usize = self.diagram.graph.edges.len();

        ui.horizontal_wrapped(|ui| {
            section_label(ui, "ER DIAGRAM", self.theme);
            ui.add_space(8.0);
            badge(
                ui,
                &plural_count(visible_tables, "table", "tables"),
                self.theme.accent_soft,
                self.theme.accent,
            );
            badge(
                ui,
                &plural_count(relationship_count, "relationship", "relationships"),
                self.theme.surface_hover,
                self.theme.text_secondary,
            );
            if matches!(self.diagram.layout_state, ErLayoutState::Computing { .. }) {
                badge(
                    ui,
                    "Arranging map…",
                    self.theme.surface_hover,
                    self.theme.text_secondary,
                );
            }
            if large_schema {
                ui.separator();
                let search_changed =
                    input(ui, &mut self.diagram.search, "Find table or column…", 220.0, self.theme).changed();
                self.diagram.show_all =
                    diagram_show_all_after_search_edit(self.diagram.show_all, &self.diagram.search, search_changed);
                let search_mode = diagram_search_mode(large_schema, self.diagram.show_all);
                if search_mode {
                    ui.label(RichText::new("Neighborhood:").small().color(self.theme.text_muted));
                    if ui
                        .selectable_label(self.diagram.neighborhood_depth == 1, "1 hop")
                        .clicked()
                    {
                        self.diagram.neighborhood_depth = 1;
                    }
                    if ui
                        .selectable_label(self.diagram.neighborhood_depth == 2, "2 hops")
                        .clicked()
                    {
                        self.diagram.neighborhood_depth = 2;
                    }
                    ui.add_space(4.0);
                    if secondary_button_with_icon(
                        ui,
                        Icon::Workflow,
                        &format!("Show all {all_table_count} tables"),
                        self.theme,
                    )
                    .clicked()
                    {
                        self.diagram.show_all = true;
                    }
                } else {
                    badge(ui, "All tables", self.theme.warning, self.theme.text_inverse);
                    if compact_button_with_icon(ui, Icon::Search, "Focus search", self.theme).clicked() {
                        self.diagram.show_all = false;
                    }
                }
            }
            ui.separator();
            let design_label = if self.diagram.design.enabled {
                "Design Mode ✓"
            } else {
                "Design Mode"
            };
            if secondary_button(ui, design_label, self.theme)
                .on_hover_text("Draft schema edits — never mutates DB until Apply")
                .clicked()
            {
                self.diagram.design.enabled = !self.diagram.design.enabled;
                if self.diagram.design.enabled {
                    let names: Vec<String> = self
                        .schema_explorer
                        .schema
                        .table_details
                        .iter()
                        .map(|t| format!("{}.{}", t.schema, t.name))
                        .collect();
                    self.diagram
                        .design
                        .set_schema_fingerprint(crate::diagram::design_mode::schema_fingerprint_from_names(&names));
                }
            }
        });
    }
}

pub(super) fn diagram_canvas_size(content_size: egui::Vec2, viewport_size: egui::Vec2) -> egui::Vec2 {
    egui::vec2(
        content_size.x.max(viewport_size.x).max(640.0),
        content_size.y.max(viewport_size.y).max(360.0),
    )
}

pub(super) fn paint_diagram_grid(painter: &egui::Painter, rect: egui::Rect, zoom: f32, theme: DbProTheme) {
    let grid_step = (24.0 * zoom).max(8.0);
    let visible = painter.clip_rect().intersect(rect);
    if visible.is_negative() {
        return;
    }
    let grid_color = theme.border_subtle.linear_multiply(0.55);
    let start_x = rect.left() + ((visible.left() - rect.left()) / grid_step).floor() * grid_step;
    let mut grid_x = start_x;
    while grid_x <= visible.right() {
        if grid_x >= visible.left() {
            painter.line_segment(
                [egui::pos2(grid_x, visible.top()), egui::pos2(grid_x, visible.bottom())],
                egui::Stroke::new(1.0, grid_color),
            );
        }
        grid_x += grid_step;
    }
    let start_y = rect.top() + ((visible.top() - rect.top()) / grid_step).floor() * grid_step;
    let mut grid_y = start_y;
    while grid_y <= visible.bottom() {
        if grid_y >= visible.top() {
            painter.line_segment(
                [egui::pos2(visible.left(), grid_y), egui::pos2(visible.right(), grid_y)],
                egui::Stroke::new(1.0, grid_color),
            );
        }
        grid_y += grid_step;
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

pub(super) fn paint_scene_edges(
    painter: &egui::Painter,
    graph: &ErGraph,
    scene: &ErRenderScene,
    viewport: &ErViewport,
    zoom: f32,
    theme: DbProTheme,
) {
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

        let bend_x = (from.x + to.x) / 2.0;
        let bend_a = egui::pos2(bend_x, from.y);
        let bend_b = egui::pos2(bend_x, to.y);
        let stroke = egui::Stroke::new((1.2 * zoom).clamp(1.0, 2.0), theme.accent);
        painter.line_segment([from, bend_a], stroke);
        painter.line_segment([bend_a, bend_b], stroke);
        painter.line_segment([bend_b, to], stroke);

        // Directional arrowhead at the destination connection point
        let arrow_len = (6.0 * zoom).clamp(4.0, 8.0);
        let arrow_half_w = arrow_len * 0.55;
        let (p1, p2) = if source_on_right {
            (
                egui::pos2(to.x - arrow_len, to.y - arrow_half_w),
                egui::pos2(to.x - arrow_len, to.y + arrow_half_w),
            )
        } else {
            (
                egui::pos2(to.x + arrow_len, to.y - arrow_half_w),
                egui::pos2(to.x + arrow_len, to.y + arrow_half_w),
            )
        };
        painter.add(egui::Shape::convex_polygon(
            vec![to, p1, p2],
            theme.accent,
            egui::Stroke::NONE,
        ));

        if scene.lod.shows_edge_labels() {
            let label = diagram_foreign_key_label(&edge.foreign_key);
            let display_label = if label.len() > 36 {
                format!("{}…", &label[..34])
            } else {
                label
            };
            let label_position = egui::pos2(bend_x, (from.y + to.y) / 2.0);
            let label_galley =
                painter.layout_no_wrap(display_label.clone(), FontId::proportional(10.0), theme.text_secondary);
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
                theme.text_secondary,
            );
        }
    }
}

pub(super) fn paint_er_node_lod(
    painter: &egui::Painter,
    node: &ErNode,
    screen_rect: egui::Rect,
    selected: bool,
    lod: ErLod,
    zoom: f32,
    theme: DbProTheme,
) {
    match lod {
        ErLod::Compact if !selected => {
            // Compact pill card: only header with table name and PK count
            painter.rect_filled(screen_rect, egui::Rounding::same(6.0), theme.surface_panel);
            painter.rect_stroke(
                screen_rect,
                egui::Rounding::same(6.0),
                egui::Stroke::new(1.0, theme.border_default),
            );
            painter.rect_filled(screen_rect, egui::Rounding::same(6.0), theme.surface_hover);
            let title = crate::components::truncate_ellipsis(&node.table.name, 20);
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

            let full_table_name = if node.table.schema.is_empty() {
                node.table.name.clone()
            } else {
                format!("{}.{}", node.table.schema, node.table.name)
            };
            let max_title_chars = if zoom > 1.0 { 24 } else { 18 };
            let display_title = crate::components::truncate_ellipsis(&full_table_name, max_title_chars);
            painter.text(
                screen_rect.min + egui::vec2(14.0 * zoom, 20.0 * zoom),
                egui::Align2::LEFT_CENTER,
                display_title,
                FontId::proportional(13.0 * zoom),
                theme.text_primary,
            );
            painter.text(
                egui::pos2(screen_rect.max.x - 12.0 * zoom, screen_rect.min.y + 20.0 * zoom),
                egui::Align2::RIGHT_CENTER,
                "TABLE",
                FontId::proportional(9.0 * zoom),
                if selected { theme.accent } else { theme.text_muted },
            );

            // Columns:
            let max_cols = if selected && lod == ErLod::Compact {
                3
            } else {
                lod.max_columns()
            };
            for (index, column) in node.table.columns.iter().take(max_cols).enumerate() {
                let row_top = screen_rect.min.y + (ER_HEADER_HEIGHT + index as f32 * ER_ROW_HEIGHT) * zoom;
                let row_rect = egui::Rect::from_min_max(
                    egui::pos2(screen_rect.min.x, row_top),
                    egui::pos2(screen_rect.max.x, row_top + ER_ROW_HEIGHT * zoom),
                );
                if index % 2 == 0 {
                    painter.rect_filled(row_rect, egui::Rounding::ZERO, theme.surface_app);
                }
                let is_foreign_key = node
                    .table
                    .foreign_keys
                    .iter()
                    .any(|foreign_key| foreign_key.from_columns.iter().any(|name| name == &column.name));
                let marker_color = if column.is_primary_key {
                    theme.warning
                } else if is_foreign_key {
                    theme.accent
                } else {
                    theme.border_strong
                };
                painter.circle_filled(
                    egui::pos2(row_rect.min.x + 13.0 * zoom, row_rect.center().y),
                    2.5 * zoom,
                    marker_color,
                );

                let col_name_max = if lod.shows_data_types() { 16 } else { 24 };
                let col_name = crate::components::truncate_ellipsis(&column.name, col_name_max);
                painter.text(
                    egui::pos2(row_rect.min.x + 23.0 * zoom, row_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    col_name,
                    FontId::proportional(11.0 * zoom),
                    theme.text_primary,
                );
                if lod.shows_data_types() {
                    let type_display = if column.is_primary_key {
                        format!("PK · {}", column.data_type)
                    } else if is_foreign_key {
                        format!("FK · {}", column.data_type)
                    } else if !column.nullable {
                        format!("{} · NN", column.data_type)
                    } else {
                        column.data_type.clone()
                    };
                    painter.text(
                        egui::pos2(row_rect.max.x - 12.0 * zoom, row_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        type_display,
                        FontId::monospace(9.5 * zoom),
                        if column.is_primary_key {
                            theme.warning
                        } else if is_foreign_key {
                            theme.accent
                        } else {
                            theme.text_secondary
                        },
                    );
                }
            }
            if node.table.columns.len() > max_cols {
                painter.text(
                    egui::pos2(
                        screen_rect.min.x + 14.0 * zoom,
                        screen_rect.min.y + (ER_HEADER_HEIGHT + ER_ROW_HEIGHT * (max_cols as f32 - 0.5)) * zoom,
                    ),
                    egui::Align2::LEFT_CENTER,
                    format!("+ {} more columns", node.table.columns.len() - max_cols),
                    FontId::proportional(10.0 * zoom),
                    theme.text_muted,
                );
            }
        }
    }
}

pub(super) fn draw_diagram_zoom_controls(
    ui: &mut egui::Ui,
    canvas_rect: egui::Rect,
    zoom: &mut f32,
    pan: &mut egui::Vec2,
    graph: &ErGraph,
    active_filter: Option<&[usize]>,
    theme: DbProTheme,
) {
    let controls_rect = egui::Rect::from_min_size(
        egui::pos2(canvas_rect.right() - 176.0, canvas_rect.top() + 12.0),
        egui::vec2(164.0, 32.0),
    );
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
                    *zoom = (*zoom - 0.1).clamp(0.5, 2.0);
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
                    *zoom = (*zoom + 0.1).clamp(0.5, 2.0);
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
                    let bounds_size = target_bounds.size();
                    if bounds_size.x > 10.0 && bounds_size.y > 10.0 {
                        let zoom_x = (canvas_rect.width() - 80.0) / bounds_size.x;
                        let zoom_y = (canvas_rect.height() - 80.0) / bounds_size.y;
                        *zoom = zoom_x.min(zoom_y).clamp(0.5, 1.5);
                        *pan = egui::vec2(
                            (canvas_rect.width() - bounds_size.x * *zoom) / 2.0 - target_bounds.left() * *zoom,
                            (canvas_rect.height() - bounds_size.y * *zoom) / 2.0 - target_bounds.top() * *zoom,
                        );
                    } else {
                        *zoom = 1.0;
                        *pan = egui::Vec2::ZERO;
                    }
                }
            });
        });
    });
}
