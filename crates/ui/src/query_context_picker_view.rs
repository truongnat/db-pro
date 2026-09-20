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

pub(super) fn draw_picker(
    context: &QueryContextPickerContext<'_>,
    ctx: &egui::Context,
    anchor: egui::Rect,
) -> Option<QueryContextPickerAction> {
    let mut action = None;
    let menu_position = egui::pos2(anchor.left(), anchor.bottom() + 4.0);
    let menu = egui::Area::new(egui::Id::new("query_context_picker"))
        .order(egui::Order::Foreground)
        .fixed_pos(menu_position)
        .show(ctx, |ui| {
            egui::Frame {
                fill: context.theme.surface_elevated,
                inner_margin: egui::Margin::same(8.0),
                rounding: egui::Rounding::same(8.0),
                stroke: egui::Stroke::new(1.0, context.theme.border_subtle),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.set_min_width(280.0);
                ui.label(
                    RichText::new("Connection")
                        .small()
                        .strong()
                        .color(context.theme.text_muted),
                );
                ui.add_space(4.0);
                egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                    for (id, name, environment) in context.connections {
                        let selected = context.current_connection_id == Some(id.as_str());
                        let label = if environment.is_empty() {
                            name.clone()
                        } else {
                            format!("{name} · {environment}")
                        };
                        if ui.selectable_label(selected, label).clicked() {
                            action = Some(QueryContextPickerAction::SelectConnection(id.clone()));
                        }
                    }
                });
                ui.separator();
                ui.label(RichText::new("Schema").small().strong().color(context.theme.text_muted));
                ui.add_space(4.0);
                for schema in context.available_schemas {
                    if ui.selectable_label(context.current_schema == schema, schema).clicked() {
                        action = Some(QueryContextPickerAction::SelectSchema(schema.clone()));
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
    if clicked_outside {
        Some(QueryContextPickerAction::Close)
    } else {
        action
    }
}
