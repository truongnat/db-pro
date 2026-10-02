//! Sidebar shell layout, chrome and resize interaction.
use super::super::*;
use super::sidebar_chrome_view::{SidebarChromeAction, SidebarChromeContext};
use egui::{vec2, Pos2, Rect, Sense, Stroke};
use std::{cell::RefCell, rc::Rc};

const SIDEBAR_CLIP_BLEED: f32 = 1.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_scroll_preserves_persisted_tree_namespace() {
        let capture = |native| {
            let ctx = egui::Context::default();
            let runtime = RefCell::new(crate::native_runtime_shell::RsUiShellRuntime::default());
            let mut id = None;
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(260.0, 300.0))),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let mut content = |ui: &mut egui::Ui| {
                            id = Some(ui.make_persistent_id(("codex_tbl_folder", "public")));
                        };
                        if native {
                            draw_explorer_scroll(&runtime, ui, &mut content);
                        } else {
                            egui::ScrollArea::vertical()
                                .id_salt("codex_navigator_scroll")
                                .show(ui, content);
                        }
                    });
                },
            );
            id.expect("tree content ID")
        };
        assert_eq!(capture(false), capture(true));
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum SidebarSurfaceAction {
    Chrome(SidebarChromeAction),
    Resize(f32),
}

pub(super) struct SidebarSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) sidebar_width: f32,
    pub(super) active_name: &'a str,
    pub(super) command_palette_shortcut: &'a str,
    pub(super) new_connection_shortcut: &'a str,
    pub(super) new_query_shortcuts: &'a [String],
    pub(super) runtime: Rc<RefCell<crate::native_runtime_shell::RsUiShellRuntime>>,
}

struct SidebarPanel {
    left: f32,
    y_range: egui::Rangef,
    full: Rect,
    layer_id: egui::LayerId,
}

pub(super) fn draw<F>(
    context: &SidebarSurfaceContext<'_>,
    egui_context: &egui::Context,
    mut draw_activity: F,
) -> Vec<SidebarSurfaceAction>
where
    F: FnMut(&mut egui::Ui),
{
    let panel = draw_panel(context, egui_context);
    let mut actions = draw_content(context, egui_context, &panel, &mut draw_activity);
    if let Some(width) = draw_resize_handle(context, egui_context, &panel) {
        actions.push(SidebarSurfaceAction::Resize(width));
    }
    actions
}

fn draw_panel(context: &SidebarSurfaceContext<'_>, egui_context: &egui::Context) -> SidebarPanel {
    let viewport = egui_context.screen_rect();
    let sidebar_width = crate::native_runtime_shell::layout_shell(
        rs_ui_core::Size::new(viewport.width(), viewport.height()),
        context.sidebar_width,
    )
    .map(|regions| regions.sidebar.width())
    .unwrap_or(context.sidebar_width);
    let response = egui::SidePanel::left("sidebar")
        .resizable(false)
        .exact_width(sidebar_width)
        .show_separator_line(false)
        .frame(egui::Frame {
            fill: context.theme.surface_panel,
            inner_margin: egui::Margin::ZERO,
            outer_margin: egui::Margin::ZERO,
            stroke: egui::Stroke::NONE,
            rounding: egui::Rounding::ZERO,
            shadow: egui::Shadow::NONE,
        })
        .show(egui_context, |ui| {
            let panel_origin = ui.max_rect().min;
            let full = Rect::from_min_size(panel_origin, vec2(sidebar_width, ui.max_rect().height()));
            ui.painter()
                .rect_filled(full, egui::Rounding::ZERO, context.theme.surface_panel);
            ui.allocate_rect(full, Sense::hover());
            (full, ui.layer_id())
        });
    let (full, layer_id) = response.inner;
    let left = response.response.rect.left();
    let y_range = response.response.rect.y_range();
    SidebarPanel {
        left,
        y_range,
        full,
        layer_id,
    }
}

