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
            ScrollArea::new(context.theme)
                .id_salt("query_context_connections_scroll")
                .auto_shrink([true, true])
                .max_height(180.0)
                .show(ui, |ui| {
                    if context.connections.is_empty() {
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new("No connections available")
                                .small()
                                .color(context.theme.text_muted),
                        );
                        ui.add_space(2.0);
                    } else {
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
                    }
                });
            Separator::horizontal(context.theme).show(ui);
            ui.label(
                RichText::new(t!("query.context_schema"))
                    .small()
                    .strong()
                    .color(context.theme.text_muted),
            );
            ui.add_space(4.0);
            // Schema lists grow with tenants (PostgreSQL search_paths, one schema per
            // tenant). Without a bounded scroll the popup clipped every schema past
            // the viewport, making them unreachable — UI-03.
            ScrollArea::new(context.theme)
                .id_salt("query_context_schemas_scroll")
                .auto_shrink([true, true])
                .max_height(180.0)
                .show(ui, |ui| {
                    if context.available_schemas.is_empty() {
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new("No schemas available")
                                .small()
                                .color(context.theme.text_muted),
                        );
                        ui.add_space(2.0);
                    } else {
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
                    }
                });
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
