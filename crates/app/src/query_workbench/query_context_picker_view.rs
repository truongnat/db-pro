//! Query connection/schema picker rendering and intent collection.
use super::*;
use egui::RichText;

pub(super) struct QueryContextPickerContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) current_connection_id: Option<&'a str>,
    pub(super) current_schema: &'a str,
    pub(super) available_schemas: &'a [String],
    pub(super) connections: &'a [(String, String, String)],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum QueryContextPickerAction {
    SelectConnection(String),
    SelectSchema(String),
    Close,
}

/// Preferred content-width band for the grouped picker.
const PICKER_MIN_WIDTH: f32 = 300.0;
const PICKER_MAX_WIDTH: f32 = 360.0;
/// Room the popup surface itself consumes: the frame margin on both sides plus
/// the border stroke. A content width must leave this much inside the viewport.
const SURFACE_INSET: f32 = 18.0;
/// Breathing room kept between the popup surface and the viewport edge.
const VIEWPORT_GUTTER: f32 = 16.0;
/// Longest list region inside the picker; longer lists scroll within it.
const LIST_MAX_HEIGHT: f32 = 180.0;
/// Vertical gap between option rows (shared `ui` item spacing default).
const LIST_ROW_GAP: f32 = 4.0;

/// Height of a list region: exactly the rows it holds, capped so a long list
/// scrolls instead of growing the popup.
///
/// The region is *allocated* rather than left to `ScrollArea::auto_shrink`,
/// because inside an auto-sized `Area` the child `Ui` only offers the height the
/// previous frame produced: a shrinking scroller then converges to a couple of
/// rows and the rest of a long list stays unreachable.
fn list_viewport_height(rows: usize) -> f32 {
    let row_height = crate::tokens::component::table::TABLE_ROW_HEIGHT_COMPACT + LIST_ROW_GAP;
    (rows as f32 * row_height).clamp(row_height, LIST_MAX_HEIGHT)
}

pub(super) fn draw_picker(
    context: &QueryContextPickerContext<'_>,
    ctx: &egui::Context,
    anchor: egui::Rect,
) -> Option<QueryContextPickerAction> {
    let mut action = None;
    let menu_position = egui::pos2(anchor.left(), anchor.bottom() + 4.0);
    let max_available = (ctx.content_rect().width() - VIEWPORT_GUTTER - SURFACE_INSET).max(1.0);
    let picker_width = anchor
        .width()
        .clamp(PICKER_MIN_WIDTH, PICKER_MAX_WIDTH)
        .min(max_available);

    let menu = FloatingPopup::new(egui::Id::new("query_context_picker"), context.theme, menu_position)
        .order(egui::Order::Foreground)
        .constrain_to(ctx.content_rect().shrink(4.0))
        .surface_elevated()
        .radius(8.0)
        .margin(egui::Margin::same(8.0 as i8))
        .shadow(false)
        .show(ctx, |ui| {
            ui.set_width(picker_width);
            ui.set_max_width(picker_width);
            ui.label(
                RichText::new(t!("query.context_connection"))
                    .small()
                    .strong()
                    .color(context.theme.text_muted),
            );
            ui.add_space(4.0);
            let connection_rows = context.connections.len().max(1);
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), list_viewport_height(connection_rows)),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ScrollArea::new(context.theme)
                        .id_salt("query_context_connections_scroll")
                        .max_height(ui.available_height())
                        .show(ui, |ui| {
                            for (id, name, environment) in context.connections {
                                let selected = context.current_connection_id == Some(id.as_str());
                                let label = if environment.is_empty() {
                                    name.clone()
                                } else {
                                    format!("{name} · {environment}")
                                };
                                if crate::components::select::paint_option(
                                    ui,
                                    crate::components::select::SelectOption {
                                        label: &label,
                                        selected,
                                        theme: context.theme,
                                        icon: None,
                                        disabled: false,
                                    },
                                )
                                .clicked()
                                {
                                    action = Some(QueryContextPickerAction::SelectConnection(id.clone()));
                                }
                            }
                        });
                },
            );
            Separator::horizontal(context.theme).show(ui);
            ui.label(
                RichText::new(t!("query.context_schema"))
                    .small()
                    .strong()
                    .color(context.theme.text_muted),
            );
            ui.add_space(4.0);
            // Schema lists grow with tenants (PostgreSQL search_paths, one schema per
            // tenant), so the region is sized to its rows and capped: a long list
            // scrolls inside a bounded window instead of being clipped away — UI-03.
            let schema_rows = context.available_schemas.len().max(1);
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), list_viewport_height(schema_rows)),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ScrollArea::new(context.theme)
                        .id_salt("query_context_schemas_scroll")
                        .max_height(ui.available_height())
                        .show(ui, |ui| {
                            for schema in context.available_schemas {
                                if crate::components::select::paint_option(
                                    ui,
                                    crate::components::select::SelectOption {
                                        label: schema,
                                        selected: context.current_schema == schema,
                                        theme: context.theme,
                                        icon: None,
                                        disabled: false,
                                    },
                                )
                                .clicked()
                                {
                                    action = Some(QueryContextPickerAction::SelectSchema(schema.clone()));
                                }
                            }
                        });
                },
            );
        });

    let clicked_outside = ctx.input(|input| {
        input.pointer.any_click()
            && input
                .pointer
                .interact_pos()
                .is_some_and(|position| !menu.response.rect.contains(position) && !anchor.contains(position))
    });
    let escape = ctx.input(|input| input.key_pressed(egui::Key::Escape));
    if clicked_outside || escape {
        Some(QueryContextPickerAction::Close)
    } else {
        action
    }
}

#[cfg(test)]
#[path = "tests/query_context_picker_view_tests.rs"]
mod tests;