fn draw_content<F>(
    context: &SidebarSurfaceContext<'_>,
    egui_context: &egui::Context,
    panel: &SidebarPanel,
    draw_activity: &mut F,
) -> Vec<SidebarSurfaceAction>
where
    F: FnMut(&mut egui::Ui),
{
    let content_rect = content_rect(context, panel);
    // Keep pointer hit-testing on the panel's layer so the tree ScrollArea can receive wheel input.
    let mut ui = sidebar_ui(egui_context, content_rect, panel.layer_id);
    let chrome = SidebarChromeContext {
        theme: context.theme,
        active_name: context.active_name,
        command_palette_shortcut: context.command_palette_shortcut,
        new_connection_shortcut: context.new_connection_shortcut,
        new_query_shortcuts: context.new_query_shortcuts,
    };
    let actions = chrome
        .draw(&mut ui)
        .into_iter()
        .map(SidebarSurfaceAction::Chrome)
        .collect::<Vec<_>>();
    ui.add_space(SPACE_SM);
    ui.separator();
    ui.add_space(SPACE_XS);
    draw_activity(&mut ui);
    actions
}

fn content_rect(context: &SidebarSurfaceContext<'_>, panel: &SidebarPanel) -> Rect {
    let pad_left = SPACE_SM;
    let pad_right = SPACE_MD;
    let pad_y = SPACE_SM;
    let content_w = (context.sidebar_width - pad_left - pad_right).max(0.0);
    Rect::from_min_size(
        Pos2::new(panel.full.left() + pad_left, panel.full.top() + pad_y),
        vec2(content_w, (panel.full.height() - 2.0 * pad_y).max(0.0)),
    )
}

pub(crate) fn draw_sidebar_scroll<F>(
    runtime: &RefCell<crate::native_runtime_shell::RsUiShellRuntime>,
    id_salt: &'static str,
    ui: &mut egui::Ui,
    mut draw_content: F,
) where
    F: FnMut(&mut egui::Ui),
{
    if crate::native_explorer_paint::active(ui.ctx()) {
        draw_explorer_scroll(runtime, ui, &mut draw_content);
        return;
    }
    let viewport = ui.available_size();
    let offset = route_sidebar_wheel(runtime, ui, viewport);

    let mut scroll_area = egui::ScrollArea::vertical()
        .id_salt(id_salt)
        .auto_shrink([false, false])
        .max_height(viewport.y);
    if let Some(offset) = offset {
        scroll_area = scroll_area.vertical_scroll_offset(offset);
    }
    let output = scroll_area.show(ui, |ui| draw_content(ui));
    sync_sidebar_scroll(
        runtime,
        output.inner_rect.size(),
        output.content_size,
        output.state.offset,
    );
}

fn draw_explorer_scroll<F>(
    runtime: &RefCell<crate::native_runtime_shell::RsUiShellRuntime>,
    ui: &mut egui::Ui,
    draw_content: &mut F,
) where
    F: FnMut(&mut egui::Ui),
{
    let size = ui.available_size();
    let mut offset = route_sidebar_wheel(runtime, ui, size).unwrap_or(0.0);
    let (viewport, _) = ui.allocate_exact_size(size, Sense::hover());
    let previous = runtime.borrow().sidebar_scroll_state();
    let content_height = previous.map_or(size.y, |state| state.content_size.height);
    let scrollable = (content_height - size.y).max(0.0);
    let track = Rect::from_min_max(Pos2::new(viewport.right() - 8.0, viewport.top()), viewport.max);
    if scrollable > 0.0 {
        let track_response = ui.interact(track, ui.id().with("rs-ui-explorer-track"), Sense::click());
        let thumb_height = (size.y * size.y / content_height).max(24.0).min(size.y);
        let thumb_top = viewport.top() + offset / scrollable * (size.y - thumb_height);
        let thumb = Rect::from_min_size(Pos2::new(track.left(), thumb_top), vec2(8.0, thumb_height));
        let response = ui.interact(thumb, ui.id().with("rs-ui-explorer-thumb"), Sense::drag());
        if track_response.clicked() {
            if let Some(pointer) = track_response.interact_pointer_pos() {
                if !thumb.contains(pointer) && size.y > thumb_height {
                    offset = ((pointer.y - viewport.top() - thumb_height * 0.5) / (size.y - thumb_height) * scrollable)
                        .clamp(0.0, scrollable);
                }
            }
        }
        if response.dragged() && size.y > thumb_height {
            offset += ui.input(|input| input.pointer.delta().y) * scrollable / (size.y - thumb_height);
            offset = offset.clamp(0.0, scrollable);
        }
    }
    let width = (size.x - 10.0).max(0.0);
    let content_rect = Rect::from_min_size(viewport.min - vec2(0.0, offset), vec2(width, size.y));
    // ScrollArea uses the default child namespace; retain persisted expansion IDs.
    let mut content = ui.new_child(egui::UiBuilder::new().max_rect(content_rect));
    content.set_clip_rect(Rect::from_min_size(viewport.min, vec2(width, size.y)).intersect(ui.clip_rect()));
    draw_content(&mut content);
    let content_size = vec2(width, (content.min_rect().bottom() - content_rect.top()).max(0.0));
    sync_sidebar_scroll(runtime, size, content_size, vec2(0.0, offset));
    let current = runtime.borrow().sidebar_scroll_offset().unwrap_or(0.0);
    if current != offset {
        ui.ctx().request_repaint();
    }
    if content_size.y > size.y {
        let height = (size.y * size.y / content_size.y).max(24.0).min(size.y);
        let top = viewport.top() + current / (content_size.y - size.y) * (size.y - height);
        crate::native_explorer_paint::scrollbar(
            ui,
            Rect::from_min_size(Pos2::new(track.left(), top), vec2(6.0, height)),
            ui.visuals().widgets.inactive.fg_stroke.color,
        );
    }
}

