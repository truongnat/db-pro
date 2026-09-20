use super::diagram_view::{
    diagram_canvas_size, draw_diagram_zoom_controls, paint_diagram_grid, paint_er_node_lod, paint_scene_edges,
    DiagramAction, DiagramViewContext,
};
use super::*;
use crate::diagram::*;

pub(super) fn draw_diagram_empty_state(
    ctx: &mut DiagramViewContext<'_>,
    ui: &mut egui::Ui,
    all_table_count: usize,
    search_mode: bool,
    no_matches: bool,
) {
    let canvas_size = egui::vec2(ui.available_width(), ui.available_height().max(360.0));
    egui::Frame {
        fill: ctx.theme.surface_editor,
        inner_margin: egui::Margin::ZERO,
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.set_min_size(canvas_size);
        paint_diagram_grid(ui.painter(), ui.max_rect(), 1.0, ctx.theme);
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
                ctx.theme.text_secondary,
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
            ui.label(RichText::new(description).small().color(ctx.theme.text_muted));
            if no_matches {
                ui.add_space(10.0);
                if compact_button(ui, "Clear search", ctx.theme).clicked() {
                    ctx.diagram.search.clear();
                }
            }
            ui.add_space(56.0);
        });
    });
}

pub(super) fn draw_diagram_canvas(
    ctx: &mut DiagramViewContext<'_>,
    ui: &mut egui::Ui,
    active_filter: Option<&[usize]>,
) -> Option<DiagramAction> {
    let world_size = if let Some(filter) = active_filter {
        ctx.diagram.graph.active_subset_bounds(filter).size()
    } else {
        ctx.diagram.graph.world_bounds.size()
    };
    let viewport_size = egui::vec2(ui.available_width(), ui.available_height());
    let canvas_size = diagram_canvas_size(world_size * ctx.diagram.zoom, viewport_size);
    let zoom = ctx.diagram.zoom;
    let pan = ctx.diagram.pan;
    let theme = ctx.theme;
    let mut action = None;

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
                &ctx.diagram.graph,
                &ctx.diagram.spatial_index,
                &viewport,
                response.rect,
                active_filter,
            );

            paint_scene_edges(&painter, &ctx.diagram.graph, &scene, &viewport, zoom, theme);

            let selected_table_name = ctx.explorer.selected_table.as_deref();
            for &node_id in &scene.visible_nodes {
                if let Some(node) = ctx.diagram.graph.nodes.get(node_id) {
                    let screen_rect = viewport.world_to_screen_rect(node.world_rect);
                    let selected = selected_table_name == Some(node.table.name.as_str());
                    paint_er_node_lod(&painter, node, screen_rect, selected, scene.lod, zoom, theme);
                }
            }

            draw_diagram_zoom_controls(
                ui,
                response.rect,
                &mut ctx.diagram.zoom,
                &mut ctx.diagram.pan,
                &ctx.diagram.graph,
                active_filter,
                theme,
            );
            update_diagram_pan(ctx, &response);

            if response.clicked() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let world_pos = viewport.screen_to_world_pos(pointer);
                    if let Some(hit_id) = ctx
                        .diagram
                        .spatial_index
                        .hit_test_node(world_pos, &ctx.diagram.graph.nodes)
                    {
                        if let Some(name) = ctx.diagram.graph.nodes.get(hit_id).map(|node| node.table.name.clone()) {
                            action = Some(DiagramAction::OpenTable(name));
                        }
                    }
                }
            }
        });
    });

    action
}

fn update_diagram_pan(ctx: &mut DiagramViewContext<'_>, response: &egui::Response) {
    if response.drag_started() {
        ctx.diagram.pan_origin = Some(ctx.diagram.pan);
    }
    if response.dragged() {
        if let Some(origin) = ctx.diagram.pan_origin {
            ctx.diagram.pan = origin + response.drag_delta();
        }
    }
    if response.drag_stopped() {
        ctx.diagram.pan_origin = None;
    }
}
