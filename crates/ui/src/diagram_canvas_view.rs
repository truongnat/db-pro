use super::diagram_view::{
    diagram_canvas_size, draw_diagram_zoom_controls, paint_diagram_grid, paint_er_node_lod, paint_scene_edges,
};
use super::*;
use crate::diagram::*;

impl DbProApp {
    pub(super) fn draw_diagram_empty_state(
        &mut self,
        ui: &mut egui::Ui,
        all_table_count: usize,
        search_mode: bool,
        no_matches: bool,
    ) {
        let canvas_size = egui::vec2(ui.available_width(), ui.available_height().max(360.0));
        egui::Frame {
            fill: self.theme.surface_editor,
            inner_margin: egui::Margin::ZERO,
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.set_min_size(canvas_size);
            paint_diagram_grid(ui.painter(), ui.max_rect(), 1.0, self.theme);
            ui.vertical_centered(|ui| {
                let top_space = if no_matches { 56.0 } else { 40.0 };
                ui.add_space(top_space);
                ui.label(icon_text(
                    if no_matches { Icon::Search } else { Icon::Workflow },
                    if no_matches {
                        "No matching tables"
                    } else if search_mode {
                        "Focus the schema map"
                    } else {
                        "No schema map yet"
                    },
                    self.theme.text_secondary,
                ));
                ui.add_space(8.0);
                let description = if no_matches {
                    "Try a different table or column name.".to_owned()
                } else if search_mode {
                    format!(
                        "This schema has {all_table_count} tables. Search by table or column to open a focused neighborhood map, or show all tables explicitly."
                    )
                } else {
                    "Connect to a database and load its tables to see the relationship map.".to_owned()
                };
                ui.label(RichText::new(description).small().color(self.theme.text_muted));
                if no_matches {
                    ui.add_space(10.0);
                    if compact_button(ui, "Clear search", self.theme).clicked() {
                        self.schema.diagram.search.clear();
                    }
                }
                ui.add_space(56.0);
            });
        });
    }

    pub(super) fn draw_diagram_canvas(&mut self, ui: &mut egui::Ui, active_filter: Option<&[usize]>) {
        let world_size = if let Some(filter) = active_filter {
            self.schema.diagram.graph.active_subset_bounds(filter).size()
        } else {
            self.schema.diagram.graph.world_bounds.size()
        };
        let viewport_size = egui::vec2(ui.available_width(), ui.available_height());
        let canvas_size = diagram_canvas_size(world_size * self.schema.diagram.zoom, viewport_size);
        let zoom = self.schema.diagram.zoom;
        let pan = self.schema.diagram.pan;
        let theme = self.theme;

        egui::Frame {
            fill: theme.surface_editor,
            inner_margin: egui::Margin::ZERO,
            ..Default::default()
        }
        .show(ui, |ui| {
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                let (response, painter) = ui.allocate_painter(canvas_size, Sense::click_and_drag());
                paint_diagram_grid(&painter, response.rect, zoom, theme);

                let viewport = ErViewport::new(pan, zoom, response.rect.min);
                let scene = prepare_render_scene(
                    &self.schema.diagram.graph,
                    &self.schema.diagram.spatial_index,
                    &viewport,
                    response.rect,
                    active_filter,
                );

                paint_scene_edges(&painter, &self.schema.diagram.graph, &scene, &viewport, zoom, theme);

                let selected_table_name = self.schema.explorer.selected_table.as_deref();
                for &node_id in &scene.visible_nodes {
                    if let Some(node) = self.schema.diagram.graph.nodes.get(node_id) {
                        let screen_rect = viewport.world_to_screen_rect(node.world_rect);
                        let selected = selected_table_name == Some(node.table.name.as_str());
                        paint_er_node_lod(&painter, node, screen_rect, selected, scene.lod, zoom, theme);
                    }
                }

                draw_diagram_zoom_controls(
                    ui,
                    response.rect,
                    &mut self.schema.diagram.zoom,
                    &mut self.schema.diagram.pan,
                    &self.schema.diagram.graph,
                    active_filter,
                    theme,
                );
                self.update_diagram_pan(&response);

                if response.clicked() {
                    if let Some(pointer) = response.interact_pointer_pos() {
                        let world_pos = viewport.screen_to_world_pos(pointer);
                        if let Some(hit_id) = self
                            .schema
                            .diagram
                            .spatial_index
                            .hit_test_node(world_pos, &self.schema.diagram.graph.nodes)
                        {
                            let table_name = self
                                .schema
                                .diagram
                                .graph
                                .nodes
                                .get(hit_id)
                                .map(|node| node.table.name.clone());
                            if let Some(name) = table_name {
                                self.open_diagram_table(&name);
                            }
                        }
                    }
                }
            });
        });
    }

    fn update_diagram_pan(&mut self, response: &egui::Response) {
        if response.drag_started() {
            self.schema.diagram.pan_origin = Some(self.schema.diagram.pan);
        }
        if response.dragged() {
            if let Some(origin) = self.schema.diagram.pan_origin {
                self.schema.diagram.pan = origin + response.drag_delta();
            }
        }
        if response.drag_stopped() {
            self.schema.diagram.pan_origin = None;
        }
    }

    fn open_diagram_table(&mut self, table: &str) {
        if self.schema.explorer.selected_table.as_deref() != Some(table)
            && !self.table.mutation.staged_changes.is_empty()
        {
            self.feedback.runtime_message = "Apply or discard staged changes before opening another table".to_owned();
            return;
        }
        let scope = TableDataState::layout_scope(
            self.connection.lifecycle.active_connection_id(),
            self.active_schema(),
            self.schema.explorer.selected_table.as_deref(),
        );
        self.table.data.persist_layout(scope);
        self.schema.explorer.selected_table = Some(table.to_owned());
        let scope = TableDataState::layout_scope(
            self.connection.lifecycle.active_connection_id(),
            self.active_schema(),
            self.schema.explorer.selected_table.as_deref(),
        );
        self.table.data.restore_layout(scope);
        self.schema.explorer.selected_schema_object = None;
        self.schema.explorer.schema_object_view = SchemaObjectView::Definition;
        self.table.state.table_info = None;
        self.table.state.table_ddl = None;
        self.table.state.table_info_error = None;
        self.table.state.table_ddl_error = None;
        self.table.state.ddl_execute_confirmation = false;
        self.table.state.ddl_execution_request = None;
        self.table.data_query.reset_for_table();
        self.table.state.table_info_request = None;
        self.table.state.table_ddl_request = None;
        self.table.data.selected_cell = None;
        self.table.data.selected_row = None;
        self.table.data.selected_rows.clear();
        self.table.data.selection_anchor_row = None;
        self.table.data.selection_anchor_cell = None;
        self.table.editing.data_editing_cell = None;
        self.table.editing.data_edit_value.clear();
        self.table.editing.data_edit_error = None;
        self.table.editing.data_delete_confirmation = false;
        self.table.editing.discard_changes_confirmation = false;
        self.table.state.table_view = TableView::Structure;
        self.set_active_query_text(format!("SELECT *\nFROM {table}\nLIMIT 100;"));
        self.request_table_info();
        self.workspace.activity = Activity::Explorer;
        self.workspace.sidebar_open = true;
        self.workspace.active_tab = WorkspaceTab::Table;
    }
}