fn route_sidebar_wheel(
    runtime: &RefCell<crate::native_runtime_shell::RsUiShellRuntime>,
    ui: &mut egui::Ui,
    viewport: egui::Vec2,
) -> Option<f32> {
    let scroll_rect = Rect::from_min_size(ui.cursor().min, viewport);
    if !ui.rect_contains_pointer(scroll_rect) {
        return runtime.try_borrow().ok()?.sidebar_scroll_offset();
    }
    let wheel_delta = ui.input(|input| input.smooth_scroll_delta.y);
    match runtime.try_borrow_mut() {
        Ok(mut runtime) => match runtime.scroll_sidebar(
            rs_ui_core::Size::new(viewport.x, viewport.y),
            rs_ui_core::Point::new(0.0, wheel_delta),
        ) {
            Ok(offset) => {
                if wheel_delta != 0.0 {
                    ui.ctx().input_mut(|input| input.smooth_scroll_delta.y = 0.0);
                }
                Some(offset)
            }
            Err(error) => {
                tracing::error!(%error, "rs-ui could not apply sidebar scroll input");
                runtime.sidebar_scroll_offset()
            }
        },
        Err(error) => {
            tracing::error!(%error, "rs-ui sidebar runtime is already borrowed");
            None
        }
    }
}

fn sync_sidebar_scroll(
    runtime: &RefCell<crate::native_runtime_shell::RsUiShellRuntime>,
    viewport: egui::Vec2,
    content_size: egui::Vec2,
    offset: egui::Vec2,
) {
    match runtime.try_borrow_mut() {
        Ok(mut runtime) => {
            if let Err(error) = runtime.sync_sidebar_scroll(
                rs_ui_core::Size::new(viewport.x, viewport.y),
                rs_ui_core::Size::new(content_size.x, content_size.y),
                rs_ui_core::Point::new(offset.x, offset.y),
            ) {
                tracing::error!(%error, "rs-ui could not sync sidebar scroll layout");
            }
        }
        Err(error) => tracing::error!(%error, "rs-ui sidebar runtime is already borrowed"),
    }
}

fn sidebar_ui(egui_context: &egui::Context, content_rect: Rect, layer_id: egui::LayerId) -> egui::Ui {
    let id = egui::Id::new("dbpro_sidebar_content");
    let mut ui = egui::Ui::new(
        egui_context.clone(),
        layer_id,
        id,
        egui::UiBuilder::new().max_rect(content_rect),
    );
    ui.set_clip_rect(content_rect.expand(SIDEBAR_CLIP_BLEED));
    ui.set_min_size(content_rect.size());
    ui.set_max_size(content_rect.size());
    let _ = ui.interact(content_rect, id.with("bg"), Sense::hover());
    ui
}

