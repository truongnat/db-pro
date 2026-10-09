use super::diagram_view::{
    draw_diagram_zoom_controls, fit_diagram_viewport, paint_diagram_grid, paint_er_node_lod, paint_scene_edges,
    zoom_controls_rect, DiagramAction, DiagramViewContext,
};
use super::*;
use crate::diagram::*;
use lucide_icons::Icon;
use std::time::Duration;

/// Delay before the canvas re-fits to the search-neighborhood subset after the
/// last keystroke, so typing does not jerk the viewport per character.
const SEARCH_FIT_DEBOUNCE: Duration = Duration::from_millis(350);

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
        let viewport = ErViewport::new(egui::Vec2::ZERO, 1.0, ui.max_rect().min);
        paint_diagram_grid(ui.painter(), &viewport, ui.max_rect(), ctx.theme);
        ui.vertical_centered(|ui| {
            let top_space = if no_matches { 56.0 } else { 40.0 };
            ui.add_space(top_space);
            let arranging = search_mode
                && !no_matches
                && matches!(ctx.diagram.layout_state, crate::diagram::ErLayoutState::Computing { .. });
            ui.label(icon_text(
                if no_matches { Icon::Search } else { Icon::Workflow },
                if no_matches {
                    "No matching tables"
                } else if arranging {
                    "Loading the schema map"
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
            } else if search_mode && matches!(ctx.diagram.layout_state, crate::diagram::ErLayoutState::Computing { .. })
            {
                format!("Arranging {all_table_count} tables…")
            } else if search_mode {
                format!(
                    "This schema has {all_table_count} tables. Search by table or column \
                     to open a focused neighborhood map, or show all tables explicitly."
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
    let theme = ctx.theme;
    let mut action = None;

    egui::Frame {
        fill: theme.surface_editor,
        inner_margin: egui::Margin::ZERO,
        ..Default::default()
    }
    .show(ui, |ui| {
        let canvas_size = egui::vec2(ui.available_width(), ui.available_height().max(360.0));
        let (response, painter) = ui.allocate_painter(canvas_size, Sense::click_and_drag());
        let viewport_rect = response.rect;

        apply_initial_viewport(ctx, viewport_rect, active_filter);

        // Gestures must not leak through the overlay widgets stacked on the canvas.
        let overlay_rects = [zoom_controls_rect(viewport_rect), minimap_rect(viewport_rect)];
        let pointer_on_overlay = response
            .interact_pointer_pos()
            .or_else(|| response.hover_pos())
            .is_some_and(|pointer| overlay_rects.iter().any(|rect| rect.contains(pointer)));

        if !pointer_on_overlay {
            handle_zoom_input(ctx, ui, &response, viewport_rect);
        }

        let viewport = ErViewport::new(ctx.diagram.pan, ctx.diagram.zoom, viewport_rect.min);
        ctx.diagram.hovered_node = if pointer_on_overlay {
            None
        } else {
            response.hover_pos().and_then(|pointer| {
                ctx.diagram
                    .spatial_index
                    .hit_test_node(viewport.screen_to_world_pos(pointer), &ctx.diagram.graph.nodes)
            })
        };

        if !pointer_on_overlay || ctx.diagram.drag_node.is_some() || ctx.diagram.pan_origin.is_some() {
            handle_drag_input(ctx, &response, viewport, &overlay_rects);
        }
        update_cursor(ui, ctx.diagram, response.hovered() && !pointer_on_overlay);

        paint_diagram_grid(&painter, &viewport, viewport_rect, theme);

        let scene = prepare_render_scene(
            &ctx.diagram.graph,
            &ctx.diagram.spatial_index,
            &viewport,
            viewport_rect,
            active_filter,
        );

        let highlight_node = highlight_node(ctx, &scene);
        paint_scene_edges(
            &painter,
            &ctx.diagram.graph,
            &scene,
            &viewport,
            theme,
            highlight_node,
        );

        let selected_table_name = ctx.explorer.selected_table.as_deref();
        for &node_id in &scene.visible_nodes {
            if let Some(node) = ctx.diagram.graph.nodes.get(node_id) {
                let screen_rect = viewport.world_to_screen_rect(node.world_rect);
                let selected = selected_table_name == Some(node.table.name.as_str())
                    || ctx.diagram.selected_node == Some(node_id);
                let hovered = ctx.diagram.hovered_node == Some(node_id);
                paint_er_node_lod(&painter, node, screen_rect, selected, hovered, scene.lod, theme);
            }
        }

        // Single click pins the relationship highlight on that table (click
        // empty canvas to unpin); only a double click opens the table.
        if response.clicked() && ctx.diagram.drag_node.is_none() {
            ctx.diagram.selected_node = ctx.diagram.hovered_node;
        }
        if response.double_clicked() && ctx.diagram.drag_node.is_none() {
            if let Some(node_id) = ctx.diagram.hovered_node {
                if let Some(name) = ctx
                    .diagram
                    .graph
                    .nodes
                    .get(node_id)
                    .map(|node| node.table.name.clone())
                {
                    action = Some(DiagramAction::OpenTable(name));
                }
            }
        }

        // Context menu: the hovered table gets Open/Copy Name, and the canvas
        // always offers the same fit/reset actions the zoom strip exposes.
        // The node target is stored under the shared menu id so the popup
        // survives the pointer leaving the node.
        let menu_target_id = response.id.with("floating_ctx_menu").with("target");
        if is_context_menu_triggered(&response, ui) {
            let table = if pointer_on_overlay {
                None
            } else {
                ctx.diagram
                    .hovered_node
                    .and_then(|id| ctx.diagram.graph.nodes.get(id))
                    .map(|node| node.table.name.clone())
            };
            ui.ctx().data_mut(|data| data.insert_temp(menu_target_id, table));
        }
        let menu_target: Option<String> = ui
            .ctx()
            .data(|data| data.get_temp::<Option<String>>(menu_target_id))
            .unwrap_or_default();
        let mut menu_action = None;
        {
            let diagram = &mut ctx.diagram;
            context_action_menu(ui, &response, theme, |ui, close_menu| {
                if let Some(name) = &menu_target {
                    if ctx_menu_item(
                        ui,
                        Some(Icon::ExternalLink),
                        &format!("Open Table `{name}`"),
                        None,
                        theme.text_primary,
                        theme,
                    )
                    .clicked()
                    {
                        menu_action = Some(DiagramAction::OpenTable(name.clone()));
                        *close_menu = true;
                    }
                    if ctx_menu_item(ui, Some(Icon::Copy), "Copy Table Name", None, theme.text_primary, theme)
                        .clicked()
                    {
                        ui.ctx().output_mut(|output| output.commands.push(egui::OutputCommand::CopyText(name.clone())));
                        *close_menu = true;
                    }
                    ui.separator();
                }
                if ctx_menu_item(ui, Some(Icon::Maximize2), "Fit Diagram to View", None, theme.text_primary, theme)
                    .clicked()
                {
                    let bounds = if let Some(filter) = active_filter {
                        diagram.graph.active_subset_bounds(filter)
                    } else {
                        diagram.graph.world_bounds
                    };
                    if let Some((fit_zoom, fit_pan)) = fit_diagram_viewport(bounds, viewport_rect) {
                        diagram.zoom = fit_zoom;
                        diagram.pan = fit_pan;
                    }
                    *close_menu = true;
                }
                if ctx_menu_item(
                    ui,
                    Some(Icon::RotateCcw),
                    "Reset Zoom & Pan",
                    None,
                    theme.text_primary,
                    theme,
                )
                .clicked()
                {
                    diagram.zoom = 1.0;
                    diagram.pan = egui::Vec2::ZERO;
                    *close_menu = true;
                }
            });
        }
        if let Some(menu_action) = menu_action {
            action = Some(menu_action);
        }

        draw_diagram_zoom_controls(
            ui,
            viewport_rect,
            &mut ctx.diagram.zoom,
            &mut ctx.diagram.pan,
            &ctx.diagram.graph,
            active_filter,
            theme,
        );
        draw_minimap(ctx, ui, viewport_rect, &viewport, minimap_rect(viewport_rect));
        maybe_fit_search(ctx, ui, viewport_rect, active_filter);
        sync_layout_snapshot(ctx);
    });

    action
}

/// Node whose relationships stay highlighted while everything else dims:
/// an active drag wins, then the click-pinned selection, then hover, then
/// the explorer's selected table.
fn highlight_node(ctx: &DiagramViewContext<'_>, scene: &ErRenderScene) -> Option<usize> {
    if let Some((node_id, _)) = ctx.diagram.drag_node {
        return Some(node_id);
    }
    if let Some(node_id) = ctx.diagram.selected_node {
        return Some(node_id);
    }
    if let Some(node_id) = ctx.diagram.hovered_node {
        return Some(node_id);
    }
    let selected = ctx.explorer.selected_table.as_deref()?;
    scene.visible_nodes.iter().copied().find(|&id| {
        ctx.diagram
            .graph
            .nodes
            .get(id)
            .is_some_and(|node| node.table.name == selected)
    })
}

/// Fits the viewport once per graph — either restoring the persisted snapshot
/// for this connection or fitting the whole world. Runs before the first paint
/// so the diagram never flashes at 100% then jumps.
fn apply_initial_viewport(
    ctx: &mut DiagramViewContext<'_>,
    viewport_rect: egui::Rect,
    active_filter: Option<&[usize]>,
) {
    if ctx.diagram.auto_fit_done
        || ctx.diagram.graph.nodes.is_empty()
        || matches!(ctx.diagram.layout_state, ErLayoutState::Computing { .. })
    {
        return;
    }
    ctx.diagram.auto_fit_done = true;
    if let Some(snapshot) = ctx
        .connection_id
        .as_deref()
        .and_then(|id| ctx.diagram.saved_layouts.get(id))
    {
        if let Some(zoom) = snapshot.zoom {
            ctx.diagram.zoom = zoom.clamp(ER_MIN_ZOOM, ER_MAX_ZOOM);
        }
        if let Some([x, y]) = snapshot.pan {
            ctx.diagram.pan = egui::vec2(x, y);
        }
        if snapshot.zoom.is_some() || snapshot.pan.is_some() {
            return;
        }
    }
    let target_bounds = match active_filter {
        Some(filter) => ctx.diagram.graph.active_subset_bounds(filter),
        None => ctx.diagram.graph.world_bounds,
    };
    if let Some((zoom, pan)) = super::diagram_view::fit_diagram_viewport(target_bounds, viewport_rect) {
        ctx.diagram.zoom = zoom;
        ctx.diagram.pan = pan;
    }
    if let Some(capture_zoom) = ctx.diagram.capture_zoom_override {
        let zoom = capture_zoom.clamp(ER_MIN_ZOOM, ER_MAX_ZOOM);
        ctx.diagram.zoom = zoom;
        ctx.diagram.pan = viewport_rect.center().to_vec2()
            - viewport_rect.min.to_vec2()
            - target_bounds.center().to_vec2() * zoom;
    }
}

/// After typing pauses, re-fit the viewport to the search-neighborhood subset
/// (or back to the whole world once the query is cleared).
fn maybe_fit_search(
    ctx: &mut DiagramViewContext<'_>,
    ui: &egui::Ui,
    viewport_rect: egui::Rect,
    active_filter: Option<&[usize]>,
) {
    let query = ctx.diagram.search.trim().to_ascii_lowercase();
    if ctx.diagram.fitted_search == query {
        return;
    }
    let Some(changed_at) = ctx.diagram.search_changed_at else {
        return;
    };
    if changed_at.elapsed() < SEARCH_FIT_DEBOUNCE {
        ui.ctx().request_repaint_after(Duration::from_millis(60));
        return;
    }
    ctx.diagram.fitted_search = query.clone();
    // Empty query refits the whole world; otherwise fit the matched subset.
    let bounds = active_filter
        .map(|filter| ctx.diagram.graph.active_subset_bounds(filter))
        .unwrap_or(ctx.diagram.graph.world_bounds);
    if let Some((zoom, pan)) = super::diagram_view::fit_diagram_viewport(bounds, viewport_rect) {
        ctx.diagram.zoom = zoom;
        ctx.diagram.pan = pan;
    }
}

/// Mouse wheel zooms toward the pointer; trackpad pinch feeds `zoom_delta`.
fn handle_zoom_input(
    ctx: &mut DiagramViewContext<'_>,
    ui: &egui::Ui,
    response: &egui::Response,
    viewport_rect: egui::Rect,
) {
    if !response.hovered() {
        return;
    }
    let (scroll_y, pinch) = ui.ctx().input(|input| (input.smooth_scroll_delta.y, input.zoom_delta()));
    let factor = pinch * (scroll_y * 0.0022).exp();
    if (factor - 1.0).abs() < f32::EPSILON {
        return;
    }
    let new_zoom = (ctx.diagram.zoom * factor).clamp(ER_MIN_ZOOM, ER_MAX_ZOOM);
    if (new_zoom - ctx.diagram.zoom).abs() < f32::EPSILON {
        return;
    }
    let anchor = response
        .hover_pos()
        .unwrap_or_else(|| viewport_rect.center());
    let world_anchor = (anchor - viewport_rect.min - ctx.diagram.pan) / ctx.diagram.zoom;
    ctx.diagram.pan = anchor - viewport_rect.min - world_anchor * new_zoom;
    ctx.diagram.zoom = new_zoom;
}

/// Routes a pointer drag to either a node move or a canvas pan. Node drag wins
/// when the press starts on a table card; the grab offset keeps the card glued
/// to the same point under the cursor for the whole gesture. Presses that start
/// on overlay widgets (zoom strip, minimap) never reach the canvas.
fn handle_drag_input(
    ctx: &mut DiagramViewContext<'_>,
    response: &egui::Response,
    viewport: ErViewport,
    overlay_rects: &[egui::Rect],
) {
    if response.drag_started() {
        // `press_origin` is where the button went down — `interact_pointer_pos`
        // has already moved a few px by the time the drag is decided, and the
        // press point is also the right place to pick the grabbed node.
        let press_screen = response
            .ctx
            .input(|input| input.pointer.press_origin())
            .filter(|pointer| !overlay_rects.iter().any(|rect| rect.contains(*pointer)));
        let hit = press_screen
            .map(|pointer| viewport.screen_to_world_pos(pointer))
            .and_then(|world| {
                let node_id = ctx
                    .diagram
                    .spatial_index
                    .hit_test_node(world, &ctx.diagram.graph.nodes)?;
                let grab = world - ctx.diagram.graph.nodes.get(node_id)?.world_rect.min;
                Some((node_id, grab))
            });
        ctx.diagram.drag_node = hit;
        ctx.diagram.pan_origin = if hit.is_none() {
            press_screen.map(|press| (ctx.diagram.pan, press))
        } else {
            None
        };
    }

    if response.dragged() {
        if let Some((node_id, grab_offset)) = ctx.diagram.drag_node {
            if let Some(pointer) = response.interact_pointer_pos() {
                let new_min = viewport.screen_to_world_pos(pointer) - grab_offset;
                ctx.diagram.graph.move_node(node_id, new_min);
            }
        } else if let Some((grab_pan, press)) = ctx.diagram.pan_origin {
            if let Some(pointer) = response.interact_pointer_pos() {
                ctx.diagram.pan = grab_pan + (pointer - press);
            }
        }
    }

    if response.drag_stopped() {
        if let Some((node_id, _)) = ctx.diagram.drag_node.take() {
            ctx.diagram.graph.recompute_world_bounds();
            ctx.diagram.spatial_index = ErSpatialIndex::build(
                &ctx.diagram.graph.nodes,
                &ctx.diagram.graph.edges,
                DEFAULT_SPATIAL_CELL_SIZE,
            );
            if let (Some(conn), Some(node)) = (
                ctx.connection_id.as_deref(),
                ctx.diagram.graph.nodes.get(node_id),
            ) {
                let key = er_table_key(&node.table.schema, &node.table.name);
                ctx.diagram
                    .saved_layouts
                    .entry(conn.to_owned())
                    .or_default()
                    .positions
                    .insert(key, [node.world_rect.min.x, node.world_rect.min.y]);
            }
        }
        ctx.diagram.pan_origin = None;
    }
}

fn update_cursor(ui: &egui::Ui, diagram: &DiagramState, canvas_hovered: bool) {
    let cursor = if diagram.drag_node.is_some() || diagram.pan_origin.is_some() {
        egui::CursorIcon::Grabbing
    } else if !canvas_hovered {
        return;
    } else if diagram.hovered_node.is_some() {
        egui::CursorIcon::Move
    } else {
        egui::CursorIcon::Grab
    };
    ui.ctx().output_mut(|output| output.cursor_icon = cursor);
}

/// Writes the live zoom/pan into the per-connection snapshot so the next open
/// restores the same view. Node positions are recorded separately on drop.
fn sync_layout_snapshot(ctx: &mut DiagramViewContext<'_>) {
    let Some(conn) = ctx.connection_id.as_deref() else {
        return;
    };
    let snapshot = ctx.diagram.saved_layouts.entry(conn.to_owned()).or_default();
    let pan = [ctx.diagram.pan.x, ctx.diagram.pan.y];
    let zoom = ctx.diagram.zoom;
    if snapshot.zoom != Some(zoom) || snapshot.pan != Some(pan) {
        snapshot.zoom = Some(zoom);
        snapshot.pan = Some(pan);
    }
}

/// Screen rect of the minimap overlay (bottom-right corner).
fn minimap_rect(viewport_rect: egui::Rect) -> egui::Rect {
    let size = egui::vec2(168.0, 112.0);
    egui::Rect::from_min_size(
        egui::pos2(
            viewport_rect.right() - size.x - 10.0,
            viewport_rect.bottom() - size.y - 10.0,
        ),
        size,
    )
}

/// Bottom-right overview of the whole world; click or drag inside it recenters
/// the canvas viewport on that world point.
fn draw_minimap(
    ctx: &mut DiagramViewContext<'_>,
    ui: &mut egui::Ui,
    viewport_rect: egui::Rect,
    viewport: &ErViewport,
    minimap_rect: egui::Rect,
) {
    if ctx.diagram.graph.nodes.is_empty() {
        return;
    }
    let theme = ctx.theme;
    let mut minimap_ui = ui.new_child(egui::UiBuilder::new().max_rect(minimap_rect));
    minimap_ui.set_clip_rect(minimap_ui.clip_rect().intersect(minimap_rect));
    {
        let ui = &mut minimap_ui;
        egui::Frame {
            fill: theme.surface_floating,
            inner_margin: egui::Margin::same(5.0 as i8),
            corner_radius: egui::CornerRadius::same(6.0 as u8),
            stroke: egui::Stroke::new(1.0, theme.border_subtle),
            ..Default::default()
        }
        .show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
            let map_rect = response.rect;
            let world = ctx.diagram.graph.world_bounds;
            let scale = (map_rect.width() / world.width())
                .min(map_rect.height() / world.height())
                .max(f32::EPSILON);
            let map_center = map_rect.center();
            let to_map = |world_pos: egui::Pos2| -> egui::Pos2 {
                map_center + (world_pos - world.center()) * scale
            };

            for node in &ctx.diagram.graph.nodes {
                let mini = egui::Rect::from_min_max(to_map(node.world_rect.min), to_map(node.world_rect.max));
                let highlighted = ctx.diagram.hovered_node == Some(node.id)
                    || ctx
                        .explorer
                        .selected_table
                        .as_deref()
                        .is_some_and(|name| name == node.table.name);
                painter.rect_filled(
                    mini,
                    egui::CornerRadius::same(1.0 as u8),
                    if highlighted {
                        theme.accent
                    } else {
                        theme.text_muted.linear_multiply(0.55)
                    },
                );
            }

            let visible_world = viewport.visible_world_rect(viewport_rect, 0.0);
            let view_rect = egui::Rect::from_min_max(to_map(visible_world.min), to_map(visible_world.max));
            painter.rect_stroke(
                view_rect.intersect(map_rect),
                egui::CornerRadius::same(2.0 as u8),
                egui::Stroke::new(1.0, theme.accent), egui::StrokeKind::Inside);

            if (response.dragged() || response.clicked()) && !response.drag_stopped() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let world_target = world.center() + (pointer - map_center) / scale;
                    ctx.diagram.pan = viewport_rect.center().to_vec2()
                        - viewport_rect.min.to_vec2()
                        - world_target.to_vec2() * ctx.diagram.zoom;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dragging_canvas_updates_pan() {
        let context = egui::Context::default();
        DbProTheme::install_fonts(&context);
        let explorer = SchemaExplorerState::default();
        let mut diagram = DiagramState::default();
        let start = egui::pos2(160.0, 160.0);
        run_canvas_frame(&context, &explorer, &mut diagram, vec![egui::Event::PointerMoved(start)]);
        run_canvas_frame(&context, &explorer, &mut diagram, vec![press_event(start)]);
        // Two separate move frames: a per-frame delta accumulation bug would
        // leave pan at the last frame's (10, 5) instead of the full (74, 53).
        run_canvas_frame(&context, &explorer, &mut diagram, vec![moved_to(start + egui::vec2(64.0, 48.0))]);
        run_canvas_frame(&context, &explorer, &mut diagram, vec![moved_to(start + egui::vec2(74.0, 53.0))]);
        run_canvas_frame(&context, &explorer, &mut diagram, vec![release_event(start + egui::vec2(74.0, 53.0))]);

        assert_eq!(
            diagram.pan,
            egui::vec2(74.0, 53.0),
            "pan must track the full distance from the press point, not per-frame motion"
        );
    }

    fn run_canvas_frame(
        context: &egui::Context,
        explorer: &SchemaExplorerState,
        diagram: &mut DiagramState,
        events: Vec<egui::Event>,
    ) {
        let mut view = DiagramViewContext {
            theme: DbProTheme::dark(),
            diagram,
            explorer,
            active_driver: "SQLite",
            connected: true,
            connection_id: None,
        };
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900.0, 600.0));
        let _ = crate::test_frame::frame(&context, egui::RawInput {
            screen_rect: Some(screen),
            events,
            ..Default::default()
        }, |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                let (response, _) =
                    ui.allocate_painter(egui::vec2(880.0, 560.0), egui::Sense::click_and_drag());
                let viewport = ErViewport::new(view.diagram.pan, view.diagram.zoom, response.rect.min);
                handle_drag_input(&mut view, &response, viewport, &[]);
            });
        });
    }

    fn moved_to(pos: egui::Pos2) -> egui::Event {
        egui::Event::PointerMoved(pos)
    }

    fn press_event(pos: egui::Pos2) -> egui::Event {
        button_event(pos, true)
    }

    fn release_event(pos: egui::Pos2) -> egui::Event {
        button_event(pos, false)
    }

    fn button_event(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }
}
