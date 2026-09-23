//! Sidebar shell layout, chrome and resize interaction.
use super::super::*;
use super::sidebar_chrome_view::{SidebarChromeAction, SidebarChromeContext};
use egui::{vec2, Pos2, Rect, Sense, Stroke};

const SIDEBAR_CLIP_BLEED: f32 = 1.0;

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
}

struct SidebarPanel {
    left: f32,
    y_range: egui::Rangef,
    full: Rect,
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
    let response = egui::SidePanel::left("sidebar")
        .resizable(false)
        .exact_width(context.sidebar_width)
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
            let full = Rect::from_min_size(panel_origin, vec2(context.sidebar_width, ui.max_rect().height()));
            ui.painter()
                .rect_filled(full, egui::Rounding::ZERO, context.theme.surface_panel);
            ui.allocate_rect(full, Sense::hover());
        });
    let left = response.response.rect.left();
    let y_range = response.response.rect.y_range();
    SidebarPanel {
        left,
        y_range,
        full: Rect::from_min_size(
            Pos2::new(left, response.response.rect.top()),
            vec2(context.sidebar_width, response.response.rect.height()),
        ),
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
    let mut ui = sidebar_ui(egui_context, content_rect);
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

fn sidebar_ui(egui_context: &egui::Context, content_rect: Rect) -> egui::Ui {
    let id = egui::Id::new("dbpro_sidebar_content");
    let layer_id = egui::LayerId::new(egui::Order::Middle, id);
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
    let response = grip_ui.allocate_rect(resize_rect, Sense::drag());
    let hovering = response.hovered();
    let dragging = response.dragged();
    if hovering || dragging {
        egui_context.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    let width = if dragging {
        egui_context
            .pointer_interact_pos()
            .or_else(|| response.interact_pointer_pos())
            .map(|pointer| pointer.x - panel.left)
    } else {
        None
    };
    if dragging {
        egui_context.request_repaint();
    }
    let painter = egui_context.layer_painter(egui::LayerId::background());
    let line_x = painter.round_to_pixel_center(edge_x - 1.0);
    painter.vline(line_x, panel.y_range, resize_stroke(context.theme, hovering, dragging));
    width
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