fn draw_resize_handle(
    context: &SidebarSurfaceContext<'_>,
    egui_context: &egui::Context,
    panel: &SidebarPanel,
) -> Option<f32> {
    let grip = egui_context.style().interaction.resize_grab_radius_side.max(5.0);
    let edge_x = panel.left + context.sidebar_width;
    let resize_rect = Rect::from_x_y_ranges((edge_x - grip)..=(edge_x + grip), panel.y_range);
    let id = egui::Id::new("dbpro_sidebar_resize");
    let layer_id = egui::LayerId::new(egui::Order::PanelResizeLine, id);
    let mut grip_ui = egui::Ui::new(
        egui_context.clone(),
        layer_id,
        id,
        egui::UiBuilder::new().max_rect(resize_rect),
    );
    grip_ui.set_clip_rect(egui_context.screen_rect());
    let response = grip_ui.allocate_rect(resize_rect, Sense::drag().union(Sense::focusable_noninteractive()));
    let hovering = response.hovered();
    let dragging = response.dragged();
    if hovering || dragging {
        egui_context.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    let pointer = egui_context
        .pointer_interact_pos()
        .or_else(|| response.interact_pointer_pos());
    let mut width = None;
    match context.runtime.try_borrow_mut() {
        Ok(mut runtime) => {
            if response.drag_started() {
                if let Some(pointer) = pointer {
                    let origin = egui_context
                        .input(|input| input.pointer.press_origin())
                        .unwrap_or(pointer);
                    if let Err(error) = runtime.begin_sidebar_resize(
                        context.sidebar_width,
                        SIDEBAR_MIN_WIDTH,
                        SIDEBAR_MAX_WIDTH,
                        rs_ui_core::Point::new(origin.x, origin.y),
                    ) {
                        tracing::error!(%error, "rs-ui could not begin sidebar resize");
                    }
                }
            }
            if dragging {
                if let Some(pointer) = pointer {
                    match runtime.update_sidebar_resize(rs_ui_core::Point::new(pointer.x, pointer.y)) {
                        Ok(next_width) => width = Some(next_width),
                        Err(error) => tracing::error!(%error, "rs-ui could not update sidebar resize"),
                    }
                }
            }
            if response.drag_stopped() {
                if let Err(error) = runtime.end_sidebar_resize() {
                    tracing::error!(%error, "rs-ui could not finish sidebar resize");
                }
            }
            if let Some(command) = focused_resize_command(egui_context, response.has_focus()) {
                match runtime.adjust_sidebar_width(context.sidebar_width, SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH, command)
                {
                    Ok(next_width) => width = Some(next_width),
                    Err(error) => tracing::error!(%error, "rs-ui could not adjust sidebar width"),
                }
            }
        }
        Err(error) => tracing::error!(%error, "rs-ui sidebar runtime is already borrowed"),
    }
    if dragging {
        egui_context.request_repaint();
    }
    let painter = egui_context.layer_painter(egui::LayerId::background());
    let line_x = painter.round_to_pixel_center(edge_x - 1.0);
    painter.vline(line_x, panel.y_range, resize_stroke(context.theme, hovering, dragging));
    width
}

fn focused_resize_command(ctx: &egui::Context, has_focus: bool) -> Option<rs_ui_runtime::BehaviorCommand> {
    if !has_focus {
        return None;
    }
    ctx.input(|input| {
        if input.key_pressed(egui::Key::ArrowLeft) {
            Some(rs_ui_runtime::BehaviorCommand::Decrement)
        } else if input.key_pressed(egui::Key::ArrowRight) {
            Some(rs_ui_runtime::BehaviorCommand::Increment)
        } else {
            None
        }
    })
}

fn resize_stroke(theme: DbProTheme, hovering: bool, dragging: bool) -> Stroke {
    if dragging {
        Stroke::new(1.5, theme.accent)
    } else if hovering {
        Stroke::new(1.0, theme.accent)
    } else {
        Stroke::new(1.0, theme.border_subtle)
    }
}
